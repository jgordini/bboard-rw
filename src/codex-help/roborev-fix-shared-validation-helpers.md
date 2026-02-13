# RoboRev Fix: Shared Validation Helpers

Date: 2026-02-13

## Objective
- Remove duplicated idea/comment validation logic across route server functions.

## Added
- `src/routes/validation_helpers.rs`
  - `validate_idea_title_and_content(title, content)`
  - `validate_idea_tags(tags)`
  - `validate_comment_content(content)`

## Updated Call Sites
- `src/routes/ideas.rs`
  - `create_idea_auth` now uses `validate_idea_title_and_content`.
- `src/routes/idea_detail.rs`
  - `create_comment` now uses `validate_comment_content`.
  - `update_comment_mod` now uses `validate_comment_content`.
  - `update_idea_content_mod` now uses `validate_idea_title_and_content` + `validate_idea_tags`.
- `src/routes/mod.rs`
  - Added `mod validation_helpers;`.

## Behavior
- Validation messages and limits were preserved.
- `cargo check` passed after the refactor.
