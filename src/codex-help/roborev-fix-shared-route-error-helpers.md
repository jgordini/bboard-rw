# RoboRev Fix: Shared Route Error Helpers

Date: 2026-02-13

## Objective
- Reduce duplicated `map_err` logging/response closures across route server functions.

## Added
- `src/routes/error_helpers.rs`
  - `server_fn_error_with_log(context, error, user_message)`
  - `server_fn_server_error_with_log(context, error, user_message)` (kept for parity where `ServerError` is required)

## Updated
- `src/routes/mod.rs`
  - Added `mod error_helpers;`
- `src/routes/ideas.rs`
  - Replaced repeated logging + `ServerFnError::new(...)` closures with `server_fn_error_with_log`.
- `src/routes/idea_detail.rs`
  - Replaced repeated logging + `ServerFnError::new(...)` closures with `server_fn_error_with_log`.
- `src/routes/home.rs`
  - Replaced local error logging closures with shared helpers.
- `src/routes/profile.rs`
  - Replaced local error logging closures with shared helpers.

## Validation
- `SQLX_OFFLINE=true cargo check --manifest-path /Users/jeremy/repos/bboard-rw/Cargo.toml` passed.
