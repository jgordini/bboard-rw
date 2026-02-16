# AGENTS.md

Guidance for coding agents working in `/Users/jeremy/repos/bboard-rw`.

## Scope and Priority

- This file is for agent workflows and code conventions in this repository.
- Primary stack: Rust + Leptos (SSR + WASM hydration) + Axum + sqlx + PostgreSQL.
- If instructions conflict, follow explicit user instructions first, then repository-specific docs.

## External Agent Rules (Cursor/Copilot)

- Checked `.cursor/rules/`: not present.
- Checked `.cursorrules`: not present.
- Checked `.github/copilot-instructions.md`: not present.
- No Cursor/Copilot-specific repository rules are currently defined.

## Setup and Build Commands

- One-time prerequisites:
  - `rustup target add wasm32-unknown-unknown`
  - `cargo install cargo-leptos`
  - `cargo install sqlx-cli`
- Environment setup:
  - `cp .env.example .env`
  - Fill required env vars (`DATABASE_URL`, auth/mail/CAS settings).
- Database setup:
  - `cargo sqlx database setup`
- Local dev server (hot reload):
  - `cargo leptos watch`
- Production build:
  - `cargo leptos build -r`
- Docker-based local stack:
  - `docker compose up`

## Lint, Format, and Quality Commands

- Format code:
  - `cargo fmt`
- Check formatting (CI-friendly):
  - `cargo fmt --all -- --check`
- Lint (recommended):
  - `cargo clippy --all-targets --all-features -- -D warnings`
- Build-check all targets/features:
  - `cargo check --all-targets --all-features`

## Test Commands (Including Single-Test Runs)

- Run all unit/integration tests:
  - `cargo test`
- Run a single test by exact/substring name:
  - `cargo test validate_idea_title_and_content_rejects_empty_title`
- Run tests in a specific module:
  - `cargo test --lib routes::validation_helpers::tests`
- Run one specific module test path:
  - `cargo test --lib routes::validation_helpers::tests::validate_comment_content_enforces_rules`
- Useful extra output while debugging tests:
  - `cargo test <test_name> -- --nocapture`
- End-to-end tests (Playwright via cargo-leptos):
  - `cargo leptos end-to-end`
- Run a single e2e test:
  - `cd end2end && npx playwright test --grep "test name"`

## SQLx and Database Workflow

- SQL migrations auto-run at app startup via `sqlx::migrate!()` in `src/database.rs`.
- SQL queries use `sqlx::query!` and `sqlx::query_as!` macros.
- After changing SQL queries, refresh offline metadata:
  - `cargo sqlx prepare`
- Keep `.sqlx/` metadata in sync with query changes.

## Code Organization Expectations

- Entry points:
  - `src/main.rs` for SSR runtime startup.
  - `src/lib.rs` for crate modules and WASM `hydrate()` entry.
- Route modules live in `src/routes/`.
- Data models live in `src/models/`.
- Shared styling is in `style/main.scss`.
- Legacy files from RealWorld fork are intentionally unused; avoid new work there:
  - `models/article.rs`, `models/pagination.rs`,
  - `routes/home.rs`, `routes/editor.rs`, `routes/article.rs`,
  - `routes/profile.rs`, `routes/settings.rs`, `src/components/`.

## Route File Pattern (Follow This)

- Keep `#[server]` functions near the top of each route file.
- Keep page/component wiring toward the bottom.
- Put route-specific subcomponents in sibling `components/` modules.
- Import SSR-only items inside server function bodies where possible.
- Use role guards from `auth.rs` in server fns:
  - `require_auth()`, `require_moderator()`, `require_admin()`.

## Rust Style and Conventions

- Let `rustfmt` drive formatting; do not hand-format against rustfmt output.
- Naming:
  - `snake_case` for functions/modules/variables.
  - `PascalCase` for structs/enums/components.
  - `UPPER_SNAKE_CASE` for constants (for example route paths/stage arrays).
- Types:
  - Prefer concrete types and explicit return types on public functions.
  - Use `Result<T, E>` consistently; avoid panics except for unrecoverable startup invariants.
- Traits/derives:
  - Keep derives explicit (`Serialize`, `Deserialize`, `Clone`, `Debug`, `PartialEq`) as needed.
- Comments:
  - Add concise comments only for non-obvious behavior or invariants.

## Imports and Feature Gating

- Common ordering pattern in files:
  - `crate::...` imports,
  - then external crates,
  - then `std::...` imports.
- Gate server-only code with `#[cfg(feature = "ssr")]`.
- Keep SSR-only modules/functions unavailable to hydration build unless required.
- In mixed files, conditionally import SSR dependencies to prevent WASM build breakage.

## Error Handling Patterns

- For Leptos server actions/functions, return `Result<_, ServerFnError>`.
- Map internal errors to user-safe messages.
- Preferred pattern in route server functions:
  - Log technical details with `server_fn_error_with_log(...)`.
  - Return concise user-facing error text.
- For model/database methods, use `Result<_, sqlx::Error>`.
- Avoid exposing secrets, raw SQL, or sensitive internals in user-visible errors.

## Leptos UI Conventions

- Use `#[component]` for view components.
- Keep route-level pages thin; push UI details into route component modules.
- Use `Resource`/`ServerAction` and small helper functions for state transitions.
- Keep path literals centralized in `src/routes/paths.rs` when reused.

## Auth and Security Expectations

- Session model is `UserSession` via HTTP-only cookie `user_session`.
- Role values: `0=user`, `1=moderator`, `2=admin`.
- Always enforce authorization in server functions, not just in UI rendering.
- Maintain validation rules on server-side boundaries.

## Testing and Validation Conventions

- Prefer small unit tests near helper logic (`#[cfg(test)]` modules).
- Existing unit tests are concentrated in:
  - `src/routes/validation_helpers.rs`
  - `src/profanity.rs`
- For bug fixes, add or adjust the narrowest test that reproduces the behavior.
- Run targeted tests first, then broader suites when practical.
