# EC_site

Rust (axum) によるマイクロサービスを Kubernetes 上で運用する構成の学習・検証を目的とした、
Amazon 風 EC サイトのデモです。

> **これはポートフォリオ用のデモサイトです。**
> 商品はすべてサンプルで、**注文確定・決済・配送は実装していません**。
> 会員登録の際は、他サービスで使用していないメールアドレスとパスワードをご利用ください。

---

## 構成

```
                      ┌─────────────────┐
   ブラウザ ──HTTPS──▶│  Cloudflare     │  TLS 終端 / WAF / レート制限
                      └────────┬────────┘
                               │ Tunnel（アウトバウンド接続のみ）
    ┌──────────────────────────┼───────────────────────────────────┐
    │ 自宅サーバー / k3s        │                                   │
    │                    ┌─────▼──────┐                            │
    │  ns: ec-system     │ cloudflared│                            │
    │                    └─────┬──────┘                            │
    │                    ┌─────▼──────────────────┐                │
    │                    │ Gateway API (Envoy)    │                │
    │                    │ ec-apigate-way :80     │                │
    │                    └─┬────┬────┬────────┬───┘                │
    │        /api/products │    │    │        │ /                  │
    │             /api/users│   │    │        │                    │
    │                /api/cart│  │   │        │                    │
    │  ns: ec-backend      ▼    ▼    ▼        ▼  ns: ec-frontend   │
    │        ┌─────────┐ ┌──────┐ ┌──────┐  ┌──────────┐          │
    │        │product  │ │user  │ │cart  │  │ frontend │          │
    │        │api x2   │ │api x2│ │api x2│  │ nginx x2 │          │
    │        └────┬────┘ └───┬──┘ └───┬──┘  └──────────┘          │
    │             └──────────┼────────┘                            │
    │                   ┌────▼─────┐                               │
    │                   │PostgreSQL│ StatefulSet + PVC             │
    │                   └──────────┘                               │
    └──────────────────────────────────────────────────────────────┘
```

**設計上のポイント**

- **単一オリジン構成**: フロントと API を同じホスト名で配信し、パスで振り分ける。
  これにより CORS が不要になり、API の URL をフロントのビルドに埋め込む必要もなくなる。
- **ユーザー識別は JWT のみ**: カート API は `user_id` をリクエストで受け取らず、
  検証済みトークンの `sub` から導出する。他人のカートを指定する経路が構造的に存在しない。
- **TLS は Cloudflare 側**: クラスタ内は平文 HTTP。証明書の管理とポート開放が不要。

---

## 技術スタック

| 領域 | 採用 |
| --- | --- |
| バックエンド | Rust 1.9x / axum 0.7 / sqlx 0.7 / tokio |
| 認証 | bcrypt（パスワード）+ JWT HS256（セッション、24h） |
| DB | PostgreSQL 15 |
| フロントエンド | React 19 / Vite 7 / Tailwind CSS 3 |
| コンテナ | マルチステージビルド（distroless 相当の最小実行イメージ・非 root） |
| オーケストレーション | Kubernetes (k3s) / Gateway API / HPA / PDB / NetworkPolicy |
| 公開 | Cloudflare Tunnel |
| 監視 | kube-prometheus-stack / Grafana / Blackbox Exporter（`k8s/monitoring/`・任意） |

---

## API

すべて `/api` プレフィックスで公開され、Gateway が剥がしてバックエンドへ渡す。

### product-api （認証不要・参照のみ）

| Method | Path | 説明 |
| --- | --- | --- |
| GET | `/api/products` | 商品一覧 |
| GET | `/api/products/{id}` | 商品詳細 |
| GET | `/api/products/search?q=` | キーワード検索（64 文字まで） |
| GET | `/api/products/category/{category}` | カテゴリ絞り込み |

商品の登録・更新・削除 API は**実装していない**。データはマイグレーションの seed で投入する。

### user-api

| Method | Path | 認証 | 説明 |
| --- | --- | --- | --- |
| POST | `/api/users` | – | 登録（パスワード 8 文字以上） |
| POST | `/api/login` | – | ログイン、JWT を返す |
| GET | `/api/users/{id}` | 必要 | 本人のみ取得可 |
| PUT | `/api/users/{id}` | 必要 | 本人のみ更新可 |
| DELETE | `/api/users/{id}` | 必要 | 本人のみ削除可 |

### cart-api （すべて認証必須）

| Method | Path | 説明 |
|---|---|---|
| GET | `/api/cart` | カート取得 |
| POST | `/api/cart/add` | `{product_id, quantity}` を追加（既存なら加算） |
| PUT | `/api/cart/update` | `{product_id, quantity}` に更新 |
| DELETE | `/api/cart/remove` | `{product_id}` を削除 |
| DELETE | `/api/cart` | 全削除 |

エラーは HTTP ステータスと `{"error": "..."}` で返す。

---

## ローカル開発

### 1. バックエンドを起動

```bash
make up
```

PostgreSQL と 3 つの API が起動し、マイグレーション（テーブル作成 + サンプル商品投入）が自動実行される。

### 2. フロントエンドを起動

```bash
make front
```

http://localhost:5173 が開く。Vite の proxy が `/api/*` を各サービスへ振り分け、本番の Gateway と同じパス構成を再現する。

### そのほか

```bash
make health       # 3 API の疎通確認
make logs         # ログ追跡
make db-shell     # psql に接続
make clean        # DB ボリュームごと削除
make check-ports  # ポート定義の一致を検査
```

---

## Kubernetes へのデプロイ

### 前提

```bash
# Gateway API の CRD（standard チャネル）
kubectl apply -f https://github.com/kubernetes-sigs/gateway-api/releases/download/v1.5.1/standard-install.yaml
```

```bash
# Gateway API の実装（Envoy Gateway）
# チャートが同梱する Gateway API のバージョンを、上でインストールしたものと
# 揃えること。v1.8.3 は Gateway API v1.5.1 に対応する。
helm install envoy-gateway oci://docker.io/envoyproxy/gateway-helm \
  --version v1.8.3 --namespace envoy-gateway-system --create-namespace --wait
```

k3s は metrics-server を同梱しているため、HPA はそのまま動作する。

> **`controllerName` は変更できない。** ダミー値のまま作成済みの GatewayClass がある場合、
> `kubectl apply` は `Value is immutable` で失敗する。一度削除してから作り直すこと。
>
> ```bash
> kubectl delete gatewayclass ec-gateway-class
> ```

### 手順

```bash
# 1. イメージをビルド（k3s のノード上で実行する）
docker build -t ec-site/product-api:dev ./product_api
docker build -t ec-site/user-api:dev    ./user_api
docker build -t ec-site/cart-api:dev    ./cart_api
docker build -t ec-site/frontend:dev    ./frontend
```

```bash
# 1b. k3s の containerd に取り込む
#     k3s は docker とイメージストアを共有しないため、この手順が必須。
for i in product-api user-api cart-api frontend; do
  docker save ec-site/$i:dev | sudo k3s ctr images import -
done
```

確認: `sudo k3s ctr images ls | grep ec-site` に 4 つ並ぶこと。

```bash
# 2. ネームスペースと Secret
kubectl apply -f k8s/namespace.yaml
make k8s-secrets
```

```bash
# 3. 全マニフェストを適用
make k8s-apply
```

```bash
# 4. 確認
make k8s-status
```

`kubectl get httproute -n ec-system` で `ResolvedRefs` が `True` になっていれば、
Gateway からバックエンドへの参照が解決できている。

### Cloudflare Tunnel

Zero Trust ダッシュボードで Public Hostname を追加し、Service に以下を指定する。

```
http://ec-gateway-entry.envoy-gateway-system.svc.cluster.local:80
```

パスによる振り分けはクラスタ内の Gateway API が行うため、
Cloudflare 側はホスト名を 1 つ登録するだけでよい。

---

## 監視（任意）

`make k8s-apply` には含まれない。必要なときだけ導入する。

```bash
make monitoring-install   # kube-prometheus-stack を導入
make monitoring-apply     # 監視対象を登録
make monitoring-status    # 状態確認
```

Secret は `scripts/create-secrets.sh` が生成する（Grafana 管理者パスワード、
Blackbox のプローブ用アカウント、Slack Webhook）。

### 何を見ているか

| 層 | 取得元 | 内容 |
|---|---|---|
| ノード | node-exporter | CPU / メモリ / **ディスク空き** |
| Kubernetes | kube-state-metrics | Pod・Deployment・PVC・**CronJob の成否** |
| コンテナ | kubelet / cAdvisor | リソース使用量 |
| HTTP | **Envoy** | リクエスト数・レイテンシ・5xx（アプリの計装は不要） |
| 公開経路 | **Blackbox Exporter** | 外形監視（ログイン・商品一覧・認証拒否） |
| トンネル | cloudflared | Cloudflare への接続本数 |

### 外形監視

Blackbox Exporter は 1 リクエストで完結する検査しかできないため、
「ログイン → カート投入」のような多段の流れは再現していない。
代わりに、1 リクエストで検証範囲が最大になる対象を選んでいる。

| モジュール | 検証される範囲 |
|---|---|
| `https_login` | Gateway → user-api → PostgreSQL → bcrypt → JWT 発行 |
| `https_products` | product-api → DB → **データが存在すること**（空配列を異常とみなす） |
| `https_expect_401` | 認証ミドルウェア（401 が返ることが正常） |
| `https_external` | 配信経路全体（Cloudflare → トンネル → Gateway → nginx） |
| `http_internal` | Gateway 直叩き。外部プローブとの差分で障害箇所を切り分ける |

外部プローブはドメイン名を含むためリポジトリに入れていない。

```bash
sed 's/EXAMPLE_DOMAIN/<あなたのドメイン>/g' \
  k8s/monitoring/probe-external.example.yaml > k8s/monitoring/probe-external.yaml
```

### 画面を開く

```bash
kubectl port-forward -n ec-monitoring svc/kps-grafana 3000:80
```

```bash
kubectl get secret grafana-admin -n ec-monitoring -o jsonpath='{.data.admin-password}' | base64 -d
```

### 環境固有の注意

- **k3s には controller-manager / scheduler / proxy / etcd のスクレイプ先が無い。**
  chart の既定のまま導入すると永久に down のターゲットが増え、偽アラートが
  常時発火して本物が埋もれる。values で無効化してある。
- **このノードは IPv4 と IPv6 の両方を InternalIP に登録している。**
  Pod ネットワークは IPv4 のみのため、kubelet の IPv6 側ターゲットへは到達できない。
  values の relabeling で除外している。
- 監視の導入自体がディスクを消費する（Prometheus 10Gi / Grafana 2Gi /
  Alertmanager 2Gi）。`standard` は `Retain` なので uninstall しても PVC は残る。

### 検知できないこと

**ノードごと停止した場合、この監視スタックも一緒に停止する。**
サイトが落ちたことを外部へ通知できない。ノード障害と回線障害を検知するには、
クラスタの外に死活監視を置く必要がある。

---

## セキュリティ

**実装済み**

- パスワードは bcrypt でハッシュ化し、レスポンスに一切含めない
- カート・ユーザー情報の操作は JWT 検証を通し、本人のリソースのみ許可
- 全 Pod が非 root・読み取り専用ルートファイルシステム・capabilities 全 drop
- NetworkPolicy で「Gateway → API」「API → DB」以外の通信を遮断
- Secret はリポジトリに含めず、`scripts/create-secrets.sh` がその場で生成
- 検索クエリの長さ制限と LIKE ワイルドカードのエスケープ
- nginx でセキュリティヘッダと CSP を付与

**既知の制約**

- 認証トークンを localStorage に保持しているため、XSS でトークンを読み出されうる。
  実サービス化する場合は httpOnly Cookie への移行が必要。
- ログインのレート制限は Cloudflare 側の WAF ルールに依存している。
- 単一ノード構成のため、ノード障害時は全サービスが停止する。

---

## リポジトリ構成

```
EC_site/
├── product_api/     商品サービス (axum, :8084)
├── user_api/        認証・ユーザーサービス (axum, :8081)
├── cart_api/        カートサービス (axum, :8085)
├── frontend/        React + Vite（本番は nginx で静的配信）
├── k8s/
│   ├── gateway/     Gateway API 一式
│   ├── monitoring/  監視スタック一式（任意）
│   └── *.yaml       Deployment / StatefulSet / Job / NetworkPolicy 等
├── scripts/
│   ├── check-ports.sh    ポート定義の横断検査
│   └── create-secrets.sh Secret 生成・登録
├── DESIGN.md        設計書
└── RELEASE_PLAN.md  公開までの作業計画
```

---

## 未実装

- 注文確定・決済・配送・在庫引当
- 管理画面と管理者ロール
- アプリケーション自身のメトリクス（`/metrics`）。
  HTTP レベルの指標は Envoy から取得しているため、可視化自体はできている。
- CI/CD パイプライン
- 自動テスト
