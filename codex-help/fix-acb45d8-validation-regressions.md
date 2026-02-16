# Fix Notes: acb45d8 Validation Regression Follow-up

## Scope
Addressed three review findings after `acb45d8`:
- Hydrate-only build compatibility around shared validation helpers.
- Missing tests for extracted validation logic.
- Validation error precedence parity in moderator idea updates.

## Code Changes
- `src/routes/mod.rs`
  - Gated `validation_helpers` module behind `#[cfg(feature = "ssr")]`.
- `src/routes/validation_helpers.rs`
  - Removed hydrate fallback profanity stub.
  - Kept profanity checks server-only by relying on SSR-gated module inclusion.
  - Added focused unit tests for:
    - idea title/content empty and length boundaries
    - profanity rejection message path
    - tag length boundary
    - comment empty/length/profanity checks
- `src/routes/idea_detail.rs`
  - Restored previous validation precedence in `update_idea_content_mod` by checking tags before title/content.

## Verification Run
- `cargo check`
- `cargo check --no-default-features --features hydrate`
- `cargo test --lib validation_helpers`
- `cargo clippy --all-targets --all-features --no-deps`

## Notes
- `cargo fmt --all -- --check` reports pre-existing formatting diffs outside this change set.
