// based on: https://github.com/rust-analyzer/expect-test/blob/master/src/lib.rs

use std::ops::Range;

use runtime::Runtime;

mod patchwork;
mod runtime;
mod str_lit_kind;

#[cfg(test)]
mod tests;

fn update_expect() -> bool {
    std::env::var("UPDATE_EXPECT").is_ok()
}

#[macro_export]
macro_rules! expect {
    ($actual:literal) => {
        {
            static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let index = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            $crate::Expect {
                file_position: $crate::FilePosition {
                    file: file!(),
                    line: line!(),
                    column: column!(),
                },
                raw_actual: stringify!($actual),
                expected: [],
                raw_expected: [],
                assertion_index: index,
            }
            .assert_eq($actual)
        }
    };
    ($actual:expr) => {
        {
            static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let index = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            $crate::Expect {
                file_position: $crate::FilePosition {
                    file: file!(),
                    line: line!(),
                    column: column!(),
                },
                raw_actual: stringify!($actual),
                expected: [],
                raw_expected: [],
                assertion_index: index,
            }
            .assert_debug_eq($actual)
        }
    };
    ($actual:literal, $($expected:literal),*) => {
        {
            static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let index = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            $crate::Expect {
                file_position: $crate::FilePosition {
                    file: file!(),
                    line: line!(),
                    column: column!(),
                },
                raw_actual: stringify!($actual),
                expected: [$($expected),*],
                raw_expected: [$(stringify!($expected)),*],
                assertion_index: index,
            }
            .assert_eq($actual)
        }
    };
    ($actual:expr, $($expected:literal),*) => {
        {
            static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let index = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            $crate::Expect {
                file_position: $crate::FilePosition {
                    file: file!(),
                    line: line!(),
                    column: column!(),
                },
                raw_actual: stringify!($actual),
                expected: [$($expected),*],
                raw_expected: [$(stringify!($expected)),*],
                assertion_index: index,
            }
            .assert_debug_eq($actual)
        }
    };
}

/// Self-updating string literal.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Expect<const N: usize> {
    #[doc(hidden)]
    pub file_position: FilePosition,
    #[doc(hidden)]
    pub raw_actual: &'static str,
    #[doc(hidden)]
    pub expected: [&'static str; N],
    #[doc(hidden)]
    pub raw_expected: [&'static str; N],
    #[doc(hidden)]
    pub assertion_index: usize,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FilePosition {
    #[doc(hidden)]
    pub file: &'static str,
    #[doc(hidden)]
    pub line: u32,
    #[doc(hidden)]
    pub column: u32,
}

impl std::fmt::Display for FilePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

impl<const N: usize> Expect<N> {
    fn trimmed(&self, text: &str) -> String {
        if text.contains('\n') {
            let text = text.strip_prefix('\n').unwrap_or(text);
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
            if text.ends_with('\n') {
                trimmed.push('\n');
            }
            trimmed
        } else {
            text.to_string()
        }
    }

    pub fn assert_eq(&self, actual: &str) {
        if let Some(expected) = self.expected.get(self.assertion_index) {
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
        T: std::fmt::Debug,
    {
        let actual = format!("{:#?}", actual);
        self.assert_eq(&actual)
    }

    fn find_expect_location(&self, file_contents: &str) -> ExpectLocation<N> {
        let line_number: usize = (self.file_position.line - 1).try_into().unwrap(); // Zero-indexed
        let column_number: usize = (self.file_position.column - 1).try_into().unwrap(); // Zero-indexed
        let line_byte_offset = if line_number == 0 {
            0
        } else {
            // Add 1 to skip the newline character
            file_contents
                .match_indices('\n')
                .nth(line_number - 1)
                .unwrap()
                .0
                + 1
        };
        let macro_byte_offset = line_byte_offset
            + file_contents[line_byte_offset..]
                .char_indices()
                .skip(column_number)
                .skip_while(|&(_, c)| c != '!') // expect
                .nth(1) // !
                .expect("Failed to locate macro")
                .0; // extract index from (index, char)

        let actual_byte_offset = macro_byte_offset
            + file_contents[macro_byte_offset..]
                .find(self.raw_actual)
                .expect("Unable to find actual");
        // let actual_range = actual_byte_offset..(actual_byte_offset + self.raw_actual.len());

        let mut current_offset = actual_byte_offset + self.raw_actual.len();

        let expected_ranges = self.raw_expected.map(|raw_expected| {
            let start = current_offset
                + file_contents[current_offset..]
                    .find(raw_expected)
                    .expect("Unable to find expected");
            let end = start + raw_expected.len();
            current_offset = end;
            start..end
        });

        let start_index = actual_byte_offset;
        let end_index = current_offset;

        let line_indent = file_contents[line_byte_offset..]
            .chars()
            .take_while(|&c| c == ' ')
            .count();

        ExpectLocation {
            line_indent,
            expected_ranges,
            start_index,
            end_index,
        }
    }
}

#[derive(Debug)]
struct ExpectLocation<const N: usize> {
    line_indent: usize,
    expected_ranges: [Range<usize>; N],
    start_index: usize,
    end_index: usize,
}
