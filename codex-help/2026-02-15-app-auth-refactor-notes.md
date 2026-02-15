# App/Auth Refactor Notes (2026-02-15)

## Scope
- Refactored `src/app.rs` to isolate layout concerns (`GlobalStyles`, `AppRoutes`, `AppFooter`).
- Extracted auth nav-state mapping and auth-nav rendering helpers.
- Separated logout side effect from nav rendering.
- Added auth refresh helper API in `src/auth.rs` to avoid tuple-field coupling.
- Added centralized route path constants in `src/routes/paths.rs` and used router `<A>` links for internal navigation where touched.
- Removed thin login/signup route wrappers by moving signal setup into `Login` and `Signup` components.

## Verification
- `cargo fmt`
- `cargo check` (pass)
- `cargo clippy -- -D warnings` (pass)
- `cargo test` (fails in existing test: `profanity::tests::test_number_substitution` at `src/profanity.rs:84`)
