# RoboRev Fix: Password Reset Wiring + API Drift

Date: 2026-02-13

## Problem
- Login page linked to `/reset_password`, but no route was registered.
- `reset_password.rs` used stale APIs (`User::get_email`, `auth::encode_token`, `user.set_password().update()`) and panic paths.

## Changes
- Wired reset route into active app routing:
  - `src/routes/mod.rs`: export and module include for `reset_password`.
  - `src/app.rs`: added `Route` for `/reset_password`.
- Modernized `src/routes/reset_password.rs`:
  - Added SSR-safe JWT helpers (`encode_reset_token`, `decode_reset_token`) using `RESET_TOKEN_SECRET` (fallback `JWT_SECRET`).
  - Replaced stale user lookups with `User::get_by_email`.
  - Replaced panic-based email sending flow with graceful error handling + logging.
  - Added password length check in reset step 2.
- Added `src/models/user.rs` method:
  - `set_password_by_email(email, password)` with bcrypt hash + SQL update.

## Outcome
- `/reset_password` is now reachable from login.
- Reset flow compiles against current model/auth code.
- Password reset no longer panics on missing mail config or SMTP failures.
