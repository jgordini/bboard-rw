# RoboRev Analyze Refactor Notes

Date: 2026-02-13

## Scope
- Analyze `src/routes` for refactor candidates.
- Implement a low-risk consolidation refactor.

## Findings
- `format_relative_time` duplicated in:
  - `src/routes/ideas.rs`
  - `src/routes/idea_detail.rs`
- `stage_badge_color` duplicated in:
  - `src/routes/ideas.rs`
  - `src/routes/idea_detail.rs`

## Refactor Applied
- Added `src/routes/view_helpers.rs` with shared functions:
  - `format_relative_time`
  - `stage_badge_color`
- Updated both route modules to import shared helpers.
- Removed duplicated local helper implementations.

## Why this helps
- Single source of truth for display formatting.
- Reduces drift risk when labels/timing logic evolve.
- Shrinks route files and improves maintainability.
