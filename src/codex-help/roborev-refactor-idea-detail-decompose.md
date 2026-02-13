# RoboRev Continue: Idea Detail Decomposition

Date: 2026-02-13

## Goal
Reduce hotspot complexity in `src/routes/idea_detail.rs` by breaking `IdeaDetailPage` into focused components.

## Changes
- Refactored `IdeaDetailPage` to delegate rendering to new components:
  - `IdeaDetailLoaded`
  - `IdeaDetailCard`
  - `CommentsSection`
  - `CommentItem`
- Kept behavior and server actions unchanged.
- Preserved existing UI flows (vote, edit idea, pin/lock, comment CRUD/moderation).

## Complexity Delta (function span/branch proxy)
- `IdeaDetailPage`
  - Before: ~507 lines, 35 branch keywords
  - After: ~64 lines, 4 branch keywords
- New split components:
  - `IdeaDetailCard`: 282 lines, 21 branch keywords
  - `CommentsSection`: 70 lines, 3 branch keywords
  - `CommentItem`: 125 lines, 6 branch keywords

## Result
- Primary hotspot is no longer concentrated in one function.
- Component boundaries now align with UI responsibilities.
- Next logical step: move extracted components into dedicated module files to reduce file-level size and improve navigability.
