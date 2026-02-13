# RepoRev Fix: Async Refetch + Confirm Dedupe

Date: 2026-02-13

## What Was Fixed
1. Added shared async helpers in `src/routes/async_helpers.rs`:
- `spawn_server_action_refetch_resource(future, resource)`
- `spawn_server_action_with_error(future, on_ok, error_signal)`

2. Added shared confirm helper in `src/routes/view_helpers.rs`:
- `confirm_action(message: &str) -> bool`

3. Replaced duplicated call-site patterns:
- `src/routes/idea_detail/components/card.rs`
- `src/routes/idea_detail/components/comments.rs`
- `src/routes/admin/components/flags.rs`
- `src/routes/admin/components/moderation.rs`
- `src/routes/admin/components/users.rs`

## Duplication Eliminated
- Repeated `spawn_server_action_refetch(..., move || resource.refetch())` closures
- Repeated edit-form `spawn_server_action(..., on_ok, set_error)` error plumbing
- Repeated destructive confirm dialog boilerplate with `window().confirm_with_message(...).unwrap_or(false)`

## Verification
- `cargo check` passes after refactor.
