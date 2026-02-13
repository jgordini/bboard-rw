# RoboRev Fix: UI/Action Duplication Helpers

Date: 2026-02-13

## Objective
- Remove remaining high-value duplication in reset-password UI state handling, ideas login checks, admin user error callbacks, and article comment error mapping.

## Changes
1. Shared login status helper
- Updated `src/routes/view_helpers.rs`:
  - Added `is_user_logged_in(user_resource: &Resource<Result<Option<UserSession>, ServerFnError>>) -> bool`.
- Updated call sites:
  - `src/routes/ideas/components/submission.rs`
  - `src/routes/ideas/components/card.rs`

2. Shared reset-password action status extraction
- Updated `src/routes/reset_password.rs`:
  - Added `action_status_message(...)` helper.
  - Replaced duplicated `result_of_call.with(...)` error/status closures in:
    - `AskForEmail`
    - `ConfirmPassword`

3. Shared admin users alert handler
- Updated `src/routes/admin/components/users.rs`:
  - Added `show_admin_error(ServerFnError)`.
  - Reused it for both role update and user delete handlers.

4. Shared article comment server error mapping
- Updated `src/routes/article.rs`:
  - Added `comment_server_error(operation, error)`.
  - Reused in:
    - `post_comment`
    - `get_comments`
    - `delete_comment`

## Validation
- `SQLX_OFFLINE=true cargo check --manifest-path /Users/jeremy/repos/bboard-rw/Cargo.toml` passed.
