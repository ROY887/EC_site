.PHONY: help build up down restart logs ps clean front health check-ports \
        k8s-secrets k8s-apply k8s-status k8s-migrate k8s-delete \
        monitoring-install monitoring-apply monitoring-status monitoring-uninstall
.SILENT:

help:
	echo "======================================================"
	echo ""
	echo "ローカル開発 (docker-compose + Vite)"
	echo "  make build        バックエンドのイメージをビルド"
	echo "  make up           PostgreSQL + 3 API を起動（マイグレーション込み）"
	echo "  make front        フロントエンドを開発サーバーで起動"
	echo "  make down         停止"
	echo "  make clean        停止してボリューム（DB データ）も削除"
	echo "  make logs         全ログを表示"
	echo "  make ps           起動中のコンテナ"
	echo "  make health       3 API の疎通確認"
	echo "  make db-shell     psql に接続"
	echo ""
	echo "検査"
	echo "  make check-ports  ポート定義の一致をチェック"
	echo ""
	echo "Kubernetes (k3s)"
	echo "  make k8s-secrets  Secret を生成して登録"
	echo "  make k8s-apply    マニフェストを適用"
	echo "  make k8s-migrate  マイグレーション Job を再実行"
	echo "  make k8s-status   状態を確認"
	echo "  make k8s-delete   ワークロードを削除"
	echo ""
	echo "監視（任意）"
	echo "  make monitoring-install    kube-prometheus-stack を導入"
	echo "  make monitoring-apply      監視マニフェストを適用"
	echo "  make monitoring-status     監視スタックの状態"
	echo "  make monitoring-uninstall  監視スタックを削除（PVC は残る）"

# ---------- ローカル開発 ----------

build:
	docker compose build

up:
	docker compose up -d
	echo "バックエンド起動完了。フロントは 'make front' で起動してください。"

front:
	cd frontend && npm run dev

down:
	docker compose down

ps:
	docker compose ps

logs:
	docker compose logs -f

logs-product:
	docker compose logs -f product-api

logs-user:
	docker compose logs -f user-api

logs-cart:
	docker compose logs -f cart-api

logs-db:
	docker compose logs -f postgres

restart:
	docker compose restart

db-shell:
	docker exec -it ec_postgres psql -U ecuser -d ec_db

clean:
	docker compose down -v
	echo "DB のボリュームも削除しました。次回起動時にマイグレーションが再実行されます。"

health:
	echo "疎通確認中..."
	curl -sf http://localhost:8084/health && echo "  <- product-api OK" || echo "product-api DOWN"
	curl -sf http://localhost:8081/health && echo "  <- user-api OK"    || echo "user-api DOWN"
	curl -sf http://localhost:8085/health && echo "  <- cart-api OK"    || echo "cart-api DOWN"

# ---------- 検査 ----------

check-ports:
	./scripts/check-ports.sh

# ---------- Kubernetes ----------

k8s-secrets:
	./scripts/create-secrets.sh

k8s-apply:
	kubectl apply -f k8s/namespace.yaml
	kubectl apply -f k8s/StorageClass.yaml
	kubectl apply -f k8s/DB_Stateful.yaml
	kubectl apply -f k8s/migration_job.yaml
	kubectl apply -f k8s/product_api_deployment.yaml
	kubectl apply -f k8s/user_api_deployment.yaml
	kubectl apply -f k8s/cart_api_deployment.yaml
	kubectl apply -f k8s/frontend_deployment.yaml
	kubectl apply -f k8s/gateway/
	kubectl apply -f k8s/networkpolicy.yaml
	kubectl apply -f k8s/db_backup_cronjob.yaml
	kubectl apply -f k8s/HorizontalPodAutoscaler.yaml
	kubectl apply -f k8s/cloudflared.yaml
	echo "適用完了。'make k8s-status' で確認してください。"

k8s-migrate:
	kubectl delete job db-migrate -n ec-backend --ignore-not-found
	kubectl apply -f k8s/migration_job.yaml
	kubectl wait --for=condition=complete job/db-migrate -n ec-backend --timeout=180s

k8s-status:
	echo "--- Pods ---"
	kubectl get pods -n ec-backend -n ec-frontend -A -l app.kubernetes.io/part-of=ec-microservices
	echo ""
	echo "--- Gateway ---"
	kubectl get gateway,httproute -n ec-system
	echo ""
	echo "--- HPA ---"
	kubectl get hpa -A

k8s-delete:
	kubectl delete -f k8s/cloudflared.yaml --ignore-not-found
	kubectl delete -f k8s/gateway/ --ignore-not-found
	kubectl delete -f k8s/frontend_deployment.yaml --ignore-not-found
	kubectl delete -f k8s/cart_api_deployment.yaml --ignore-not-found
	kubectl delete -f k8s/user_api_deployment.yaml --ignore-not-found
	kubectl delete -f k8s/product_api_deployment.yaml --ignore-not-found
	echo "DB とネームスペースは残しています（必要なら手動で削除してください）。"

# ---------- 監視（任意・k8s/monitoring/） ----------
# 本体のデプロイ（k8s-apply）には含めない。監視は opt-in。

HELM_RELEASE ?= kps
KPS_VERSION  ?= 88.3.0

monitoring-install:
	helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
	helm repo update prometheus-community
	helm upgrade --install $(HELM_RELEASE) prometheus-community/kube-prometheus-stack \
	  --version $(KPS_VERSION) \
	  --namespace ec-monitoring --create-namespace \
	  -f k8s/monitoring/values-kube-prometheus-stack.yaml \
	  --wait --timeout 10m
	echo "導入完了。'make monitoring-apply' で監視対象を登録してください。"

monitoring-apply:
	kubectl apply -f k8s/monitoring/blackbox-exporter.yaml
	kubectl apply -f k8s/monitoring/podmonitor-infra.yaml
	kubectl apply -f k8s/monitoring/alertmanager-config.yaml
	kubectl apply -f k8s/monitoring/PrometheusRule.yaml
	kubectl apply -f k8s/monitoring/probe-internal.yaml
	if [ -f k8s/monitoring/probe-external.yaml ]; then \
	  kubectl apply -f k8s/monitoring/probe-external.yaml; \
	else \
	  echo "注意: probe-external.yaml が無いため外部プローブは未適用です。"; \
	  echo "  sed 's/EXAMPLE_DOMAIN/<ドメイン>/g' k8s/monitoring/probe-external.example.yaml > k8s/monitoring/probe-external.yaml"; \
	fi
	echo "適用完了。'make monitoring-status' で確認してください。"

monitoring-status:
	echo "--- Pods ---"
	kubectl get pods -n ec-monitoring
	echo ""
	echo "--- Probe / PodMonitor / Rule ---"
	kubectl get probe,podmonitor,prometheusrule,alertmanagerconfig -n ec-monitoring
	echo ""
	echo "Prometheus: kubectl port-forward -n ec-monitoring svc/$(HELM_RELEASE)-kube-prometheus-stack-prometheus 9090:9090"
	echo "Grafana   : kubectl port-forward -n ec-monitoring svc/$(HELM_RELEASE)-grafana 3000:80"

monitoring-uninstall:
	helm uninstall $(HELM_RELEASE) --namespace ec-monitoring || true
	echo "PVC は StorageClass standard が Retain のため残ります。"
	kubectl get pvc -n ec-monitoring
