# RoboRev Complexity Analysis

Date: 2026-02-13

## Method
- Static complexity proxy audit across `src/**/*.rs`.
- Metrics used:
  - Function span (lines from function start to next function start).
  - Branch count proxy (`if`, `else if`, `match`, `for`, `while`, `loop`) per function.
  - File-level line/function/branch totals.

## File-Level Hotspots
- `src/routes/idea_detail.rs`: 732 lines, 10 functions, 54 branch keywords.
- `src/routes/admin.rs`: 638 lines, 18 functions, 36 branch keywords.
- `src/routes/ideas.rs`: 606 lines, 12 functions, 30 branch keywords.
- `src/routes/reset_password.rs`: 305 lines, 10 functions, 19 branch keywords.

## Function-Level Hotspots (Top by branch count + size)
- `src/routes/idea_detail.rs:174` `IdeaDetailPage`: 507 lines, 35 branches.
- `src/routes/admin.rs:511` `UsersTab`: 121 lines, 10 branches.
- `src/routes/admin.rs:335` `FlagsTab`: 103 lines, 9 branches.
- `src/routes/reset_password.rs:94` `reset_password_1`: 61 lines, 9 branches.
- `src/routes/ideas.rs:127` `IdeasPage`: 195 lines, 8 branches.
- `src/routes/ideas.rs:322` `IdeaSubmissionDialog`: 168 lines, 8 branches.
- `src/routes/ideas.rs:490` `IdeaCard`: 100 lines, 8 branches.

## Structural Drivers of Complexity
- Route modules combine server actions, domain validation, UI rendering, and side-effects in one file.
- Large `view!` trees with nested `Suspense`/`Show`/`For` blocks increase cognitive load.
- Repeated async mutation patterns (`spawn_local` + action + `refetch`) are duplicated across tabs/cards.
- Per-item inline edit state in comment/user rows creates deeply nested closure scopes.

## Primary Risks
- High regression risk in feature edits touching `IdeaDetailPage` or admin tabs.
- Slower onboarding and code review due large mixed-concern functions.
- Reduced testability from UI/business logic coupling.

## Refactor Priorities
1. Decompose `IdeaDetailPage` into focused components:
   - `IdeaHeaderCard`
   - `IdeaMetaActions`
   - `CommentsPanel`
   - `CommentItem`
2. Split admin tabs into separate modules (`admin/flags.rs`, `admin/users.rs`, `admin/moderation.rs`).
3. Introduce shared async action helper for `spawn_local + Result handling + refetch` patterns.
4. Move input validation into reusable service helpers (idea/comment/reset flows).
5. Add focused tests for extracted validation helpers and action adapters.

## Practical Target
- Keep route component functions under ~120 lines where possible.
- Keep branch proxy under ~10 for top-level UI functions.
