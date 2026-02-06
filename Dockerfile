FROM rust:1.91-bookworm as builder

ENV CARGO_BUILD_JOBS=1

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
    mkdir -p /app

WORKDIR /app
ENV JWT_SECRET="replaceme when ran in prod"
COPY . .

ENV SQLX_OFFLINE=true
RUN cargo leptos build -r -vv

FROM debian:bookworm-slim as runner

WORKDIR /app

COPY --from=builder /app/target/release/realworld-leptos /app/realworld-leptos
COPY --from=builder /app/target/site /app/site

ENV LEPTOS_OUTPUT_NAME="realworld-leptos"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_SITE_PKG_DIR="pkg"

EXPOSE 8080

# Remember to set JWT_SECRET and DATABASE_URL environmental variables
CMD ["/app/realworld-leptos"]
