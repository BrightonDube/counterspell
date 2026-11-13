FROM node:25-bookworm-slim AS web-builder
WORKDIR /app/web

COPY web/package.json web/package-lock.json ./
RUN npm ci
COPY web/ ./
RUN npm run build

FROM rust:1.99-bookworm AS rust-builder
ENV CARGO_HTTP_TIMEOUT=60 \
    CARGO_NET_RETRY=5
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
RUN cargo build --release -p counterspell-server -p counterspell-judge -p counterspell-grimoire -p counterspell-mcp

FROM rust:1.99-bookworm AS migration-tools
ENV CARGO_HTTP_TIMEOUT=60 \
    CARGO_NET_RETRY=5
ARG SQLX_CLI_VERSION=0.8.6
RUN cargo install sqlx-cli --version "${SQLX_CLI_VERSION}" --no-default-features --features rustls,postgres --locked

FROM debian:bookworm-slim AS server-runtime
RUN groupadd --system --gid 10001 counterspell \
    && useradd --system --uid 10001 --gid counterspell --no-create-home counterspell
COPY --from=rust-builder /app/target/release/counterspell-server /usr/local/bin/counterspell-server
COPY --from=web-builder /app/web/build /app/web/build

ENV HOST=0.0.0.0 \
    PORT=8080 \
    STATIC_DIR=/app/web/build
USER counterspell:counterspell
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/counterspell-server"]

FROM debian:bookworm-slim AS judge-runtime
RUN groupadd --system --gid 10001 counterspell \
    && useradd --system --uid 10001 --gid counterspell --no-create-home counterspell
COPY --from=rust-builder /app/target/release/counterspell-judge /usr/local/bin/counterspell-judge
USER counterspell:counterspell
ENTRYPOINT ["/usr/local/bin/counterspell-judge"]

FROM debian:bookworm-slim AS migration-runtime
RUN groupadd --system --gid 10001 counterspell \
    && useradd --system --uid 10001 --gid counterspell --no-create-home counterspell
COPY --from=migration-tools /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
COPY migrations/ /app/migrations/
USER counterspell:counterspell
WORKDIR /app
ENTRYPOINT ["/usr/local/bin/sqlx"]
