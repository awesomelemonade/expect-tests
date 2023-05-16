use crate::{patchwork::Patchwork, runtime::format_patch, str_lit_kind::StrLitKind};

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
    expect!(StrLitKind::from(""), "Normal");
}

#[test]
fn test_lit_kind_for_patch_normal() {
    expect!(StrLitKind::from("ABCDEFG"), "Normal");
    expect!(StrLitKind::from("single line"), "Normal");
}

#[test]
fn test_lit_kind_for_patch_new_lines() {
    expect!(
        StrLitKind::from("hello\nworld\n"),
        r#"
            Raw(
                1,
            )"#
    );
}

#[test]
fn test_lit_kind_for_patch_tabs() {
    expect!(
        StrLitKind::from(r"hello\tworld"),
        r#"
            Raw(
                1,
            )"#
    );
}

#[test]
fn test_lit_kind_for_patch_double_quotes() {
    expect!(
        StrLitKind::from("{\"foo\": 42}"),
        r#"
            Raw(
                1,
            )"#
    );
}

#[test]
fn test_lit_kind_for_patch_double_quote_hash() {
    expect!(
        StrLitKind::from("\"#\""),
        r#"
            Raw(
                2,
            )"#
    );
}

#[test]
fn test_lit_kind_for_patch_double_quote_triple_hash() {
    expect!(
        StrLitKind::from("\"###\""),
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
