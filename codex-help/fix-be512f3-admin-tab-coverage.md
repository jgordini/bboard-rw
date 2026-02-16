# Fix: Admin Export Review Follow-up (be512f3)

## Scope
- Added explicit, testable tab-resolution logic for the admin dashboard.
- Added unit tests to cover admin-only tab visibility and routing guards.
- Kept export status behavior hardened with explicit states and generic client-safe errors.

## Files Changed
- `src/routes/admin/components.rs`
- `codex-help/fix-be512f3-admin-tab-coverage.md`

## Behavior Notes
- Admin-only tabs (`export`, `users`) remain visible and reachable only for admin users.
- Non-admin users are routed to the unknown-tab fallback if those tab keys are selected.
- Shared tabs (`overview`, `flags`, `moderation`) remain available to all roles.

## Validation
- `cargo check --features ssr,hydrate`
- `cargo clippy --features ssr,hydrate --no-deps`
- `cargo test routes::admin::components::tests --features ssr,hydrate`
- `cargo test export_status_transitions --features ssr,hydrate`
