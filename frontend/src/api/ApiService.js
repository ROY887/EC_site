// src/api/ApiService.js
// クラスベースAPIサービス

import { 
  API_URLS, 
  REQUEST_TIMEOUT, 
  logApiCall, 
  logApiResponse, 
  logApiError 
} from './config';




 // 基底APIサービスクラス
class ApiService {
  constructor(baseURL) {
    this.baseURL = baseURL;
    this.token = null;
    this.defaultHeaders = {
      'Content-Type': 'application/json',
    };
  }



  
  //認証トークンを設定
  setToken(token) {
    this.token = token;
    return this;
  }

  
   
  // 認証トークンをクリア
  clearToken() {
    this.token = null;
    return this;
  }

  
   //ヘッダーを取得
  getHeaders(customHeaders = {}) {
    const headers = {
      ...this.defaultHeaders,
      ...customHeaders,
    };

    if (this.token) {
      headers['Authorization'] = `Bearer ${this.token}`;
    }

    return headers;
  }

  
   // 汎用リクエストメソッド
   
  async request(endpoint, options = {}) {
    const url = `${this.baseURL}${endpoint}`;
    const method = options.method || 'GET';
    
    // リクエストログ
    logApiCall(method, url, options.body ? JSON.parse(options.body) : null);

    const config = {
      ...options,
      headers: this.getHeaders(options.headers),
      signal: AbortSignal.timeout(REQUEST_TIMEOUT),
    };

    try {
      const response = await fetch(url, config);

      // レスポンスの処理
      const contentType = response.headers.get('content-type');
      let data;

      if (contentType && contentType.includes('application/json')) {
        data = await response.json();
      } else {
        data = await response.text();
      }

      // エラーレスポンスの処理
      if (!response.ok) {
        const error = {
          status: response.status,
          statusText: response.statusText,
          data: data,
          message: data?.message || data || `HTTP ${response.status}: ${response.statusText}`,
          url: url,
          method: method,
        };
        
        logApiError(method, url, error);
        throw error;
      }

      // 成功ログ
      logApiResponse(method, url, data);
      
      return data;
    } catch (error) {
      // タイムアウトエラー
      if (error.name === 'TimeoutError') {
        const timeoutError = {
          status: 408,
          message: 'リクエストがタイムアウトしました',
          url: url,
          method: method,
          originalError: error,
        };
        logApiError(method, url, timeoutError);
        throw timeoutError;
      }

      // ネットワークエラー
      if (error.name === 'TypeError' && error.message.includes('fetch')) {
        const networkError = {
          status: 0,
          message: 'ネットワークエラー: APIサーバーに接続できません',
          url: url,
          method: method,
          originalError: error,
        };
        logApiError(method, url, networkError);
        throw networkError;
      }

      // その他のエラー（既にフォーマット済み）
      if (error.status) {
        throw error;
      }

      // 予期しないエラー
      const unexpectedError = {
        status: 500,
        message: error.message || '予期しないエラーが発生しました',
        url: url,
        method: method,
        originalError: error,
      };
      logApiError(method, url, unexpectedError);
      throw unexpectedError;
    }
  }

  
   // GET リクエスト
   
  get(endpoint, options = {}) {
    return this.request(endpoint, {
      method: 'GET',
      ...options,
    });
  }

  
  //  POST リクエスト
   
  post(endpoint, data, options = {}) {
    return this.request(endpoint, {
      method: 'POST',
      body: JSON.stringify(data),
      ...options,
    });
  }

   // PUT リクエスト
   
  put(endpoint, data, options = {}) {
    return this.request(endpoint, {
      method: 'PUT',
      body: JSON.stringify(data),
      ...options,
    });
  }

  
  //  DELETE リクエスト
   
  delete(endpoint, data = null, options = {}) {
    return this.request(endpoint, {
      method: 'DELETE',
      body: data ? JSON.stringify(data) : null,
      ...options,
    });
  }
}

// ========================================
// Product API Service
// ========================================
class ProductService extends ApiService {
  constructor() {
    super(API_URLS.PRODUCT);
  }

  // ヘルスチェック
  healthCheck = () => this.get('/health');

  // 商品一覧取得
  getAll = () => this.get('/products');

  // 商品詳細取得
  getById = (id) => this.get(`/products/${id}`);

  // カテゴリで検索
  getByCategory = (category) => this.get(`/products/category/${category}`);

  // キーワード検索
  search = (keyword) => this.get(`/products/search?q=${encodeURIComponent(keyword)}`);

  // 商品作成（管理者用）
  create = (productData) => this.post('/products', productData);

  // 商品更新（管理者用）
  update = (id, updates) => this.put(`/products/${id}`, updates);

  // 商品削除（管理者用）
  delete = (id) => this.delete(`/products/${id}`);
}

// ========================================
// User API Service
// ========================================
class UserService extends ApiService {
  constructor() {
    super(API_URLS.USER);
  }

  // ヘルスチェック
  healthCheck = () => this.get('/health');

  // ユーザー登録
  signUp = async (username, email, password) => {
    const response = await this.post('/users', { username, email, password });
    return response;
  };

  // ログイン
  login = async (email, password) => {
    const response = await this.post('/login', { email, password });
    
    // ログイン成功時の処理（トークンがあれば設定）
    // Rustバックエンドのレスポンス構造に合わせて調整してください
    // if (response.token) {
    //   this.setToken(response.token);
    // }
    
    return response;
  };

  // ログアウト
  logout = () => {
    this.clearToken();
  };

  // ユーザー情報取得
  getById = (id) => this.get(`/users/${id}`);

  // 全ユーザー取得（開発用）
  getAll = () => this.get('/users');

  // ユーザー情報更新
  update = (id, updates) => this.put(`/users/${id}`, updates);

  // ユーザー削除
  delete = (id) => this.delete(`/users/${id}`);
}

// Cart API Service
class CartService extends ApiService {
  constructor() {
    super(API_URLS.CART);
  }

  // ヘルスチェック
  healthCheck = () => this.get('/health');

  // カートに商品を追加
  add = (userId, productId, quantity = 1) => 
    this.post('/cart/add', {
      id: null, // バックエンドで自動生成
      user_id: userId,
      product_id: productId,
      quantity: quantity,
    });

  // カート取得
  getUser = (userId) => this.get(`/cart/${userId}`);

  // 商品数量を更新
  update = (userId, productId, quantity) => 
    this.put('/cart/update', {
      user_id: userId,
      product_id: productId,
      quantity: quantity,
    });

  // カートから商品を削除
  remove = (userId, productId) => 
    this.delete('/cart/remove', {
      user_id: userId,
      product_id: productId,
    });

  // カートを全削除
  clear = (userId) => this.delete(`/cart/${userId}`);

}

// シングルトンインスタンスをエクスポート
export const productApi = new ProductService();
export const userApi = new UserService();
export const cartApi = new CartService();

// デフォルトエクスポート
export default {
  product: productApi,
  user: userApi,
  cart: cartApi,
};








