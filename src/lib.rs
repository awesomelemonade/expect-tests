// based on: https://github.com/rust-analyzer/expect-test/blob/master/src/lib.rs

use std::{
    collections::HashMap,
    env,
    fmt::{self},
    fs, mem,
    ops::Range,
    panic,
    path::{Path, PathBuf},
    sync::Mutex,
};

use once_cell::sync::{Lazy, OnceCell};
const HELP: &str = "
You can update all `expect!` tests by running:
    env UPDATE_EXPECT=1 cargo test
To update a single test, place the cursor on `expect` token and use `run` feature of rust-analyzer.
";

fn update_expect() -> bool {
    std::env::var("UPDATE_EXPECT").is_ok()
}

#[macro_export]
macro_rules! expect {
    ($actual:literal) => {
        $crate::Expect {
            file_position: $crate::FilePosition {
                file: file!(),
                line: line!(),
                column: column!(),
            },
            raw_actual: stringify!($actual),
            expected: None,
            raw_expected: None,
        }
        .assert_eq($actual)
    };
    ($actual:literal, $expected:literal) => {
        $crate::Expect {
            file_position: $crate::FilePosition {
                file: file!(),
                line: line!(),
                column: column!(),
            },
            raw_actual: stringify!($actual),
            expected: Some($expected),
            raw_expected: Some(stringify!($expected)),
        }
        .assert_eq($actual)
    };
    ($actual:expr) => {
        $crate::Expect {
            file_position: $crate::FilePosition {
                file: file!(),
                line: line!(),
                column: column!(),
            },
            raw_actual: stringify!($actual),
            expected: None,
            raw_expected: None,
        }
        .assert_debug_eq($actual)
    };
    ($actual:expr, $expected:literal) => {
        $crate::Expect {
            file_position: $crate::FilePosition {
                file: file!(),
                line: line!(),
                column: column!(),
            },
            raw_actual: stringify!($actual),
            expected: Some($expected),
            raw_expected: Some(stringify!($expected)),
        }
        .assert_debug_eq($actual)
    };
}

/// Self-updating string literal.
#[derive(Debug)]
pub struct Expect {
    #[doc(hidden)]
    pub file_position: FilePosition,
    #[doc(hidden)]
    pub raw_actual: &'static str,
    #[doc(hidden)]
    pub expected: Option<&'static str>,
    #[doc(hidden)]
    pub raw_expected: Option<&'static str>,
}

#[derive(Debug)]
pub struct FilePosition {
    #[doc(hidden)]
    pub file: &'static str,
    #[doc(hidden)]
    pub line: u32,
    #[doc(hidden)]
    pub column: u32,
}

impl fmt::Display for FilePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

#[derive(Debug, Clone, Copy)]
enum StrLitKind {
    Normal,     // use ""
    Raw(usize), // use r#""# with variable number of #'s
}

impl StrLitKind {
    fn write_start(&self, w: &mut impl std::fmt::Write) -> std::fmt::Result {
        match self {
            Self::Normal => write!(w, "\""),
            Self::Raw(n) => {
                write!(w, "r")?;
                for _ in 0..*n {
                    write!(w, "#")?;
                }
                write!(w, "\"")
            }
        }
    }

    fn write_end(&self, w: &mut impl std::fmt::Write) -> std::fmt::Result {
        match self {
            Self::Normal => write!(w, "\""),
            Self::Raw(n) => {
                write!(w, "\"")?;
                for _ in 0..*n {
                    write!(w, "#")?;
                }
                Ok(())
            }
        }
    }
}

impl Expect {
    fn trimmed(&self, text: &str) -> String {
        if text.contains('\n') {
            let text = if text.starts_with('\n') {
                &text[1..]
            } else {
                text
            };
            let indent_amount = text
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.len() - line.trim_start().len())
                .min()
                .unwrap_or(0);

            let mut trimmed = text
                .lines()
                .map(|line| {
                    if line.len() < indent_amount {
                        ""
                    } else {
                        &line[indent_amount..]
                    }
                })
                .collect::<Vec<&str>>()
                .join("\n");
            // text always contains at least 1 character (because of if text.contains('\n'))
            if text.chars().last().unwrap() == '\n' {
                trimmed.push('\n');
            }
            trimmed
        } else {
            text.to_string()
        }
    }

    pub fn assert_eq(&self, actual: &str) {
        if let Some(expected) = self.expected {
            let expected = self.trimmed(expected);
            if expected != actual {
                Runtime::fail_expect(self, &expected, actual);
            }
        } else {
            Runtime::fail_expect(self, "", actual);
        }
    }

    pub fn assert_debug_eq<T>(&self, actual: T)
    where
        T: fmt::Debug,
    {
        let actual = format!("{:#?}", actual);
        self.assert_eq(&actual)
    }

    fn find_expect_location(&self, file: &str) -> ExpectLocation {
        let line_number: usize = (self.file_position.line - 1).try_into().unwrap(); // Zero-indexed
        let column_number: usize = (self.file_position.column - 1).try_into().unwrap(); // Zero-indexed
        let line_byte_offset = if line_number == 0 {
            0
        } else {
            // Add 1 to skip the newline character
            file.match_indices("\n").nth(line_number - 1).unwrap().0 + 1
        };
        let macro_byte_offset = line_byte_offset
            + (&file[line_byte_offset..])
                .char_indices()
                .skip(column_number)
                .skip_while(|&(_, c)| c != '!') // expect
                .skip(1) // !
                .next()
                .expect("Failed to locate macro")
                .0; // extract index from (index, char)

        let actual_byte_offset = macro_byte_offset
            + (&file[macro_byte_offset..])
                .find(self.raw_actual)
                .expect("Unable to find actual");
        let actual_range = actual_byte_offset..(actual_byte_offset + self.raw_actual.len());

        let expected_range = if let Some(raw_expected) = self.raw_expected {
            let expect_byte_offset = actual_byte_offset
                + (&file[actual_byte_offset..])
                    .find(raw_expected)
                    .expect("Unable to find expected");
            expect_byte_offset..(expect_byte_offset + raw_expected.len())
        } else {
            let expect_byte_offset = actual_byte_offset + self.raw_actual.len();
            expect_byte_offset..expect_byte_offset
        };
        let line_indent = (&file[line_byte_offset..])
            .chars()
            .take_while(|&c| c == ' ')
            .count();

        ExpectLocation {
            line_indent,
            actual_range,
            expected_range,
        }
    }
}

#[derive(Default)]
struct Runtime {
    help_printed: bool,
    per_file: HashMap<&'static str, FileRuntime>,
}
static RT: Lazy<Mutex<Runtime>> = Lazy::new(Default::default);

impl Runtime {
    fn fail_expect(expect: &Expect, expected: &str, actual: &str) {
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
        let print_help = !mem::replace(&mut self.help_printed, true);
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
            position.to_string(),
            help,
            expected,
            actual,
            format_chunks(diff)
        );
        // Use resume_unwind instead of panic!() to prevent a backtrace, which is unnecessary noise.
        panic::resume_unwind(Box::new(()));
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
        let original_text = fs::read_to_string(&path).unwrap();
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
        fs::write(&self.path, &self.patchwork.text).unwrap()
    }
}

#[derive(Debug)]
struct ExpectLocation {
    line_indent: usize,
    actual_range: Range<usize>,
    expected_range: Range<usize>,
}

#[derive(Debug)]
struct Patchwork {
    text: String,
    indels: Vec<(Range<usize>, usize)>,
}

impl Patchwork {
    fn new(text: String) -> Patchwork {
        Patchwork {
            text,
            indels: Vec::new(),
        }
    }
    fn patch_insert(&mut self, offset: usize, patch: &str) {
        self.patch_range(offset..offset, patch)
    }
    fn patch_range(&mut self, range: Range<usize>, patch: &str) {
        let (delete, insert) = self
            .indels
            .iter()
            .take_while(|(delete, _)| delete.start < range.start)
            .map(|(delete, insert)| (delete.end - delete.start, insert))
            .fold((0usize, 0usize), |(x1, y1), (x2, y2)| (x1 + x2, y1 + y2));

        let offset = insert - delete;
        self.text
            .replace_range((range.start + offset)..(range.end + offset), &patch);

        self.indels.push((range, patch.len()));
        self.indels.sort_by_key(|(delete, _insert)| delete.start);
    }
}

fn lit_kind_for_patch(patch: &str) -> StrLitKind {
    let has_double_quote = patch.chars().any(|c| c == '"');
    if has_double_quote {
        // Find the maximum number of hashes that follow a double quote in the string.
        // We need to use one more than that to delimit the string.
        let max_hashes = patch
            .split('"')
            .map(|s: &str| s.chars().take_while(|&c| c == '#').count())
            .max()
            .unwrap();
        StrLitKind::Raw(max_hashes + 1)
    } else {
        let has_backslash_or_newline = patch.chars().any(|c| matches!(c, '\\' | '\n'));
        if has_backslash_or_newline {
            StrLitKind::Raw(1)
        } else {
            StrLitKind::Normal
        }
    }
}

fn format_patch(desired_indent: usize, patch: &str) -> String {
    let lit_kind = lit_kind_for_patch(patch);
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
        if patch.chars().last().unwrap() == '\n' {
            buf.push('\n');
        }
    } else {
        buf.push_str(patch);
    }
    lit_kind.write_end(&mut buf).unwrap();
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
            if let Ok(workspace_root) = env::var("CARGO_WORKSPACE_DIR") {
                return Ok(workspace_root.into());
            }

            // If a hack isn't used, we use a heuristic to find the "top-level" workspace.
            // This fails in some cases, see https://github.com/rust-analyzer/expect-test/issues/33
            let my_manifest = env::var("CARGO_MANIFEST_DIR")?;
            let workspace_root = Path::new(&my_manifest)
                .ancestors()
                .filter(|it| it.join("Cargo.toml").exists())
                .last()
                .unwrap()
                .to_path_buf();

            Ok(workspace_root)
        })
        .unwrap_or_else(|_: env::VarError| {
            panic!(
                "No CARGO_MANIFEST_DIR env var and the path is relative: {}",
                path.display()
            )
        })
        .join(path)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trivial_assert_empty_literal() {
        expect!("", "");
    }

    #[test]
    fn test_trivial_assert_literal() {
        expect!("ABC", "ABC");
    }

    #[test]
    fn test_trivial_assert_expression() {
        expect!(&5, "5");
    }

    #[test]
    fn test_trivial_assert_expression2() {
        let x = 5;
        expect!(x, "5");
    }

    #[test]
    fn test_vec() {
        let v = vec![1];
        expect!(
            v,
            r#"
            [
                1,
            ]"#
        );
    }

    #[test]
    fn test_spacing_assert() {
        expect!(
            "\n", r#"

"#
        );
        expect!(
            " \n", r#"
 
"#
        );
        expect!(
            "\n ", r#"

 "#
        );
        expect!(
            "
                ",
            r#"

                "#
        );
    }

    #[test]
    fn test_lit_kind_for_patch_empty() {
        expect!(&lit_kind_for_patch(""), "Normal");
    }

    #[test]
    fn test_lit_kind_for_patch_normal() {
        expect!(&lit_kind_for_patch("ABCDEFG"), "Normal");
        expect!(&lit_kind_for_patch("single line"), "Normal");
    }

    #[test]
    fn test_lit_kind_for_patch_new_lines() {
        expect!(
            &lit_kind_for_patch("hello\nworld\n"),
            r#"
            Raw(
                1,
            )"#
        );
    }

    #[test]
    fn test_lit_kind_for_patch_tabs() {
        expect!(
            &lit_kind_for_patch(r"hello\tworld"),
            r#"
            Raw(
                1,
            )"#
        );
    }

    #[test]
    fn test_lit_kind_for_patch_double_quotes() {
        expect!(
            &lit_kind_for_patch("{\"foo\": 42}"),
            r#"
            Raw(
                1,
            )"#
        );
    }

    #[test]
    fn test_lit_kind_for_patch_double_quote_hash() {
        expect!(
            &lit_kind_for_patch("\"#\""),
            r#"
            Raw(
                2,
            )"#
        );
    }

    #[test]
    fn test_lit_kind_for_patch_double_quote_triple_hash() {
        expect!(
            &lit_kind_for_patch("\"###\""),
            r#"
            Raw(
                4,
            )"#
        );
    }

    #[test]
    fn test_format_patch_multi_line() {
        let patch = format_patch(0, "hello\nworld\n");
        expect!(&patch, r##""r#\"\n    hello\n    world\n\"#""##);
    }

    #[test]
    fn test_format_patch_single_line() {
        let patch = format_patch(0, "single line");
        expect!(&patch, r#""\"single line\"""#);
    }

    #[test]
    fn test_patchwork() {
        let mut patchwork = Patchwork::new("one two three".to_string());
        patchwork.patch_range(4..7, "zwei");
        patchwork.patch_range(0..3, "один");
        patchwork.patch_range(8..13, "3");
        patchwork.patch_insert(13, "333");
        expect!(
            &patchwork,
            r#"
            Patchwork {
                text: "один zwei 3333",
                indels: [
                    (
                        0..3,
                        8,
                    ),
                    (
                        4..7,
                        4,
                    ),
                    (
                        8..13,
                        1,
                    ),
                    (
                        13..13,
                        3,
                    ),
                ],
            }"#
        );
    }
}
