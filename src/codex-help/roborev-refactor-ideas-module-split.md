# RoboRev Continue: Ideas Module Split

Date: 2026-02-13

## Goal
Split `routes/ideas.rs` UI-heavy components into a dedicated submodule and keep the parent focused on server/actions + shared sorting helpers.

## Changes
- In `src/routes/ideas.rs`:
  - Added `mod components;`
  - Imported `IdeasBoard` from the submodule.
  - Reduced `IdeasPage` to a thin entry wrapper.
  - Kept server functions and sorting helpers in parent module.
  - Marked `SortMode` and `sort_ideas` as `pub(super)` for submodule reuse.
- Added `src/routes/ideas/components.rs` containing:
  - `IdeasBoard`
  - `IdeaSubmissionDialog`
  - `IdeaCard`

## Outcome
- `ideas.rs` now has clearer responsibility boundaries.
- UI rendering complexity for the ideas board is isolated to `ideas/components.rs`.
- Compile check passes after module split.
