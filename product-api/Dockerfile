# ビルド用イメージ
FROM rust:1.81 as builder
WORKDIR /app

# 依存関係をキャッシュ
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true

# ソースコードをコピーしてビルド
COPY . .
RUN cargo build --release

# 実行用イメージ
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/product-api .
EXPOSE 8080
CMD ["./product-api"]
