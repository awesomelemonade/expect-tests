use crate::runtime::Runtime;
use std::ops::Range;

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
    pub fn find_expect_location(&self, file_contents: &str) -> ExpectLocation<N> {
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
                .skip_while(|&(_, c)| c != '!') // macro location (ex: "expect" and "expect_tokens")
                .nth(1) // !
                .expect("Failed to locate macro")
                .0; // extract index from (index, char)

        fn find_ignore_whitespace(haystack: &str, pattern: &str) -> Option<(usize, usize)> {
            fn validate(haystack: &str, pattern: &str) -> Option<(usize, usize)> {
                let s = haystack.trim_start();
                if s.starts_with(pattern) {
                    let start_index = haystack.len() - s.len();
                    Some((start_index, start_index + pattern.len()))
                } else {
                    None
                }
            }

            fn validate_fallback(haystack: &str, pattern: &str) -> Option<(usize, usize)> {
                let mut haystack_iterator = haystack.chars().peekable();
                let mut index = 0;
                while let Some(_whitespace) = haystack_iterator.next_if(|c| c.is_whitespace()) {
                    index += 1;
                }
                let start_index = index;
                for pattern_char in pattern.chars().filter(|c| !c.is_whitespace()) {
                    while let Some(_whitespace) = haystack_iterator.next_if(|c| c.is_whitespace()) {
                        index += 1;
                    }
                    if let Some(c) = haystack_iterator.next()
                        && c == pattern_char
                    {
                        index += 1;
                    } else {
                        return None;
                    }
                }
                Some((start_index, index))
            }
            // First we're going to skip to the next character, to account for the parentheses
            // then we're going to trim_start() and see if starts_with directly works
            //
            // Because stringify! does not return the raw source code in all
            // situations, we must have some fallback implementation in case
            // directly matching does not work. In this fallback case, we are
            // going to process char by char, ignoring whitespace
            let trimmed = haystack.trim_start();
            // trim the left parentheses in expect!()
            let trimmed = trimmed.get(1..)?;

            let num_trimmed = haystack.len() - trimmed.len();
            let (start, end) =
                validate(trimmed, pattern).or_else(|| validate_fallback(trimmed, pattern))?;
            Some((num_trimmed + start, num_trimmed + end))
        }
        let (actual_start, actual_end) =
            find_ignore_whitespace(&file_contents[macro_byte_offset..], self.raw_actual)
                .unwrap_or_else(|| {
                    panic!(
                        "Unable to find actual: `{}` in `{}`",
                        self.raw_actual,
                        &file_contents[macro_byte_offset..]
                    )
                });
        let actual_byte_offset = macro_byte_offset + actual_start;
        let mut current_offset = macro_byte_offset + actual_end;

        // let actual_byte_offset = macro_byte_offset
        //     + file_contents[macro_byte_offset..]
        //         .find(self.raw_actual)
        //         .unwrap_or_else(|| {
        //             panic!(
        //                 "Unable to find actual: `{}` in `{}`",
        //                 self.raw_actual,
        //                 &file_contents[macro_byte_offset..]
        //             )
        //         });
        // let actual_range = actual_byte_offset..(actual_byte_offset + self.raw_actual.len());
        // let mut current_offset = actual_byte_offset + self.raw_actual.len();

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
pub struct ExpectLocation<const N: usize> {
    pub line_indent: usize,
    pub expected_ranges: [Range<usize>; N],
    pub start_index: usize,
    pub end_index: usize,
}
