# Favicon Cache-Busting Verification

Date: 2026-02-16

## Requested change
- Replace `assets/favicon.ico`
- Update favicon link in `src/app.rs` to use `href="/favicon.ico?v=20260214"`

## Verification
- `src/app.rs` already contains:
  - `<link rel="shortcut icon" type="image/ico" href="/favicon.ico?v=20260214"/>`
- `assets/favicon.ico` exists in the repository.
- Git history already includes this update in commit `9c83d82`.

## Validation run
- `cargo check`: passed
- `cargo clippy --all-targets --all-features -- -D warnings`: passed
- `cargo test`: failed in existing test `profanity::tests::test_number_substitution` at `src/profanity.rs:84`
