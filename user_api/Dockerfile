# --- ビルド用ステージ ---
FROM rust:slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# ここで sqlx-cli をインストール（バイナリを直接落とすより確実です）
# 必要な機能(postgres)だけに絞ることでビルド時間を短縮
RUN cargo install sqlx-cli --version 0.7.3 --no-default-features --features postgres

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && rm -rf src

COPY src ./src
COPY migrations ./migrations
RUN cargo build --release

# --- 実行用ステージ ---
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \  
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# ビルドしたアプリをコピー
COPY --from=builder /app/target/release/user-api /app/user-api
# builderでインストールした sqlx バイナリをコピー
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
COPY --from=builder /app/migrations /app/migrations

# 起動スクリプトの修正（実行ファイル名を user-api に統一）
RUN echo '#!/bin/sh\n\
echo "Waiting for database..."\n\
sleep 5\n\
echo "Running migrations..."\n\
# DATABASE_URLは環境変数として外（docker-compose等）から渡す前提\n\
sqlx migrate run\n\
echo "Starting application..."\n\
exec /app/user-api' > /app/start.sh && \
    chmod +x /app/start.sh

EXPOSE 8085

CMD ["/app/start.sh"]