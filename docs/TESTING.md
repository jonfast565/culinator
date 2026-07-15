# Testing convention

Every Rust source module has an adjacent `test.rs` module wired with:

```rust
#[cfg(test)]
mod test;
```

For file modules such as `routes/books.rs`, tests live at `routes/books/test.rs`. Crate roots use `src/test.rs`. Binary roots use an explicit `#[path = "test.rs"]` where necessary to avoid path collisions.

Run all checks with:

```bash
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```
