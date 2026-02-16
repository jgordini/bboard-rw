# Review notes for c4a0706

## Finding
- `src/routes/article.rs` is not wired into the routes module (`src/routes/mod.rs` has no `mod article;`).
- Impact:
  - Runtime behavior changes in `src/routes/article.rs` (operation-specific comment errors) are not reachable.
  - New tests in `src/routes/article.rs` are not compiled/discovered by `cargo test`.

## Evidence
- `cargo test routes:: -- --list` shows tests for `reset_password`, `view_helpers`, and `validation_helpers`, but none for `routes::article::tests`.
- `rg -n "mod article|pub mod article" src/routes/mod.rs` finds no article module declaration.

## Suggested fix
- If this route is intended to be active, add `mod article;` and route export/wiring needed by app router.
- If article route is intentionally deprecated, remove stale `src/routes/article.rs` and avoid committing behavior/test updates there.
