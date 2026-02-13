# RoboRev Duplication Analysis (Post Validation Refactor)

Date: 2026-02-13

## Scope
- Re-ran duplication scan across `src/routes/**/*.rs` after introducing `validation_helpers`.
- Focused on non-markup logic duplication.

## Completed Duplications
1. Idea validation shared
- `src/routes/ideas.rs:19`
- `src/routes/idea_detail.rs:67`
- Now delegated to `src/routes/validation_helpers.rs:3` (`validate_idea_title_and_content`).

2. Comment validation shared
- `src/routes/idea_detail.rs:53`
- `src/routes/idea_detail.rs:89`
- Now delegated to `src/routes/validation_helpers.rs:36` (`validate_comment_content`).

## Remaining High-Value Duplications
1. Reset-password UI error extraction closure
- `src/routes/reset_password.rs:225`
- `src/routes/reset_password.rs:262`
- Same `result_of_call.with(...).unwrap_or_default()` flow; only tracing message differs.

2. Logged-in unwrapping closure in ideas components
- `src/routes/ideas/components/card.rs:31`
- `src/routes/ideas/components/submission.rs:34`
- Same `Resource<Result<Option<UserSession>, ServerFnError>>` unwrap logic.

3. Admin users error/alert callback duplication
- `src/routes/admin/components/users.rs:17`
- `src/routes/admin/components/users.rs:29`
- Same `window().alert_with_message(&e.to_string())` path in two handlers.

4. Repeated comment action `map_err` shape in article route
- `src/routes/article.rs:117`
- `src/routes/article.rs:127`
- `src/routes/article.rs:144`
- Shared logging/response pattern; message text is reused verbatim.

## Secondary Duplication Signals
1. Cross-file profile/article error mapping pattern
- `src/routes/article.rs:18`
- `src/routes/profile.rs:15`

2. Repeated auth+vote server boilerplate in ideas route
- `src/routes/ideas.rs:39`
- `src/routes/ideas.rs:53`

## Priority Recommendation
1. Extract a small reset/action-error helper for `ServerAction::value()` rendering in reset-password.
2. Extract `is_logged_in` helper for ideas component resource unwrap.
3. Extract admin users alert-on-error callback helper (or shared function variable).
4. Add a tiny route-level `map_err` helper in article route for comment actions.
