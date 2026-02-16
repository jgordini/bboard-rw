# Fix: Admin CSV Export Hardening and Streaming

## Scope
- Replace admin CSV export server-function payload responses with authenticated SSR download endpoints.
- Stream CSV rows from SQL query cursors instead of `fetch_all()` + giant in-memory string assembly.
- Harden CSV cell encoding to mitigate spreadsheet formula injection (`=`, `+`, `-`, `@` after leading spaces/tabs).
- Stop surfacing raw backend export errors through export UI status text.

## Files Changed
- `src/routes/admin.rs`
- `src/routes/admin/components/export.rs`
- `src/setup.rs`
- `Cargo.toml`
- `Cargo.lock`

## Security/Regression Notes
- Export routes now require an admin session cookie and return:
  - `401` when unauthenticated/invalid session
  - `403` when authenticated but not admin
- CSV responses are delivered with attachment headers and `no-store` cache control.
- Stream failures are logged server-side and not exposed as internal error detail in UI.

## Validation
- `cargo fmt --all`
- `cargo check --features ssr,hydrate`
- `cargo clippy --all-targets --features ssr,hydrate -- -D warnings`
- `cargo test routes::admin::tests --features ssr,hydrate`
- `cargo test --features ssr,hydrate` (fails in pre-existing `profanity::tests::test_number_substitution`)
