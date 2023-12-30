## Expect Tests

Expect Tests is a Rust crate inspired by [rust-analyzer's expect-test](https://github.com/rust-analyzer/expect-test). Explanation can be found [here](https://blog.janestreet.com/the-joy-of-expect-tests/).

### Basic Usage:

```rust
expect!(fibonacci(15), "610");
```

The macro uses the Debug trait representation (except for string literals) and compares it to the provided string literal. In case of a mismatch, an error with a diff will be shown:

```plaintext
You can update all `expect!` tests by running:
    UPDATE_EXPECT=1 cargo test
To update a single test, place the cursor on `expect` token and use the `run` feature of rust-analyzer.

Expect:
----
987
----

Actual:
----
610
----

Diff:
----
987610
----
```

If the `UPDATE_EXPECT` environment variable is set, the macro will directly update the source file to fix it. For example:

```rust
expect!(fibonacci(15));
```

After running `UPDATE_EXPECT=1 cargo test`, your source file should automatically be updated to:

```rust
expect!(fibonacci(15), "610");
```

### Multiple Expects:

Testing callbacks can be cumbersome because you'd need to collect into a vec. Here's an alternative using `expect!`:

```rust
fn some_complicated_io_func(callback: impl Fn(i32)) {
  callback(5);
  callback(3);
  callback(10);
}

some_complicated_io_func(|status_value| {
  expect!(status_value, "5", "3", "10");
});
```

**Caveat:** Expect-tests cannot detect when the `expect!` macro is never called. Therefore, a loop like the following will pass even though "2", "3", and "4" are never run:

```rust
for i in 0..2 {
  expect!(i, "0", "1", "2", "3", "4");
}
```

### Expect Tokens:

Testing proc macros is now easier with expect tokens:

```rust
#[test]
fn test_enum() {
    let output = quote! {
        enum TrafficLight {
            Red,
            Yellow,
            Green
        }
    };
    expect_tokens!(
        output,
        r#"
        enum TrafficLight {
            Red,
            Yellow,
            Green,
        }
        "#
    );
}
```

### Examples:

Check out the [examples](https://github.com/awesomelemonade/expect-tests/blob/master/src/tests.rs) used to test this crate.

### Alternatives:

- [expect-test](https://docs.rs/expect-test/latest/expect_test/)
- [insta](https://crates.io/crates/insta)
- [k9](https://crates.io/crates/k9)

### Other Notes:

- Makes a "best effort" to comply with rustfmt. When the macro detects a multiline expect, updating will attempt to insert newlines where appropriate.
- Indentations are ignored in the string literals when comparing to make the code look nicer.
