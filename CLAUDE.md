# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

UAB Spark (Idea Board) — a full-stack Rust web application for authenticated idea submission, voting, commenting, and moderation. Built with Leptos 0.8 (SSR + WASM hydration), Axum, and PostgreSQL.

## Build & Development Commands

```bash
# Prerequisites (one-time setup)
rustup target add wasm32-unknown-unknown
cargo install cargo-leptos
cargo install sqlx-cli

# Database setup (requires PostgreSQL running)
cargo sqlx database setup

# Development server with hot reload (http://localhost:3000)
cargo leptos watch

# Production build
cargo leptos build -r

# Run all unit tests
cargo test

# Run a single test by name
cargo test validate_idea_title_and_content_rejects_empty_title

# Run tests in a specific module
cargo test --lib routes::validation_helpers::tests

# End-to-end tests (requires app running + Playwright installed)
cargo leptos end-to-end

# Run a single e2e test
cd end2end && npx playwright test --grep "test name"

# Docker development
docker compose up
```

## Architecture

**Stack:** Leptos 0.8 (SSR + WASM hydration), Axum, PostgreSQL/sqlx, SCSS

**Build features** control the SSR/client split:
- `ssr` — server binary: Axum, sqlx, bcrypt, auth, validation, profanity filter
- `hydrate` — WASM client bundle, entry point is `hydrate()` in `lib.rs`
- Both are in the default feature set; `cargo test` uses `ssr` automatically

**Source layout:**

```
src/
├── main.rs              # Entry: calls setup::init_app()
├── lib.rs               # Crate root, mod declarations, WASM hydrate() entry
├── app.rs               # App component, Router, NavBar, shell()
├── auth.rs              # CAS SSO + email/password auth, session cookies, role guards
├── database.rs          # PgPool via OnceLock, auto-runs migrations at startup
├── setup.rs             # Axum router, Leptos route generation, admin bootstrap
├── profanity.rs         # Profanity filter (SSR-only)
├── models/              # Data structs (always compiled) + DB methods (SSR-gated)
│   ├── idea.rs          # Idea CRUD, moderation, stats
│   ├── vote.rs          # Vote toggle, lookup
│   ├── comment.rs       # Comment CRUD, soft-delete, pin
│   ├── user.rs          # User auth, roles, bootstrap_admin
│   └── flag.rs          # Content flagging
├── routes/
│   ├── paths.rs         # Path constants (HOME, LOGIN, ADMIN, etc.)
│   ├── ideas.rs         # IdeasPage + server fns (list, create, vote, flag, sort)
│   ├── ideas/components/ # IdeasBoard, IdeaCard, IdeaSubmissionModal
│   ├── idea_detail.rs   # IdeaDetailPage + server fns (comments, moderation)
│   ├── idea_detail/components/ # IdeaDetailCard, CommentsSection
│   ├── admin.rs         # AdminPage + server fns (stats, flags, export, users)
│   ├── admin/components/ # Tabbed dashboard: Overview, Flags, Moderation, Users, Export
│   ├── login.rs, signup.rs, reset_password.rs, account.rs
│   ├── validation_helpers.rs  # SSR-only input validation (has unit tests)
│   ├── error_helpers.rs       # ServerFnError logging wrappers
│   ├── async_helpers.rs       # spawn_local wrappers for server actions
│   └── view_helpers.rs        # UI utilities (relative time, stage badges, confirm)
```

**Legacy files to ignore:** `models/article.rs`, `models/pagination.rs`, `routes/home.rs`, `routes/editor.rs`, `routes/article.rs`, `routes/profile.rs`, `routes/settings.rs`, `src/components/` — all remnants from a RealWorld fork, unused.

## Route Module Pattern

Each route follows a consistent structure:

1. **Server functions** (`#[server]`) at the top of the file — SSR imports inside the function body
2. **Page component** at the bottom — creates `Resource`s and passes to sub-components
3. **Sub-components** in a parallel `<route>/components/` directory

Server function template:
```rust
#[server]
pub async fn some_action(args...) -> Result<ReturnType, ServerFnError> {
    use crate::auth::require_auth; // SSR imports inside fn body
    let user = require_auth().await?;
    validate_...()?;
    Model::db_operation(...)
        .await
        .map_err(|e| server_fn_error_with_log("context", e, "user message"))
}
```

Auth guards: `require_auth()`, `require_moderator()`, `require_admin()` — defined in `auth.rs`.

## Authentication

**Two login paths:**
1. **CAS SSO (BlazerID)** — redirects to `padlock.idm.uab.edu`, callback at `/auth/cas/callback` auto-creates or retrieves user
2. **Email/password** — signup, login, logout server functions in `auth.rs`; bcrypt hashing

**Sessions:** HTTP-only `user_session` cookie with JSON-serialized `UserSession` (id, email, name, role). 7-day lifetime, SameSite=Lax.

**Roles** (stored as `i16`): 0=User, 1=Moderator, 2=Admin. Admin bootstrap on startup from `INITIAL_ADMIN_EMAIL`/`INITIAL_ADMIN_PASSWORD` env vars.

**Auth refresh:** `AuthRefresh(RwSignal<u32>)` provided via context — bump after login/logout to re-fetch user in NavBar without page reload.

## Database

Migrations run automatically at startup via `sqlx::migrate!()` in `database.rs`. The `.sqlx/` directory (checked in) holds compile-time query metadata for offline builds (`SQLX_OFFLINE=true` in Docker).

**Key tables:** `ideas` (title, content, tags, stage, vote_count, pinned_at, comments_enabled), `votes` (user_id + idea_id unique), `comments` (soft-delete via is_deleted, is_pinned), `users` (email unique, role), `flags` (target_type + target_id + user_id unique)

**Triggers:** DB-level triggers auto-maintain `ideas.vote_count` on vote insert/delete.

**Idea stages:** `["Ideate", "Review", "In Progress", "Completed"]` — defined as `STAGES` const in `models/idea.rs`.

After changing any SQL query, regenerate offline metadata: `cargo sqlx prepare`.

## Environment Variables

Required in `.env` (see `.env.example`):
- `DATABASE_URL` — PostgreSQL connection string
- `JWT_SECRET` — for password reset tokens
- `INITIAL_ADMIN_EMAIL` / `INITIAL_ADMIN_PASSWORD` — bootstrap admin (one-time)
- `MAILER_EMAIL` / `MAILER_PASSWD` / `MAILER_SMTP_SERVER` — SMTP for password reset
- `CAS_LOGIN_URL` / `CAS_VALIDATE_URL` / `CAS_SERVICE_ID` — CAS SSO config

Loaded automatically by `dotenvy` at startup.

## Key Business Rules

- Ideas: title max 100 chars, content max 500 chars, tags max 200 chars; validated against profanity filter
- Voting: one vote per user per idea (DB unique constraint)
- Comments: max 500 chars, soft-deleted (is_deleted flag), can be pinned, comments can be locked per-idea
- Flagging: users can flag ideas/comments; moderators review in admin dashboard
- Admin dashboard: 5 tabs — Overview, Flagged Content, Off-Topic, Data Export (CSV), User Management

## Styling

Single SCSS file at `style/main.scss`. UAB theme colors as CSS custom properties (`--uab-green`, `--uab-gold`, etc.). Font: Inter via Google Fonts. Body uses `alt-linear-theme` class.
