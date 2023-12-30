use crate::expect_tokens;
use quote::quote;

#[test]
fn test_empty() {
    expect_tokens!(quote! {}, "");
}

#[test]
fn test_function() {
    expect_tokens!(
        quote! {fn hello_world() {
            println!("Hello World!");
        }},
        r#"
        fn hello_world() {
            println!("Hello World!");
        }
        "#
    );
}

#[test]
fn test_struct() {
    expect_tokens!(
        quote! {
            struct Test {
                field_a: u32,
                field_b: f64,
                field_c: String,
            }
        },
        r#"
        struct Test {
            field_a: u32,
            field_b: f64,
            field_c: String,
        }
        "#
    );
}
