// src/api/config.js
//
// フロントエンドと API は同一オリジンで配信される。
//   /            -> frontend (nginx)
//   /api/*       -> Gateway が各サービスへ振り分け
// このため絶対 URL を持たず、常に相対パスを使う。
//
// 利点:
//   - CORS が不要（同一オリジン）
//   - Vite の環境変数はビルド時に埋め込まれるが、URL を持たないので
//     イメージを環境ごとに作り分ける必要がない
//   - 公開ドメインの変更がフロントエンドに影響しない

export const API_BASE = import.meta.env.VITE_API_BASE || '/api';

export const API_URLS = {
    PRODUCT: API_BASE,
    CART: API_BASE,
    USER: API_BASE,
};

export const isDevelopment = import.meta.env.DEV;
export const isProduction = import.meta.env.PROD;
export const mode = import.meta.env.MODE;

// デバッグモード
export const isDebug = import.meta.env.VITE_DEBUG === 'true';

// タイムアウト時間
export const REQUEST_TIMEOUT = 10000; // 10秒

// 認証トークンの保存キー
export const TOKEN_STORAGE_KEY = 'ec_site_token';
export const USER_STORAGE_KEY = 'ec_site_user';

export function logApiCall(method, url, data = null) {
    if (isDevelopment && isDebug) {
        console.group(`API Call: ${method} ${url}`);
        if (data) console.log('Request:', data);
        console.groupEnd();
    }
}

export function logApiResponse(method, url, response) {
    if (isDevelopment && isDebug) {
        console.group(`🟢 API Response: ${method} ${url}`);
        console.log('Response:', response);
        console.groupEnd();
    }
}

export function logApiError(method, url, err) {
    if (isDevelopment && isDebug) {
        console.group(`❌ API Error: ${method} ${url}`);
        console.log('Error:', err);
        console.groupEnd();
    }
}

if (isDevelopment) {
    console.log('🔧 API Configuration:');
    console.log('  API Base:', API_BASE);
    console.log('  Environment:', mode);
    console.log('  Debug Mode:', isDebug);
}
