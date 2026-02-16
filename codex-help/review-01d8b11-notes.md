# Review Notes: 01d8b11

## Scope
- `src/routes/admin/components/export.rs`
- `style/main.scss`

## Checks
- Reviewed diff and surrounding context with line numbers.
- Ran: `cargo test export_status_transitions --features ssr,hydrate` (pass).

## Findings
1. Browser path still reports success regardless of HTTP/auth/download failure because `trigger_csv_download` always returns `Ok(())` in hydrate builds.
2. Preparing state is set and immediately overwritten synchronously; likely not rendered/announced, undermining intended explicit transition UX.
3. Tests validate helper string/state constructors only; no component-level coverage for class toggling or live-region update behavior.
