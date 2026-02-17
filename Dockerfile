FROM rust:1.91-bookworm AS chef

ARG TARGETARCH
ARG CARGO_LEPTOS_VERSION=v0.3.2

RUN set -eux; \
    case "$TARGETARCH" in \
      amd64) LEPTOS_ARCH="x86_64-unknown-linux-gnu" ;; \
      arm64) LEPTOS_ARCH="aarch64-unknown-linux-gnu" ;; \
      *) echo "Unsupported TARGETARCH: $TARGETARCH" && exit 1 ;; \
    esac; \
    curl -fsSL "https://github.com/leptos-rs/cargo-leptos/releases/download/${CARGO_LEPTOS_VERSION}/cargo-leptos-${LEPTOS_ARCH}.tar.gz" -o /tmp/cargo-leptos.tar.gz; \
    tar -xzf /tmp/cargo-leptos.tar.gz -C /usr/local/cargo/bin --strip-components=1 "cargo-leptos-${LEPTOS_ARCH}/cargo-leptos"; \
    chmod +x /usr/local/cargo/bin/cargo-leptos; \
    rm -f /tmp/cargo-leptos.tar.gz; \
    rustup target add wasm32-unknown-unknown; \
    cargo install cargo-chef --locked

WORKDIR /app

# Stage 1: Generate recipe.json (dependency lockfile for caching)
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Build dependencies (cached layer)
FROM chef AS cacher
ENV CARGO_BUILD_JOBS=4
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Stage 3: Build the application
FROM chef AS builder
ENV CARGO_BUILD_JOBS=4
ENV JWT_SECRET="replaceme when ran in prod"
ENV SQLX_OFFLINE=true

COPY . .
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN cargo leptos build -r -vv

FROM debian:bookworm-slim AS runner

WORKDIR /app

COPY --from=builder /app/target/release/uab-spark /app/uab-spark
COPY --from=builder /app/target/site /app/site

ENV LEPTOS_OUTPUT_NAME="uab-spark"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_SITE_PKG_DIR="pkg"

EXPOSE 8080

# Remember to set JWT_SECRET and DATABASE_URL environmental variables
CMD ["/app/uab-spark"]
