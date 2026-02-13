# CAS Flow Implementation (UAB Padlock)

Date: 2026-02-13

## Implemented Endpoints
- `GET /auth/cas/login`
  - Redirects to `CAS_LOGIN_URL` with `service=CAS_SERVICE_ID`.
- `GET /auth/cas/callback?ticket=...`
  - Validates ticket against `CAS_VALIDATE_URL`.
  - Extracts CAS attributes (`user`, `mail/email`, `displayName/cn/name`).
  - Finds or creates local user by email.
  - Creates `user_session` cookie and redirects to `/`.

## Environment Variables
- `CAS_LOGIN_URL` (default: `https://padlock.idm.uab.edu/cas/login`)
- `CAS_VALIDATE_URL` (default: `https://padlock.idm.uab.edu/cas/serviceValidate`)
- `CAS_SERVICE_ID` (default: `http://localhost:3000/auth/cas/callback`)

Production expected value for `CAS_SERVICE_ID`:
- `https://uabspark.com/auth/cas/callback`

## UI Change
- Login page includes CAS CTA button text:
  - `"Login with BlazerID"`

## Notes
- Existing local email/password login remains available.
- CAS-provisioned users are created with a generated password hash to satisfy non-null schema.
