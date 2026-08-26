# --- ビルド用ステージ ---
FROM rust:1-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install sqlx-cli --version 0.7.4 --no-default-features --features postgres,rustls

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && rm -rf src

COPY src ./src
COPY migrations ./migrations
RUN touch src/main.rs && cargo build --release

# --- 実行用ステージ ---
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 1000 app \
    && useradd --uid 1000 --gid 1000 --no-create-home --shell /usr/sbin/nologin app

WORKDIR /app

COPY --from=builder /app/target/release/cart-api /app/cart-api
COPY --from=builder /usr/local/cargo/bin/sqlx    /usr/local/bin/sqlx
# migrations は実行ユーザー(uid 1000)から読める必要がある。
# COPY は元ファイルのモードをそのまま引き継ぐため、ビルドホストの umask が
# 厳しいと 600 のまま root 所有で入り、Job の sqlx migrate run が
# "Permission denied" で落ちる。ここで明示的に読み取り権を与える。
COPY --from=builder /app/migrations              /app/migrations
RUN chmod -R a+rX /app/migrations

USER 1000:1000

EXPOSE 8085
CMD ["/app/cart-api"]
