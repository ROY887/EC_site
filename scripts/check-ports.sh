#!/usr/bin/env bash
# ポート定義の一致を検査する。
# 既製の k8s リンタは Rust のソースを読まないため、
# 「アプリがリッスンするポート」と「マニフェストのポート」のずれは検出できない。
# ここではリポジトリ全体を横断して突き合わせる。
# 使い方:  ./scripts/check-ports.sh   (make check-ports)
# 終了コード: 0 = 一致, 1 = 不一致あり

set -uo pipefail
cd "$(dirname "$0")/.." || exit 1

RED=$'\033[31m'; GREEN=$'\033[32m'; YELLOW=$'\033[33m'; BOLD=$'\033[1m'; RESET=$'\033[0m'
fail=0

# サービス名:期待ポート:ソースディレクトリ:マニフェスト:Service名
services=(
  "product-api:8084:product_api:k8s/product_api_deployment.yaml:product-service"
  "user-api:8081:user_api:k8s/user_api_deployment.yaml:user-service"
  "cart-api:8085:cart_api:k8s/cart_api_deployment.yaml:cart-service"
)

check() {
  local label="$1" expected="$2" actual="$3"
  if [[ "$actual" == "$expected" ]]; then
    printf '  %s✓%s %-28s %s\n' "$GREEN" "$RESET" "$label" "$actual"
  else
    printf '  %s✗%s %-28s %s (期待値: %s)\n' "$RED" "$RESET" "$label" "${actual:-未検出}" "$expected"
    fail=1
  fi
}

printf '%s=== ポート定義の一致検査 ===%s\n\n' "$BOLD" "$RESET"

for entry in "${services[@]}"; do
  IFS=':' read -r name port dir manifest svc <<< "$entry"
  printf '%s%s (期待ポート %s)%s\n' "$BOLD" "$name" "$port" "$RESET"

  # 1. Rust の既定ポート（PORT 未設定時のフォールバック）
  check "Rust unwrap_or" "$port" \
    "$(grep -oE 'unwrap_or\([0-9]+\)' "$dir/src/main.rs" 2>/dev/null | grep -oE '[0-9]+' | head -1)"

  # 2. Dockerfile EXPOSE
  check "Dockerfile EXPOSE" "$port" \
    "$(grep -oE '^EXPOSE +[0-9]+' "$dir/Dockerfile" 2>/dev/null | grep -oE '[0-9]+' | head -1)"

  # 3. docker-compose のポート公開
  check "compose ports" "$port" \
    "$(grep -A30 "^  ${name}:" docker-compose.yaml 2>/dev/null | grep -oE '"[0-9]+:[0-9]+"' | head -1 | tr -d '"' | cut -d: -f1)"

  # 4. Deployment の PORT 環境変数
  check "Deployment env PORT" "$port" \
    "$(grep -A2 'name: PORT' "$manifest" 2>/dev/null | grep -oE 'value: "[0-9]+"' | grep -oE '[0-9]+' | head -1)"

  # 5. containerPort
  check "containerPort" "$port" \
    "$(grep -oE 'containerPort: [0-9]+' "$manifest" 2>/dev/null | grep -oE '[0-9]+' | head -1)"

  # 6. プローブのポート（3 種類すべて一致すること）
  probe_ports=$(grep -oE '^ +port: [0-9]+' "$manifest" 2>/dev/null | grep -oE '[0-9]+' | sort -u | tr '\n' ' ')
  check "probe/service ports" "$port " "$probe_ports"

  # 7. HTTPRoute の backendRef（名前とポート）
  route_file=$(grep -rl "name: ${svc}" k8s/gateway/ 2>/dev/null | head -1)
  if [[ -n "$route_file" ]]; then
    check "HTTPRoute backend port" "$port" \
      "$(grep -A3 "name: ${svc}" "$route_file" 2>/dev/null | grep -oE 'port: [0-9]+' | grep -oE '[0-9]+' | head -1)"
  else
    printf '  %s✗%s %-28s HTTPRoute が Service "%s" を参照していません\n' "$RED" "$RESET" "HTTPRoute backendRef" "$svc"
    fail=1
  fi
  echo
done

# --- Service 名の実在確認 ---
printf '%sHTTPRoute の backendRefs が実在する Service を指しているか%s\n' "$BOLD" "$RESET"
for ref in $(grep -rhoE 'name: [a-z-]+service' k8s/gateway/*.yaml | awk '{print $2}' | sort -u); do
  if grep -rqE "^  name: ${ref} *$" k8s/*.yaml; then
    printf '  %s✓%s %s\n' "$GREEN" "$RESET" "$ref"
  else
    printf '  %s✗%s %s — 実在しません\n' "$RED" "$RESET" "$ref"
    fail=1
  fi
done
echo

# --- Gateway 名の一致 ---
printf '%sHTTPRoute の parentRefs が実在する Gateway を指しているか%s\n' "$BOLD" "$RESET"
gw=$(grep -A3 'kind: Gateway$' k8s/gateway/gateway.yaml | grep -oE '^  name: .*' | awk '{print $2}' | head -1)
for ref in $(grep -rhoA2 'parentRefs:' k8s/gateway/httproute-*.yaml | grep -oE 'name: [a-z-]+' | awk '{print $2}' | sort -u); do
  if [[ "$ref" == "$gw" ]]; then
    printf '  %s✓%s %s\n' "$GREEN" "$RESET" "$ref"
  else
    printf '  %s✗%s %s — 実在する Gateway は "%s"\n' "$RED" "$RESET" "$ref" "$gw"
    fail=1
  fi
done
echo

if [[ $fail -eq 0 ]]; then
  printf '%s✓ すべて一致しています%s\n' "$GREEN" "$RESET"
else
  printf '%s✗ 不一致があります（上記 ✗ を修正してください）%s\n' "$RED" "$RESET"
  printf '%sヒント: 適用後は HTTPRoute の status でも確認できます%s\n' "$YELLOW" "$RESET"
  printf '  kubectl get httproute -n ec-system -o jsonpath="{range .items[*]}{.metadata.name}{\\"\\t\\"}{.status.parents[*].conditions[?(@.type==\\"ResolvedRefs\\")].status}{\\"\\n\\"}{end}"\n'
fi

exit $fail
