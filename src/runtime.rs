use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Mutex,
};

use once_cell::sync::{Lazy, OnceCell};

use crate::{patchwork::Patchwork, str_lit_kind::StrLitKind, update_expect, Expect, FilePosition};
const HELP: &str = "
You can update all `expect!` tests by running:
    env UPDATE_EXPECT=1 cargo test
To update a single test, place the cursor on `expect` token and use `run` feature of rust-analyzer.
";

#[derive(Default)]
pub struct Runtime {
    help_printed: bool,
    per_file: HashMap<&'static str, FileRuntime>,
}
static RT: Lazy<Mutex<Runtime>> = Lazy::new(Default::default);

impl Runtime {
    pub fn fail_expect(expect: &Expect, expected: &str, actual: &str) {
        let mut rt = RT.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if update_expect() {
            println!("\x1b[1m\x1b[92mupdating\x1b[0m: {}", expect.file_position);
            rt.per_file
                .entry(expect.file_position.file)
                .or_insert_with(|| FileRuntime::new(expect))
                .update(expect, actual);
            return;
        }
        rt.panic(&expect.file_position, expected, actual);
    }
    fn panic(&mut self, position: &FilePosition, expected: &str, actual: &str) {
        let print_help = !std::mem::replace(&mut self.help_printed, true);
        let help = if print_help { HELP } else { "" };

        let diff = dissimilar::diff(expected, actual);

        println!(
            "\n
\x1b[1m\x1b[91merror\x1b[97m: expect test failed\x1b[0m
   \x1b[1m\x1b[34m-->\x1b[0m {}
{}
\x1b[1mExpect\x1b[0m:
----
{}
----

\x1b[1mActual\x1b[0m:
----
{}
----

\x1b[1mDiff\x1b[0m:
----
{}
----
",
            position,
            help,
            expected,
            actual,
            format_chunks(diff)
        );
        // Use resume_unwind instead of panic!() to prevent a backtrace, which is unnecessary noise.
        std::panic::resume_unwind(Box::new(()));
    }
}

struct FileRuntime {
    path: PathBuf,
    original_text: String,
    patchwork: Patchwork,
}

impl FileRuntime {
    fn new(expect: &Expect) -> FileRuntime {
        let path = to_abs_ws_path(Path::new(expect.file_position.file));
        let original_text = std::fs::read_to_string(&path).unwrap();
        let patchwork = Patchwork::new(original_text.clone());
        FileRuntime {
            path,
            original_text,
            patchwork,
        }
    }
    fn update(&mut self, expect: &Expect, actual: &str) {
        let loc = expect.find_expect_location(&self.original_text);

        let patch = format_patch(loc.line_indent, actual);
        let patch = if expect.raw_expected.is_none() {
            let is_multiline = patch.contains('\n');
            if is_multiline {
                let indent = " ".repeat(loc.line_indent);
                self.patchwork
                    .patch_insert(loc.actual_range.start, &format!("\n{indent}    "));
                format!(",\n{indent}    {patch}\n{indent}")
            } else {
                format!(", {patch}")
            }
        } else {
            patch
        };
        self.patchwork.patch_range(loc.expected_range, &patch);
        std::fs::write(&self.path, self.patchwork.text()).unwrap()
    }
}

fn format_chunks(chunks: Vec<dissimilar::Chunk>) -> String {
    let mut buf = String::new();
    for chunk in chunks {
        let formatted = match chunk {
            dissimilar::Chunk::Equal(text) => text.into(),
            dissimilar::Chunk::Delete(text) => format!("\x1b[41m{}\x1b[0m", text),
            dissimilar::Chunk::Insert(text) => format!("\x1b[42m{}\x1b[0m", text),
        };
        buf.push_str(&formatted);
    }
    buf
}

fn to_abs_ws_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_owned();
    }

    static WORKSPACE_ROOT: OnceCell<PathBuf> = OnceCell::new();
    WORKSPACE_ROOT
        .get_or_try_init(|| {
            // Until https://github.com/rust-lang/cargo/issues/3946 is resolved, this
            // is set with a hack like https://github.com/rust-lang/cargo/issues/3946#issuecomment-973132993
            if let Ok(workspace_root) = std::env::var("CARGO_WORKSPACE_DIR") {
                return Ok(workspace_root.into());
            }

            // If a hack isn't used, we use a heuristic to find the "top-level" workspace.
            // This fails in some cases, see https://github.com/rust-analyzer/expect-test/issues/33
            let my_manifest = std::env::var("CARGO_MANIFEST_DIR")?;
            let workspace_root = Path::new(&my_manifest)
                .ancestors()
                .filter(|it| it.join("Cargo.toml").exists())
                .last()
                .unwrap()
                .to_path_buf();

            Ok(workspace_root)
        })
        .unwrap_or_else(|_: std::env::VarError| {
            panic!(
                "No CARGO_MANIFEST_DIR env var and the path is relative: {}",
                path.display()
            )
        })
        .join(path)
}

pub fn format_patch(desired_indent: usize, patch: &str) -> String {
    let lit_kind = StrLitKind::from(patch);
    let indent = " ".repeat(desired_indent);
    let is_multiline = patch.contains('\n');

    let mut buf = String::new();
    lit_kind.write_start(&mut buf).unwrap();
    if is_multiline {
        for line in patch.lines() {
            buf.push('\n');
            if !line.trim().is_empty() {
                buf.push_str(&indent);
                buf.push_str("    ");
            }
            buf.push_str(line);
        }
        if patch.ends_with('\n') {
            buf.push('\n');
        }
    } else {
        buf.push_str(patch);
    }
    lit_kind.write_end(&mut buf).unwrap();
    buf
}
