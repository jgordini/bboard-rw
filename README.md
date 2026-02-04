# UAB IT Idea Board

An anonymous idea submission and voting platform for UAB IT, built with Leptos + Axum + PostgreSQL.

## Requirements

### Rust with WebAssembly support

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
```

### cargo-leptos

Compiles both backend and frontend:

```bash
cargo install cargo-leptos
```

### sqlx-cli

Required to run migrations before compiling (sqlx macros verify queries at compile time):

```bash
cargo install sqlx-cli
```

## Local Development

Start a local PostgreSQL database:

```bash
docker run --name postgres -e POSTGRES_PASSWORD=postgres -p 5432:5432 -d postgres
```

Set up and run:

```bash
cp .env.example .env
source .env
cargo sqlx database setup
cargo leptos watch
```

Navigate to http://localhost:3000

## Environment Variables

See `.env.example` for all options:

- `DATABASE_URL` - PostgreSQL connection string
- `ADMIN_PASSWORD` - Password for the admin panel (defaults to "admin")

## Testing

End-to-end tests require a local database and Playwright:

```bash
cd end2end/
npm i
npx playwright install
cd ../
cargo leptos end-to-end
```

## Docker Compose

Run the full stack in release mode:

```bash
docker compose up
```

Navigate to http://localhost:8080
