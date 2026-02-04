# Local Deployment Guide

## Prerequisites

1. **Rust 1.88+** - Update Rust if needed:
   ```bash
   rustup update stable
   rustup default stable
   ```

2. **PostgreSQL** - Ensure PostgreSQL is running locally

3. **Install cargo-leptos**:
   ```bash
   cargo install cargo-leptos
   ```

4. **Install sqlx-cli** (optional, for manual migrations):
   ```bash
   cargo install sqlx-cli --no-default-features --features postgres
   ```

## Setup Steps

1. **Create database**:
   ```bash
   psql -d postgres -c "CREATE DATABASE bboard;"
   ```

2. **Configure environment**:
   ```bash
   cp .env.example .env
   # Edit .env and set DATABASE_URL to match your PostgreSQL setup
   # Example: DATABASE_URL="postgres://your_user@localhost/bboard"
   ```

3. **Run migrations**:
   ```bash
   source .env
   export DATABASE_URL
   psql -d bboard -f migrations/20221207194615_init.up.sql
   # OR if sqlx-cli is installed:
   cargo sqlx database setup
   ```

4. **Start the development server**:
   ```bash
   source .env
   export DATABASE_URL
   cargo leptos watch
   ```

   The application will be available at: http://127.0.0.1:3000

## Alternative: Run without cargo-leptos

If cargo-leptos is not available, you can build and run directly:

```bash
source .env
export DATABASE_URL
cargo build --features ssr
cargo run --bin realworld-leptos --features ssr
```

## Troubleshooting

- **Rust version issues**: Make sure you're using Rust 1.88+ (check with `rustc --version`)
- **Database connection**: Verify PostgreSQL is running with `pg_isready`
- **Port conflicts**: The app runs on port 3000 by default (configurable in Cargo.toml)

## Current Status

✅ Database created: `bboard`
✅ Migrations applied
✅ Environment configured: `.env` file created

Next: Install cargo-leptos and run `cargo leptos watch`
