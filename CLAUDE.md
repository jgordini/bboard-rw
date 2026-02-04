# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

UAB IT Idea Board - A full-stack Rust web application for anonymous idea submission and voting. Built with Leptos (full-stack Rust framework), Axum (backend), and PostgreSQL.

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

# Run tests
cargo test

# End-to-end tests (requires Playwright)
cargo leptos end-to-end

# Docker development
docker compose up
```

## Architecture

**Technology Stack:**
- Frontend/Backend: Leptos 0.8 with SSR + WASM hydration
- HTTP Server: Axum
- Database: PostgreSQL with sqlx (compile-time checked queries)
- UI: leptos-shadcn-ui components
- Styling: SCSS with UAB theme colors

**Key Directories:**
- `src/routes/` - Page components with server functions
  - `ideas.rs` - Main board: idea list, submission form, voting
  - `admin.rs` - Password-protected admin panel
- `src/models/` - Data models with database operations
  - `idea.rs` - Idea CRUD operations
  - `vote.rs` - Voting operations
- `src/profanity.rs` - Content moderation filter
- `migrations/` - PostgreSQL schema

**Data Flow:**
Server functions in routes call model methods which execute sqlx queries. The database uses triggers to auto-update vote counts when votes are inserted/deleted.

**Build Features:**
- `ssr` - Server-side rendering (Axum, database, auth)
- `hydrate` - Client-side WASM hydration

## Environment Variables

Required in `.env` (see `.env.example`):
- `DATABASE_URL` - PostgreSQL connection string
- `ADMIN_PASSWORD` - Admin panel password (defaults to "admin")

## Key Business Logic

- Ideas: max 500 chars, validated against profanity filter
- Voting: one vote per fingerprint per idea (tracked via localStorage + DB constraint)
- Admin: password auth, can delete ideas individually or in bulk
