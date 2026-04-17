# product-api (Rust + Axum)

## 概要
商品を管理するマイクロサービス。  
PostgreSQL をバックエンドに使用し、Kubernetes 上で稼働させる。

## エンドポイント
- `GET /health` - ヘルスチェック
- `GET /products` - 商品一覧を返す

## 環境変数
- `DATABASE_URL=postgres://user:password@postgres:5432/product_db`

## ビルド & 実行
```bash
cargo build
cargo run

