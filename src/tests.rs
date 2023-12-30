use crate::{
    expect,
    expect::{Expect, FilePosition},
    patchwork::{PatchOrdering, Patchwork},
    runtime::format_patch,
    str_lit_kind::StrLitKind,
};

#[test]
fn test_trivial_assert_empty_literal() {
    expect!("", "");
}

#[test]
fn test_trivial_assert_literal() {
    expect!("ABC", "ABC");
}

#[test]
fn test_trivial_assert_literal_multiline() {
    expect!(
        "ABC\nDEF",
        r#"
        ABC
        DEF"#
    );
}

#[test]
fn test_trivial_assert_literal_multiline2() {
    expect!(
        "ABC\nDEF\n",
        r#"
        ABC
        DEF
        "#
    );
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
    expect!(&patch, r##""r#\"\n    hello\n    world\n    \"#""##);
}

#[test]

fn test_format_patch_multi_line2() {
    let desired_indent = 4;
    let patch = "struct Test {\n    field_a: u32,\n    field_b: f64,\n    field_c: String,\n}\n";
    let patch = format_patch(desired_indent, patch);
    expect!(
        patch,
        r##""r#\"\n        struct Test {\n            field_a: u32,\n            field_b: f64,\n            field_c: String,\n        }\n        \"#""##
    );
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

#[test]
pub fn test_find_expect_location() {
    let expect = Expect {
        file_position: FilePosition {
            file: "src/tests2.rs",
            line: 7,
            column: 5,
        },
        raw_actual: "StrLitKind::from(\"\")",
        expected: ["ABC", "DEF"],
        raw_expected: ["\"ABC\"", "\"DEF\""],
        assertion_index: 0,
    };
    let file = "use crate::str_lit_kind::StrLitKind;\n\nuse super::*;\n\n#[test]\nfn test_lit_kind_for_patch_empty() {\n    expect!(StrLitKind::from(\"\"), \"ABC\", \"DEF\");\n}\n";
    let location = expect.find_expect_location(file);
    expect!(
        location,
        r#"
        ExpectLocation {
            line_indent: 4,
            expected_ranges: [
                132..137,
                139..144,
            ],
            start_index: 110,
            end_index: 144,
        }"#
    );
}
#[test]
pub fn test_find_expect_location_stringify() {
    let expect = Expect {
        file_position: FilePosition {
            file: "src/tests3.rs",
            line: 5,
            column: 5,
        },
        raw_actual: "stringify!(struct Test { test : u32, })",
        expected: ["test", "test2"],
        raw_expected: ["\"test\"", "\"test2\""],
        assertion_index: 0,
    };
    let file = "use super::*;\n\n#[test]\nfn test_stringify() {\n    expect!(\n        stringify!(\n            struct Test {\n                test: u32,\n            }\n        ),\n        \"test\",\n        \"test2\"\n    );\n}\n";
    let location = expect.find_expect_location(file);
    expect!(
        location,
        r#"
        ExpectLocation {
            line_indent: 4,
            expected_ranges: [
                164..170,
                180..187,
            ],
            start_index: 66,
            end_index: 187,
        }"#
    );
}

#[test]
pub fn test_fibonacci() {
    fn fibonacci(x: usize) -> usize {
        let mut z = vec![];
        z.push(0);
        z.push(1);
        for i in 2..=x {
            z.push(z[i - 1] + z[i - 2]);
        }
        z[x]
    }
    expect!(fibonacci(15), "610");
}

#[test]
pub fn test_callback_expect() {
    fn some_complicated_io_func(callback: impl Fn(i32)) {
        callback(5);
        callback(3);
        callback(10);
    }

    some_complicated_io_func(|status_value| {
        expect!(status_value, "5", "3", "10");
    })
}

#[test]
pub fn test_expect_macro_output() {
    macro_rules! test {
        () => {
            5
        };
    }
    expect!(test!(), "5");
}

#[test]
pub fn test_expect_macro_output2() {
    expect!(stringify!(1 + 1), r#""1 + 1""#);
}

#[test]
pub fn test_expect_multiline_macro() {
    expect!(
        stringify!(
            struct Test {
                field: u32,
            }
        ),
        r#""struct Test { field : u32, }""#
    );
}

#[test]
pub fn test_expect_multiline_no_macro() {
    expect!(
        {
            fn test() -> u32 {
                5
            }
            test()
        },
        "5"
    );
}

#[test]
pub fn test_expect_tuple() {
    expect!(
        (2, 3),
        r#"
        (
            2,
            3,
        )"#
    );
}

#[test]
pub fn test_expect_layered_tuple() {
    expect!(
        ((3, 4), 5),
        r#"
        (
            (
                3,
                4,
            ),
            5,
        )"#
    );
}
