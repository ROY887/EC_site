#!/usr/bin/env bash
# Secret を生成してクラスタに登録する。
# 値はリポジトリに残さず、その場で生成して kubectl に渡す。
# 既に存在する場合は上書きせず終了する（鍵が変わると既存の JWT が全て無効になるため）。
set -euo pipefail

NS_BACKEND=ec-backend
NS_SYSTEM=ec-system
NS_MONITORING=ec-monitoring

green() { printf '\033[32m%s\033[0m\n' "$1"; }
yellow() { printf '\033[33m%s\033[0m\n' "$1"; }

kubectl get namespace "$NS_BACKEND" >/dev/null 2>&1 || {
  yellow "ネームスペースがありません。先に kubectl apply -f k8s/namespace.yaml を実行してください。"
  exit 1
}

#  postgres-secret 
if kubectl get secret postgres-secret -n "$NS_BACKEND" >/dev/null 2>&1; then
  yellow "postgres-secret は既に存在します（スキップ）"
else
  # URL に含めるため記号を含まない文字集合にする
  PG_PASSWORD=$(LC_ALL=C tr -dc 'A-Za-z0-9' </dev/urandom | head -c 32)
  kubectl create secret generic postgres-secret \
    --namespace "$NS_BACKEND" \
    --from-literal=postgres-user=ecuser \
    --from-literal=postgres-password="$PG_PASSWORD" \
    --from-literal=database-url="postgres://ecuser:${PG_PASSWORD}@postgres-service.${NS_BACKEND}.svc.cluster.local:5432/ec_db"
  green "postgres-secret を作成しました"
fi

#　jwt-secret 
if kubectl get secret jwt-secret -n "$NS_BACKEND" >/dev/null 2>&1; then
  yellow "jwt-secret は既に存在します（スキップ）"
else
  JWT_SECRET=$(LC_ALL=C tr -dc 'A-Za-z0-9' </dev/urandom | head -c 64)
  kubectl create secret generic jwt-secret \
    --namespace "$NS_BACKEND" \
    --from-literal=jwt-secret="$JWT_SECRET"
  green "jwt-secret を作成しました（user-api と cart-api が共有します）"
fi

#　cloudflare-token 
kubectl get namespace "$NS_SYSTEM" >/dev/null 2>&1 || kubectl create namespace "$NS_SYSTEM"

if kubectl get secret cloudflared-token -n "$NS_SYSTEM" >/dev/null 2>&1; then
  yellow "cloudflared-token は既に存在します（スキップ）"
else
  yellow "Cloudflare Tunnel のトークンを入力してください"
  yellow "（Zero Trust > Networks > Tunnels で取得。入力は表示されません）"
  read -rsp "TUNNEL_TOKEN: " TUNNEL_TOKEN
  echo
  if [[ -z "${TUNNEL_TOKEN}" ]]; then
    yellow "入力が空のためスキップしました。後で以下を実行してください:"
    echo "  kubectl create secret generic cloudflared-token -n ${NS_SYSTEM} --from-literal=token='<トークン>'"
  else
    kubectl create secret generic cloudflared-token \
      --namespace "$NS_SYSTEM" \
      --from-literal=token="$TUNNEL_TOKEN"
    green "cloudflared-token を作成しました"
  fi
fi

echo
green "完了。登録済みの Secret:"

# ---------- 監視スタック用（k8s/monitoring/ を使う場合のみ必要） ----------
kubectl get namespace "$NS_MONITORING" >/dev/null 2>&1 || kubectl create namespace "$NS_MONITORING"

# --- Grafana の管理者パスワード ---
if kubectl get secret grafana-admin -n "$NS_MONITORING" >/dev/null 2>&1; then
  yellow "grafana-admin は既に存在します（スキップ）"
else
  GRAFANA_PASSWORD=$(LC_ALL=C tr -dc 'A-Za-z0-9' </dev/urandom | head -c 32)
  kubectl create secret generic grafana-admin \
    --namespace "$NS_MONITORING" \
    --from-literal=admin-user=admin \
    --from-literal=admin-password="$GRAFANA_PASSWORD"
  green "grafana-admin を作成しました"
  yellow "  初回ログイン: admin / 下記のパスワード（この表示は一度きり）"
  echo   "  $GRAFANA_PASSWORD"
fi

# --- Blackbox Exporter の設定（プローブ用アカウントのパスワードを含む） ---
if kubectl get secret blackbox-exporter-config -n "$NS_MONITORING" >/dev/null 2>&1; then
  yellow "blackbox-exporter-config は既に存在します（スキップ）"
else
  TEMPLATE="$(dirname "$0")/../k8s/monitoring/blackbox-config.template.yml"
  if [[ ! -f "$TEMPLATE" ]]; then
    yellow "テンプレートが見つかりません: $TEMPLATE（スキップ）"
  else
    PROBE_EMAIL="probe@example.com"
    # プローブ用アカウントのパスワード。8 文字以上（user-api の制約）。
    PROBE_PASSWORD=$(LC_ALL=C tr -dc 'A-Za-z0-9' </dev/urandom | head -c 24)
    TMPCFG=$(mktemp)
    trap 'rm -f "$TMPCFG"' EXIT
    sed -e "s|__PROBE_EMAIL__|${PROBE_EMAIL}|g" \
        -e "s|__PROBE_PASSWORD__|${PROBE_PASSWORD}|g" "$TEMPLATE" > "$TMPCFG"
    kubectl create secret generic blackbox-exporter-config \
      --namespace "$NS_MONITORING" \
      --from-file=config.yml="$TMPCFG"
    rm -f "$TMPCFG"
    green "blackbox-exporter-config を作成しました"
    yellow "  プローブ用アカウントを API 経由で作成してください（この表示は一度きり）:"
    echo   "  email    : ${PROBE_EMAIL}"
    echo   "  password : ${PROBE_PASSWORD}"
  fi
fi

#AlertmanagerのSlack Webhook 
if kubectl get secret alertmanager-slack -n "$NS_MONITORING" >/dev/null 2>&1; then
  yellow "alertmanager-slack は既に存在します（スキップ）"
else
  yellow "Slack の Incoming Webhook URL を入力してください"
  yellow "（未設定なら空 Enter でスキップ。入力は表示されません）"
  read -rsp "SLACK_WEBHOOK_URL: " SLACK_WEBHOOK_URL
  echo
  if [[ -z "${SLACK_WEBHOOK_URL}" ]]; then
    yellow "入力が空のためスキップしました。後で以下を実行してください:"
    echo "  kubectl create secret generic alertmanager-slack -n ${NS_MONITORING} --from-literal=webhook-url='<URL>'"
  else
    kubectl create secret generic alertmanager-slack \
      --namespace "$NS_MONITORING" \
      --from-literal=webhook-url="$SLACK_WEBHOOK_URL"
    green "alertmanager-slack を作成しました"
  fi
fi

kubectl get secret -n "$NS_BACKEND" -o name | sed 's/^/  /'
kubectl get secret -n "$NS_SYSTEM" -o name | sed 's/^/  /'
kubectl get secret -n "$NS_MONITORING" -o name 2>/dev/null | sed 's/^/  /'
