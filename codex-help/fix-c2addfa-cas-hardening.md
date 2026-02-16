# Fix Notes: c2addfa CAS Hardening

Date: 2026-02-16

## Goals Addressed
- Prevent unsafe CAS-to-local account auto-linking by email.
- Enforce stricter CAS ticket validation semantics.
- Add timeout bounds to CAS HTTP calls.
- Prevent stale `cas_error` query params from masking local login action errors.
- Add focused tests for CAS parsing and login error precedence.

## Code Changes
- Added CAS subject persistence:
  - New migration: `migrations/20260216000000_add_cas_subject_to_users.up.sql`
  - Rollback migration: `migrations/20260216000000_add_cas_subject_to_users.down.sql`
  - Adds `users.cas_subject` and unique constraint `users_cas_subject_unique`.
- Added `User` model methods:
  - `get_by_cas_subject`
  - `create_cas_user`
- Hardened CAS auth flow in `src/auth.rs`:
  - CAS validation now uses `reqwest::Client::builder()` with timeout and connect timeout.
  - Requires HTTP success status from CAS validation endpoint.
  - Replaced regex parsing with XML parsing via `roxmltree`.
  - Requires `<authenticationSuccess>` and rejects `<authenticationFailure>`.
  - Identity mapping now keys on stable `cas_subject` rather than email.
  - If email already belongs to an unlinked local account, login fails with `cas_error=link_required` (no auto-link).
- Updated login UI behavior in `src/routes/login.rs`:
  - Added `link_required` message.
  - Local login action errors now take precedence over query-string CAS errors.
- Added env template support:
  - `.env.example`: `CAS_HTTP_TIMEOUT_SECS`.

## Test Coverage Added
- `src/auth.rs`:
  - CAS success XML parse.
  - Missing `authenticationSuccess` rejection.
  - `authenticationFailure` rejection.
  - Missing mail fallback to `@uab.edu`.
- `src/routes/login.rs`:
  - `link_required` message mapping.
  - Action-error precedence over CAS query error.

## Verification Run
- `cargo fmt` ✅
- `cargo check` ✅
- `cargo clippy --all-targets --all-features` ✅
- `cargo test auth::tests:: -- --nocapture` ✅
- `cargo test routes::login::tests:: -- --nocapture` ✅
- `cargo test` ⚠️ fails on pre-existing unrelated test:
  - `profanity::tests::test_number_substitution` (`src/profanity.rs`)
