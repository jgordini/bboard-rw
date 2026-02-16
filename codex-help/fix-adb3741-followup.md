# Fix notes: adb3741 follow-up

## Scope
- Replaced reused comment failure text with operation-specific messages in `src/routes/article.rs`.
- Added helper-semantics unit tests for:
  - login-state helper behavior in `src/routes/view_helpers.rs`
  - reset-password action status mapping in `src/routes/reset_password.rs`

## Why
- Avoids misleading UX where comment get/delete errors reported as post failures.
- Reduces regression risk after helper extraction by pinning expected success/error/empty behavior.

## Verification
- `cargo check` ✅
- `cargo clippy --all-targets -- -D warnings` ✅
- `cargo test` ⚠️ fails due existing unrelated test: `profanity::tests::test_number_substitution`
- Targeted tests for changed helpers:
  - `cargo test routes::view_helpers::tests::` ✅
  - `cargo test routes::reset_password::tests::` ✅
