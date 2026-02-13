# RoboRev Fix: Leptos Watch SSR/Hydrate Compile Split

Date: 2026-02-13

## Problem
- `cargo leptos watch` failed during hydrate/client build because shared validation helpers referenced `crate::profanity`, which is only compiled under `ssr`.

## Changes
- `src/routes/validation_helpers.rs`
  - Added local `contains_profanity` wrapper with cfg split:
    - `ssr`: delegates to `crate::profanity::contains_profanity`
    - non-`ssr`: no-op (`false`)
  - Replaced direct `crate::profanity` calls with wrapper.
- `src/routes/ideas.rs`
  - Marked server-only helper imports with `#[cfg(feature = "ssr")]`.
- `src/routes/idea_detail.rs`
  - Marked server-only helper imports with `#[cfg(feature = "ssr")]`.
- `src/routes/reset_password.rs`
  - Marked `serde::{Deserialize, Serialize}` import as `ssr` only.

## Outcome
- Shared route/helper code now compiles across both server and hydrate targets under `cargo leptos watch`.

## Follow-up
- Added `#![cfg_attr(not(feature = "ssr"), allow(dead_code))]` to:
  - `src/routes/validation_helpers.rs`
  - `src/routes/error_helpers.rs`
- Purpose: avoid hydrate-only dead-code warnings for server-oriented helpers while keeping SSR behavior unchanged.
- Added `#[allow(dead_code)]` to `server_fn_error_with_log` in `src/routes/error_helpers.rs` to suppress hydrate-only usage warning in watch logs.
