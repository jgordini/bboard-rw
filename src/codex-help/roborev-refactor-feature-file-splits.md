# RoboRev Continue: Feature File Splits

Date: 2026-02-13

## Goal
Continue decomposition by splitting large component files into feature-focused files under each route area.

## Changes

### `idea_detail` component files
- Kept module entry: `src/routes/idea_detail/components.rs`
- Added:
  - `src/routes/idea_detail/components/card.rs`
  - `src/routes/idea_detail/components/comments.rs`
- `components.rs` now orchestrates `IdeaDetailLoaded` and delegates card/comments rendering.

### `admin` component files
- Kept module entry: `src/routes/admin/components.rs` with `AdminDashboard`
- Added tab files:
  - `src/routes/admin/components/overview.rs`
  - `src/routes/admin/components/flags.rs`
  - `src/routes/admin/components/moderation.rs`
  - `src/routes/admin/components/users.rs`

### `ideas` component files
- Kept module entry: `src/routes/ideas/components.rs`
- Added:
  - `src/routes/ideas/components/board.rs`
  - `src/routes/ideas/components/submission.rs`
  - `src/routes/ideas/components/card.rs`
- Fixed visibility during split by making `IdeasBoard` public in submodule for re-export.

## Validation
- `SQLX_OFFLINE=true cargo check --manifest-path /Users/jeremy/repos/bboard-rw/Cargo.toml` passes.

## Outcome
- Large UI files are now decomposed into route-local feature files.
- Parent route files stay focused on server actions and entry composition.
- Navigation and review granularity improved materially.
