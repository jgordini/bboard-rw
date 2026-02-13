# RoboRev Duplication Analysis

Date: 2026-02-13

## Scope
- Audited `src/routes/**/*.rs` for repeated non-markup logic after recent component/module splits.
- Methods:
  - Pattern scan (`Resource::new`, async action handlers, `map_err` closures).
  - Normalized 4-line duplicate block detection across route files.

## Top Duplication Findings (Ranked)
1. Idea validation logic duplicated across create/edit paths
- `src/routes/ideas.rs:18`
- `src/routes/idea_detail.rs:72`
- Same title/content/profanity checks are implemented twice with near-identical strings and bounds.
- Risk: future rule changes can diverge between create and moderator-edit flows.

2. Comment validation logic duplicated inside detail route server actions
- `src/routes/idea_detail.rs:50`
- `src/routes/idea_detail.rs:110`
- Same empty/length/profanity checks are duplicated for `create_comment` and `update_comment_mod`.
- Risk: policy drift and inconsistent moderation behavior.

3. Reset-password UI error-extraction closure duplicated
- `src/routes/reset_password.rs:225`
- `src/routes/reset_password.rs:262`
- `AskForEmail` and `ConfirmPassword` share the same `result_of_call.with(...).unwrap_or_default()` error pattern.
- Risk: repeated error formatting logic and repeated tracing blocks.

4. Logged-in status closure duplicated in ideas components
- `src/routes/ideas/components/submission.rs:34`
- `src/routes/ideas/components/card.rs:31`
- Same `Resource<Result<Option<UserSession>, ServerFnError>>` unwrapping pattern.
- Risk: readability/maintenance overhead in component logic.

5. Repeated async error/alert callback in admin users tab
- `src/routes/admin/components/users.rs:16`
- `src/routes/admin/components/users.rs:28`
- Same `window().alert_with_message(&e.to_string())` error path duplicated across role change and delete.
- Risk: local duplication and harder extension for richer admin feedback.

6. Repeated `map_err` logging shape in article comment server actions
- `src/routes/article.rs:117`
- `src/routes/article.rs:127`
- `src/routes/article.rs:144`
- Identical logging+error response structure repeated; message text currently says "posting a comment" even in non-post contexts.
- Risk: inconsistent diagnostics and copy/paste message drift.

## Objective Signal (From Normalized 4-Line Duplicate Blocks)
- Duplicate clusters of non-markup logic were detected across files in the areas above.
- Highest-value cross-file matches were:
  - idea validation chain (`ideas.rs` and `idea_detail.rs`)
  - login-state closure (`ideas/components/*`)
  - reset-password error closure (`reset_password.rs`)

## Recommended Extraction Order
1. Add shared validators module for idea/comment input checks
- Candidate: `src/routes/validation_helpers.rs` or `src/models/validation.rs`.
- Helpers:
  - `validate_idea_input(title: &str, content: &str, tags: Option<&str>)`
  - `validate_comment_input(content: &str)`

2. Add reusable action-result message helper for `ServerAction::value()` rendering
- Candidate helper to collapse duplicated `result_of_call.with(...)` UI closure in reset flow.

3. Add a tiny auth-view helper for user-resource checks
- Example: `is_user_logged_in(&Resource<Result<Option<UserSession>, ServerFnError>>) -> bool`.

4. Consolidate repeated `map_err` logger wrappers for comment actions
- Unify error text and avoid copy/paste message mistakes.

## Est. Impact
- Removes duplicated validation branches in high-traffic server actions.
- Lowers regression risk when changing profanity/length/auth messaging rules.
- Keeps split component structure while reducing closure-level repetition.
