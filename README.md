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
# Edit .env with your database credentials
cargo sqlx database setup
cargo leptos watch
```

The app automatically loads `.env` at startup via `dotenvy`, so you don't need to `source .env` manually.

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

## Cloudflare + HTTPS (Caddy, RC Cloud)

For origin HTTPS behind Cloudflare, this repo supports `caddy` with the
Cloudflare DNS challenge plugin.

1. Copy `scripts/rc-cloud.env.example` to `/var/bboard-rw/.env` on the server.
2. Set a valid `CLOUDFLARE_DNS_API_TOKEN` and `LETSENCRYPT_EMAIL`.
3. Deploy with `ENABLE_LETSENCRYPT=true` in `.env`:
   - `./scripts/update-rc-cloud.sh ubuntu@<ip>`
   - Remote: `cd /var/bboard-rw && ./scripts/deploy.sh replace`
4. In Cloudflare, set SSL/TLS mode to `Full (strict)`.

With `ENABLE_LETSENCRYPT=true`, the app backend remains private at
`127.0.0.1:8080` and Caddy serves public `80/443`.
