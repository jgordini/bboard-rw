## Change

Implemented true toggle behavior for the idea detail flag button (`btn-flag btn btn-secondary`).

## Behavior

- Button now toggles between `Flag` and `Unflag`.
- Clicking performs server-side toggle for the current user flag on that idea.
- Initial state is loaded from server so existing flags show `Unflag` immediately.

## Files

- `src/routes/ideas.rs`
  - Added `check_idea_flag_server(idea_id) -> Result<bool, ServerFnError>`
  - Added `toggle_idea_flag_server(idea_id) -> Result<bool, ServerFnError>`
- `src/routes/idea_detail/components/card.rs`
  - Loads initial flag state via `check_idea_flag_server`
  - Uses `toggle_idea_flag_server` on click
  - Removes one-way disabled behavior and updates label to `Flag`/`Unflag`
- `src/models/flag.rs`
  - Added `user_has_flag(...)`
  - Added `toggle_user_flag(...) -> Result<bool, sqlx::Error>`
  - Removed unused one-way `create(...)`

## Verification

- `cargo fmt`
- `cargo check --all-targets --all-features`
