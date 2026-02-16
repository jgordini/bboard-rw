# Fix Notes: Board Title Spark

Date: 2026-02-16

## Goal
- Align board page title text with the updated branding (`Spark`).

## Code Change
- Updated document title in `src/routes/ideas/components/board.rs`:
  - `<Title text="UAB IT Idea Board"/>` -> `<Title text="Spark"/>`
- Visible header title (`<h1 class="logo-font">"Spark"</h1>`) was already set.

## Verification
- `cargo check` ✅
- `cargo clippy --all-targets --all-features` ✅
- `cargo test` ⚠️ fails on existing unrelated test:
  - `profanity::tests::test_number_substitution` (`src/profanity.rs:84`)
