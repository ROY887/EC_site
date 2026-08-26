// src/api/ApiService.js
// クラスベースAPIサービス

import {
  API_URLS,
  REQUEST_TIMEOUT,
  TOKEN_STORAGE_KEY,
  logApiCall,
  logApiResponse,
  logApiError,
} from './config';

/**
 * 認証トークンは複数のサービスインスタンスで共有する。
 * localStorage に保存し、リロード後も維持する。
 *
 * 注意: localStorage は XSS でトークンを読み出されうる。
 * 本番サービス化する際は httpOnly Cookie への移行を検討すること。
 */
const tokenStore = {
  get() {
    try {
      return localStorage.getItem(TOKEN_STORAGE_KEY);
    } catch {
      return null;
    }
  },
  set(token) {
    try {
      if (token) localStorage.setItem(TOKEN_STORAGE_KEY, token);
      else localStorage.removeItem(TOKEN_STORAGE_KEY);
    } catch {
      /* プライベートモード等で保存できない場合は無視 */
    }
  },
};

/** 401 を受け取ったときに呼ばれるハンドラ（AuthContext が登録する） */
let onUnauthorized = null;
export function setUnauthorizedHandler(fn) {
  onUnauthorized = fn;
}

// 基底APIサービスクラス
class ApiService {
  constructor(baseURL) {
    this.baseURL = baseURL;
    this.defaultHeaders = {
      'Content-Type': 'application/json',
    };
  }

  setToken(token) {
    tokenStore.set(token);
    return this;
  }

  clearToken() {
    tokenStore.set(null);
    return this;
  }

  getHeaders(customHeaders = {}) {
    const headers = {
      ...this.defaultHeaders,
      ...customHeaders,
    };

    const token = tokenStore.get();
    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    return headers;
  }

  async request(endpoint, options = {}) {
    const url = `${this.baseURL}${endpoint}`;
    const method = options.method || 'GET';

    logApiCall(method, url, options.body ? JSON.parse(options.body) : null);

    const config = {
      ...options,
      headers: this.getHeaders(options.headers),
      signal: AbortSignal.timeout(REQUEST_TIMEOUT),
    };

    try {
      const response = await fetch(url, config);

      const contentType = response.headers.get('content-type');
      let data;

      if (contentType && contentType.includes('application/json')) {
        data = await response.json();
      } else {
        data = await response.text();
      }

      if (!response.ok) {
        // バックエンドは { "error": "..." } 形式で返す
        const message =
          (data && typeof data === 'object' && data.error) ||
          (typeof data === 'string' && data) ||
          `HTTP ${response.status}: ${response.statusText}`;

        const error = {
          status: response.status,
          statusText: response.statusText,
          data,
          message,
          url,
          method,
        };

        // 認証切れは呼び出し側に伝播させつつ、セッションを破棄する
        if (response.status === 401) {
          tokenStore.set(null);
          if (onUnauthorized) onUnauthorized();
        }

        logApiError(method, url, error);
        throw error;
      }

      logApiResponse(method, url, data);
      return data;
    } catch (error) {
      if (error.name === 'TimeoutError') {
        const timeoutError = {
          status: 408,
          message: 'リクエストがタイムアウトしました',
          url,
          method,
          originalError: error,
        };
        logApiError(method, url, timeoutError);
        throw timeoutError;
      }

      if (error.name === 'TypeError') {
        const networkError = {
          status: 0,
          message: 'ネットワークエラー: サーバーに接続できません',
          url,
          method,
          originalError: error,
        };
        logApiError(method, url, networkError);
        throw networkError;
      }

      if (error.status !== undefined) {
        throw error;
      }

      const unexpectedError = {
        status: 500,
        message: error.message || '予期しないエラーが発生しました',
        url,
        method,
        originalError: error,
      };
      logApiError(method, url, unexpectedError);
      throw unexpectedError;
    }
  }

  get(endpoint, options = {}) {
    return this.request(endpoint, { method: 'GET', ...options });
  }

  post(endpoint, data, options = {}) {
    return this.request(endpoint, {
      method: 'POST',
      body: JSON.stringify(data),
      ...options,
    });
  }

  put(endpoint, data, options = {}) {
    return this.request(endpoint, {
      method: 'PUT',
      body: JSON.stringify(data),
      ...options,
    });
  }

  delete(endpoint, data = null, options = {}) {
    return this.request(endpoint, {
      method: 'DELETE',
      body: data ? JSON.stringify(data) : null,
      ...options,
    });
  }
}

// ========================================
// Product API Service（参照のみ公開）
// ========================================
class ProductService extends ApiService {
  constructor() {
    super(API_URLS.PRODUCT);
  }

  getAll = () => this.get('/products');

  getById = (id) => this.get(`/products/${id}`);

  getByCategory = (category) =>
    this.get(`/products/category/${encodeURIComponent(category)}`);

  search = (keyword) => this.get(`/products/search?q=${encodeURIComponent(keyword)}`);
}

// ========================================
// User API Service
// ========================================
class UserService extends ApiService {
  constructor() {
    super(API_URLS.USER);
  }

  signUp = (username, email, password) =>
    this.post('/users', { username, email, password });

  login = async (email, password) => {
    const response = await this.post('/login', { email, password });

    if (response.token) {
      this.setToken(response.token);
    }

    return response;
  };

  logout = () => {
    this.clearToken();
  };

  getById = (id) => this.get(`/users/${id}`);

  update = (id, updates) => this.put(`/users/${id}`, updates);

  delete = (id) => this.delete(`/users/${id}`);
}

// ========================================
// Cart API Service
//
// 対象ユーザーは JWT から導出されるため、user_id は送らない。
// ========================================
class CartService extends ApiService {
  constructor() {
    super(API_URLS.CART);
  }

  getCart = () => this.get('/cart');

  add = (productId, quantity = 1) =>
    this.post('/cart/add', { product_id: productId, quantity });

  update = (productId, quantity) =>
    this.put('/cart/update', { product_id: productId, quantity });

  remove = (productId) => this.delete('/cart/remove', { product_id: productId });

  clear = () => this.delete('/cart');
}

// シングルトンインスタンスをエクスポート
export const productApi = new ProductService();
export const userApi = new UserService();
export const cartApi = new CartService();

export default {
  product: productApi,
  user: userApi,
  cart: cartApi,
};
