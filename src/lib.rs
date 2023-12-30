#![feature(let_chains)]
// based on: https://github.com/rust-analyzer/expect-test/blob/master/src/lib.rs

pub mod expect;
#[cfg(feature = "expect-tokens")]
pub mod expect_tokens;
mod patchwork;
mod runtime;
mod str_lit_kind;

#[cfg(test)]
mod tests;

#[cfg(feature = "expect-tokens")]
#[cfg(test)]
mod expect_tokens_tests;

#[macro_export]
macro_rules! expect {
    ($actual:literal) => {
        {
            static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let index = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            $crate::expect::Expect {
                file_position: $crate::expect::FilePosition {
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
            $crate::expect::Expect {
                file_position: $crate::expect::FilePosition {
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
            $crate::expect::Expect {
                file_position: $crate::expect::FilePosition {
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
            $crate::expect::Expect {
                file_position: $crate::expect::FilePosition {
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

#[cfg(feature = "expect-tokens")]
#[macro_export]
macro_rules! expect_tokens {
    ($actual:expr) => {
        {
            static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let index = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            $crate::expect::Expect {
                file_position: $crate::expect::FilePosition {
                    file: file!(),
                    line: line!(),
                    column: column!(),
                },
                raw_actual: stringify!($actual),
                expected: [],
                raw_expected: [],
                assertion_index: index,
            }
            .assert_eq(&$crate::expect_tokens::ExpectTokens::convert($actual))
        }
    };
    ($actual:expr, $($expected:literal),*) => {
        {
            static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let index = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            $crate::expect::Expect {
                file_position: $crate::expect::FilePosition {
                    file: file!(),
                    line: line!(),
                    column: column!(),
                },
                raw_actual: stringify!($actual),
                expected: [$($expected),*],
                raw_expected: [$(stringify!($expected)),*],
                assertion_index: index,
            }
            .assert_eq(&$crate::expect_tokens::ExpectTokens::convert($actual))
        }
    };
}
