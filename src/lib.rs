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

impl std::fmt::Display for FilePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

impl Expect {
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
        T: std::fmt::Debug,
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
            file.match_indices('\n').nth(line_number - 1).unwrap().0 + 1
        };
        let macro_byte_offset = line_byte_offset
            + file[line_byte_offset..]
                .char_indices()
                .skip(column_number)
                .skip_while(|&(_, c)| c != '!') // expect
                .nth(1) // !
                .expect("Failed to locate macro")
                .0; // extract index from (index, char)

        let actual_byte_offset = macro_byte_offset
            + file[macro_byte_offset..]
                .find(self.raw_actual)
                .expect("Unable to find actual");
        let actual_range = actual_byte_offset..(actual_byte_offset + self.raw_actual.len());

        let expected_range = if let Some(raw_expected) = self.raw_expected {
            let expect_byte_offset = actual_byte_offset
                + file[actual_byte_offset..]
                    .find(raw_expected)
                    .expect("Unable to find expected");
            expect_byte_offset..(expect_byte_offset + raw_expected.len())
        } else {
            let expect_byte_offset = actual_byte_offset + self.raw_actual.len();
            expect_byte_offset..expect_byte_offset
        };
        let line_indent = file[line_byte_offset..]
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

#[derive(Debug)]
struct ExpectLocation {
    line_indent: usize,
    actual_range: Range<usize>,
    expected_range: Range<usize>,
}
