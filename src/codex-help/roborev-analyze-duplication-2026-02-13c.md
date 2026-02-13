# RoboRev Duplication Analysis (Current Working Tree)

Date: 2026-02-13

## Completed (from prior 6-item list)
1. Idea input validation shared helper
- `src/routes/ideas.rs:20`
- `src/routes/idea_detail.rs:59`
- centralized in `src/routes/validation_helpers.rs:3`

2. Comment validation shared helper
- `src/routes/idea_detail.rs:48`
- `src/routes/idea_detail.rs:78`
- centralized in `src/routes/validation_helpers.rs:36`

3. Reset-password action status extraction shared helper
- `src/routes/reset_password.rs:239`
- `src/routes/reset_password.rs:269`
- centralized in `src/routes/reset_password.rs:159`

4. Logged-in resource unwrap helper for ideas components
- `src/routes/ideas/components/submission.rs:35`
- `src/routes/ideas/components/card.rs:31`
- centralized in `src/routes/view_helpers.rs:32`

5. Admin users error/alert callback deduped
- shared `show_admin_error` in `src/routes/admin/components/users.rs:9`

6. Article comment server-action error mapping deduped
- shared `comment_server_error` in `src/routes/article.rs:13`
- used by `src/routes/article.rs:123`, `src/routes/article.rs:131`, `src/routes/article.rs:144`

## Remaining High-Value Duplication
1. Repeated async action + `resource.refetch()` callback patterns
- `src/routes/idea_detail/components/card.rs:266`
- `src/routes/idea_detail/components/card.rs:279`
- `src/routes/idea_detail/components/comments.rs:167`
- `src/routes/idea_detail/components/comments.rs:189`
- `src/routes/admin/components/flags.rs:16`
- `src/routes/admin/components/moderation.rs:14`

2. Repeated async action + local error signal plumbing in editable forms
- `src/routes/idea_detail/components/card.rs:91`
- `src/routes/idea_detail/components/comments.rs:117`

3. Repeated destructive confirm dialog pattern (`window().confirm_with_message(...).unwrap_or(false)`)
- `src/routes/admin/components/users.rs:98`
- `src/routes/admin/components/flags.rs:82`
- `src/routes/admin/components/moderation.rs:57`

## Recommendation
- Next fix pass should extract small UI helpers for:
  - `confirm_then(action)`
  - `spawn_with_refetch(resource, future)`
  - `spawn_edit_action(future, set_error, on_success)`
