# Fix: Export Status State and Live Region Reliability

## Scope
- Replaced string-only export status with explicit UI state: `Idle | Preparing | Success | Error`.
- Removed conditional mounting for status feedback and kept a persistent live region in the DOM.
- Prevented success styling during non-success states.
- Added unit tests for status transition behavior.

## Files Changed
- `src/routes/admin/components/export.rs`
- `style/main.scss`

## Behavior Notes
- Export actions now set a preparing message first, then move to success or error.
- Success class is only applied for `Success` state.
- Status container is always mounted with `role="status"`, `aria-live="polite"`, and `aria-atomic="true"`.
- Idle state keeps the status container visually neutral until an active message is present.

## Validation
- `cargo check --features ssr,hydrate`
- `cargo test export_status_transitions --features ssr,hydrate`
- `cargo clippy --all-targets --features ssr,hydrate -- -D warnings`
