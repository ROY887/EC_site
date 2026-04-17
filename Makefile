.PHONY: help build up down restart logs clean db-init
.SILENT:



help:
	echo "EC Microservices - Docker Commands"
	echo "=================================="
	echo "make build     - すべてのイメージをビルドする"
	echo "make clean     - すべてのイメージ、コンテナ、ボリュームを削除する"
	echo "make up 		 - すべてのサービスを起動"
	echo "make down"     - すべてのサービスを停止
	echo "make restart   - すべてのサービスを再起動"
	echo "make lgos      - すべてのログを表示する"
	echo "make logs -f   - ログをフォロー(表示する)"
	echo "make ps        - 起動中のコンテナを表示する"
	echo "make db-init   - データベースの初期化" 
	echo "make config    - yamlファイルの検証(スペルミスなどチェック)"
	echo ""
	echo "Individual services:"
	echo "make logs-product   - Product APIのログ"
	echo "make logs-user      - User APIのログ"
	echo "make logs-cart      - Cart APIのログ"
	echo "make logs-frontend  - Frontendのログ"
	echo "make logs-db        - PostgreSQLのログ"
	


build:
	docker-compose build

up:
	docker-compose up -d 

down:
	docker-compose down

ps:
	docker-compose ps

db-shell:
	docker exec -it ec_postgres psql -U postgres -d ec_db

restart:
	docker-compose restart

logs:
	docker-compose logs

config:
	docker-compose config

config-services:
	docker-compose config --services

config-volumes:
	docker-compose config --volumes

db-restart:
	docker compose restart postgres

logs-product:
	docker-compose logs -f product-api

logs-user:
	docker-compose logs -f user-api

logs-cart:
	docker-compose logs -f cart-api

logs-frontend:
	docker-compose logs -f frontend

clean:
	docker-compose down --rmi all

clean-all:
	docker-system prune -af


rebuild-product:
	docker-compose build product-api
	docker-compose up -d product-api

rebuild-user:
	docker-compose build user-api
	docker-compose up -d user-api

rebuild-cart:
	docker-compose build cart-api
	docker-compose up -d cart-api

rebuild-frontend:
	dokcer-compose build frontend
	docker-compose up -d frontend

health:
	echo "checking health"
	curl -s http://localhost:8084/health || echo "PRODUCT API DOWN"
	curl -s http://localhost:8081/health || echo "USER API DOWN"
	curl -s http://localhost:8085/health || echo "CART API DOWN"
	curl -s http://localhost:5173 < dev/null  && echo "Frontend: UP" || echo "Frontend: DOWN"


