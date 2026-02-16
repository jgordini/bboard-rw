# Fix: Auth Nav Refresh Coverage

Date: 2026-02-16

## Requested change
- Add focused test coverage for auth refresh behavior in navigation after signup/login/logout.

## Implementation
- Added Playwright e2e test `nav refreshes after signup, logout, and login`.
- Test flow:
  - signs up a fresh user
  - verifies `Logout` appears in nav
  - logs out and verifies `Login` appears in nav
  - logs back in with the same credentials
  - verifies `Logout` appears again in nav

## Files changed
- `end2end/tests/example.spec.ts`
- `codex-help/fix-auth-nav-refresh-e2e.md`

## Validation
- `cargo check --features ssr,hydrate`: passed
- `cargo clippy --all-targets --all-features -- -D warnings`: passed
- `cargo test`: failed in existing unrelated test `profanity::tests::test_number_substitution` (`src/profanity.rs:84`)
- `npx playwright test tests/example.spec.ts --list` (in `end2end/`): passed (new test discovered)
- `npx playwright test tests/example.spec.ts -g "nav refreshes after signup, logout, and login" --project=chromium`: blocked (Playwright browser executable not installed)
