# --- ビルドステージ ---
FROM node:22-alpine AS builder

WORKDIR /app

COPY package*.json ./
RUN npm ci

COPY . .

# 静的ファイルを生成する（開発サーバーは本番では使わない）
RUN npm run build

# --- 配信ステージ ---
# nginx-unprivileged は非 root（uid 101）で動作し、8080 を listen する。
# k8s の runAsNonRoot / readOnlyRootFilesystem と組み合わせられる。
FROM nginxinc/nginx-unprivileged:1.27-alpine

COPY --chown=nginx:nginx nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder --chown=nginx:nginx /app/dist /usr/share/nginx/html

EXPOSE 8080

CMD ["nginx", "-g", "daemon off;"]
