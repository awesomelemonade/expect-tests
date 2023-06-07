use crate::{
    patchwork::{PatchOrdering, Patchwork},
    runtime::format_patch,
    str_lit_kind::StrLitKind,
};

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
    patchwork.patch_range(4..7, "zwei", PatchOrdering::Normal);
    patchwork.patch_range(0..3, "один", PatchOrdering::Normal);
    patchwork.patch_range(8..13, "3", PatchOrdering::Normal);
    patchwork.patch_insert(13, "333", PatchOrdering::Normal);
    expect!(
        &patchwork,
        r#"
        Patchwork {
            text: "один zwei 3333",
            patches: [
                Patch {
                    deletion_range: 0..3,
                    insertion_size: 8,
                    ordering: Normal,
                },
                Patch {
                    deletion_range: 4..7,
                    insertion_size: 4,
                    ordering: Normal,
                },
                Patch {
                    deletion_range: 8..13,
                    insertion_size: 1,
                    ordering: Normal,
                },
                Patch {
                    deletion_range: 13..13,
                    insertion_size: 3,
                    ordering: Normal,
                },
            ],
        }"#
    );
}

#[test]
pub fn test_multi_expect() {
    for i in 0..2 {
        let j = i..i + 1;
        let z = (i, i);
        expect!(j, "0..1", "1..2");
        expect!(
            z,
            r#"
            (
                0,
                0,
            )"#,
            r#"
            (
                1,
                1,
            )"#
        );
    }
}

#[test]
pub fn test_patch_ordering() {
    let mut patchwork = Patchwork::new("one two three".to_string());
    patchwork.patch_insert(13, "555", PatchOrdering::AfterOtherPatches);
    patchwork.patch_insert(13, "33", PatchOrdering::Normal);
    patchwork.patch_insert(13, "4", PatchOrdering::Normal);
    patchwork.patch_insert(13, "2", PatchOrdering::BeforeOtherPatches);
    expect!(patchwork.text(), r#""one two three2334555""#);
}
