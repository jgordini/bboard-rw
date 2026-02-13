# RoboRev Continue: Idea Detail Module Split

Date: 2026-02-13

## Goal
Move extracted `idea_detail` UI components into a dedicated module file for better navigation and ownership boundaries.

## Changes
- In `src/routes/idea_detail.rs`:
  - Added `mod components;`
  - Imported `IdeaDetailLoaded` from the submodule.
  - Removed in-file definitions of:
    - `IdeaDetailLoaded`
    - `IdeaDetailCard`
    - `CommentsSection`
    - `CommentItem`
    - `CommentForm`
- Added new file `src/routes/idea_detail/components.rs` containing those component implementations.

## Outcome
- `src/routes/idea_detail.rs` is now focused on server functions + route entry component.
- UI rendering complexity for idea detail is isolated to `src/routes/idea_detail/components.rs`.
- Compile check passes after the split.
