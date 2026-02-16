# Validation Helpers Regression Fix Notes (2026-02-15)

## Scope
Address findings from commit `acb45d8` review:
1. Hydrate build regression from shared validation helper linkage.
2. Missing tests for extracted validation logic.
3. Validation order regression in moderator update flow.

## Changes
- Gated `routes::validation_helpers` behind `ssr` in `src/routes/mod.rs` so hydrate-only builds do not compile server-only validation/profanity code.
- Restored validation order in `update_idea_content_mod` to check tags before title/content in `src/routes/idea_detail.rs`.
- Added unit tests in `src/routes/validation_helpers.rs` for:
  - Empty and max-length checks for title/content/comments.
  - Tag length boundary.
  - Profanity rejection paths.

## Verification Commands
- `cargo fmt --all`
- `cargo check --no-default-features --features hydrate`
- `cargo check`
- `cargo test validation_helpers`

## Notes
These updates keep profanity checks server-only while preserving prior user-visible error precedence in moderator edits.
