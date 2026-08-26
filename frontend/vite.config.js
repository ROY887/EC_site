import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// ローカル開発では Vite の proxy が Gateway API の役割を担い、
// 本番と同じパス構成を再現する。
//   /api/products -> product-api:8084/products
//   /api/users    -> user-api:8081/users
//   /api/login    -> user-api:8081/login
//   /api/cart     -> cart-api:8085/cart
//
// 転送先は docker-compose で公開しているポートに合わせている。
const stripApi = (path) => path.replace(/^\/api/, '')

const proxyTo = (target) => ({
  target,
  changeOrigin: true,
  rewrite: stripApi,
})

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    host: true,
    port: 5173,
    proxy: {
      '/api/products': proxyTo('http://localhost:8084'),
      '/api/users': proxyTo('http://localhost:8081'),
      '/api/login': proxyTo('http://localhost:8081'),
      '/api/cart': proxyTo('http://localhost:8085'),
    },
  },
})
