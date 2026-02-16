# Fix notes: b0194d2 follow-up

## Scope
Addressed review findings after `b0194d2`:
- Added tests for shared server-function error helpers.
- Reduced hydrate warning noise by removing top-level server-only imports in route files.

## Code changes
- `src/routes/error_helpers.rs`
  - Added unit tests covering:
    - `server_fn_error_with_log` message/variant behavior
    - `server_fn_server_error_with_log` message/variant behavior
    - representative route-style `map_err(...)` client-facing message preservation
- `src/routes/ideas.rs`
  - Replaced top-level helper imports with fully qualified helper/validation references inside `#[server]` function bodies.
- `src/routes/idea_detail.rs`
  - Replaced top-level helper imports with fully qualified helper/validation references inside `#[server]` function bodies.

## Verification
- `cargo check`
- `cargo check --no-default-features --features hydrate`
- `cargo clippy --all-targets --all-features --no-deps -- -D warnings`
- `cargo test routes::error_helpers::tests::`
