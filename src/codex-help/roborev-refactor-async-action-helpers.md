# RoboRev Refactor: Async Action Helpers

Date: 2026-02-13

## Goal
- Reduce repeated `spawn_local + server action + Result handling + refetch` logic across recently split route components.

## Added
- `src/routes/async_helpers.rs`
  - `spawn_server_action`: generic async action runner with explicit success/error callbacks.
  - `spawn_server_action_ok`: success-only wrapper.
  - `spawn_server_action_refetch`: success-only wrapper specialized for refetch patterns.

## Refactored Call Sites
- `src/routes/ideas/components/submission.rs`
- `src/routes/ideas/components/card.rs`
- `src/routes/idea_detail/components/card.rs`
- `src/routes/idea_detail/components/comments.rs`
- `src/routes/admin/components/flags.rs`
- `src/routes/admin/components/moderation.rs`
- `src/routes/admin/components/users.rs`

## Behavioral Notes
- No intended behavior changes.
- Existing side effects (refetch, error display, submit/loading toggles, edit mode transitions) were preserved.
- `cargo check` passed after refactor.
