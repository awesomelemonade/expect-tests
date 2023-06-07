Expect Tests
---

Based on https://github.com/rust-analyzer/expect-test
Explanation: https://blog.janestreet.com/the-joy-of-expect-tests/

---

Basic Usage:

```
expect!(fibonacci(15), "610");
```

The macro will use the Debug trait representation (except for string literals) and compare it to the string literal.

If there is a mismatch such as `expect!(fibonacci(15), "987");`, an error with the diff will be shown:

```
You can update all `expect!` tests by running:
    UPDATE_EXPECT=1 cargo test
To update a single test, place the cursor on `expect` token and use `run` feature of rust-analyzer.

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

---

If `UPDATE_EXPECT` environment variable is set, the macro will directly update the source file to fix it. Here's an example:

```
expect!(fibonacci(15));
```

After running `UPDATE_EXPECT=1 cargo test`, your source file should automatically be updated to

```
expect!(fibonacci(15), "610");
```

---


Multiple expects:


Maybe you want to test a callback, but for some reason (maybe due to laziness) it is too cumbersome to collect it into a vec before expect!ing it. We can do something like this:

```
fn some_complicated_io_func(callback: impl Fn(i32)) {
  callback(5);
  callback(3);
  callback(10);
}

some_complicated_io_func(|status_value| {
  expect!(status_value, "5", "3", "10");
})

```

Caveat: expect-tests cannot detect when the expect! macro is never called. Therefore, something like

```
for i in 0..2 {
  expect!(i, "0", "1", "2", "3", "4");
}
```

will pass even though "2", "3", and "4" are never run.

---

Examples (that are used to test this crate!):
https://github.com/awesomelemonade/expect-tests/blob/master/src/tests.rs

---

Alternatives:

expect-test: https://docs.rs/expect-test/latest/expect_test/
insta: https://crates.io/crates/insta
k9: https://crates.io/crates/k9

---

Other notes:
 - Does a "best effort" to comply with rustfmt. When the macro detects a multiline expect, updating will attempt to insert newlines where appropriate
 - Indentations are ignored in the string literals when comparing. This is so it is prettier in the code.
