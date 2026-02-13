# RepoRev Fix: CAS Flow + Clippy Cleanup

Date: 2026-02-13

## CAS Implementation
- Added CAS redirect endpoint: `GET /auth/cas/login`
- Added CAS callback endpoint: `GET /auth/cas/callback?ticket=...`
- Added CAS ticket validation against configurable CAS serviceValidate endpoint.
- Added first-login user provisioning and normal session cookie issuance.
- Added login CTA: `"Login with BlazerID"`.

## Config + Docs
- Added CAS env vars to `.env.example`:
  - `CAS_LOGIN_URL`
  - `CAS_VALIDATE_URL`
  - `CAS_SERVICE_ID`
- Documented CAS endpoints and variables in `README.md`.

## Quality Fixes
- Replaced all `view! {}.into_any()` unit patterns with `().into_any()` in route/component files.
- Verified:
  - `cargo check`
  - `cargo clippy --all-targets -- -D warnings`
