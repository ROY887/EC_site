// src/api/index.js
// APIサービスの統一エクスポートポイント

// 個別エクスポート（推奨）
export { productApi, userApi, cartApi } from './ApiService';

// 設定のエクスポート
export { API_URLS, isDevelopment, isProduction } from './config';

// デフォルトエクスポート
import api from './ApiService';
export default api;



// 使用例：
// import { productApi, userApi, cartApi } from '@/api';
// または
// import api from '@/api';
// api.product.getAll()