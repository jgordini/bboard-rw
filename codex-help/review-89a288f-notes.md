# Review Notes: 89a288f

## Summary
- Replaces Leptos server-function CSV export payloads with dedicated Axum GET download routes.
- Streams CSV rows with `Body::from_stream` using SQL cursor fetch.
- Adds CSV formula-injection mitigation in `csv_escape` and helper unit tests.
- Adds route wiring in `src/setup.rs` and simplifies export UI to direct URL-triggered downloads.

## Findings Draft
1. High security: `require_admin_cookie_jar` authorizes from raw `user_session` cookie JSON without integrity protection.
2. Medium regression: Export UI always reports success-started and removed failure/loading states, so auth/network/server errors are silent and duplicate clicks can trigger concurrent large exports.
3. Medium regression: Stream emits CSV header before first DB fetch; on stream/query errors the response remains `200` with partial/truncated CSV and no client-visible failure.
4. Low testing gap: Tests cover helper functions but not HTTP route behavior (401/403/header checks/stream success+error behavior).
