

export const API_URLS = {
    PRODUCT: import.meta.env.VITE_PRODUCT_API_URL || "http://0.0.0.0:8084",
    CART: import.meta.env.VITE_CART_API_URL || "http://0.0.0.0:8085",
    USER: import.meta.env.VITE_USER_API_URL || "http://0.0.0.0:8081",
};

export const isDevelopment = import.meta.env.DEV;
export const isProduction = import.meta.env.PROD;
export const mode = import.meta.env.MODE;

//デバッグモード
export const isDebug = import.meta.env.VITE_DEBUG == 'true';

//タイムアウト時間
export const REQUEST_TIMEOUT = 10000; //10秒

export function logApiCall(method, url, data = null){
    if(isDevelopment && isDebug){
        console.group('API call: ${method} ${data}');
        if(data) console.log('Request:',data);
        console.groupEnd();
    }
}

export function logApiResponse(method, url , response) {
    if(isDevelopment && isDebug){
        console.group('🟢 API Response: ${method} ${url}');
        console.log('Response:' ,response);
        console.groupEnd();
    }
}

export function logApiError(method, url, err){
    if(isDevelopment && isDebug){
        console.group('❌ API Error: ${method} ${url}');
        console.log('Error:', err);
        console.groupEnd();
    }
}

if (isDevelopment){
    console.log('🔧 API Configuration:');
    console.log('Product API:',API_URLS.PRODUCT);
    console.log(' User API:' ,API_URLS.USER);
    console.log(' CART API:' ,API_URLS.CART);
    console.log('Enviroment', mode);
    console.log(' Debug Mode:', isDebug);
}



// class ApiService {
//     constructor(baseURL) {
//         this.baseURL = baseURL;
//         this.token = null;
//         this.defaultHeaders = {
//             'Content-Type': 'application/json',
//         };
//     }

//     setToken(token){
//         this.token = token;
//         return this;
//     }

//     getHeaders(customHeaders = {}) {
//         const headers = {
//             ...this.defaultHeaders,
//             ...customHeaders,
//         };

//         if (this.token) {
//             headers['Aunthorization'] = `Bearer ${this.token}`;
//         }

//         return headers;
//     }
    
//     async request(endpoint, options = {}) {
//         const url = `${this.baseURL}${endpoint}`;
//         const method = options.method || `GET`;

//         logApiCall(method, url, options.body ? JSON.parse(options.body) : null);
        
//         const config = {
//             ...options,
//             headers: this.getHeaders(options.headers),
//             signal: AbortSignal.timeout(REQUEST_TIMEOUT),
//         };

//         try {
//             const response = await fetch(url,config);

//             const contentType = response.headers.get('content-type');
//             let data;

//             if (contextType && contentType.includes('application/json')) {
//                 data = await response.json();
//             } else {
//                 data = await response.text();
//             }

//             if (!response.ok) {
//                 const error = {
//                     status: response.status,
//                     statusText: response.status,
//                     data: data,
//                     message: data?.message || data || `HTTP ${response.status}: ${response.statusText}`,
//                     url: url,
//                     method: method,
//                 };

//                 logApiError(method, url,data);
//                 throw error;
//             }
//                 logApiResponse(method, url,data);
//                 return data;
//             } catch (error) {
//                 if (error.name === 'TimeoutError') {
//                     const networkError = {
//                         status: 0,
//                         message: 'ネットワークエラー: APIサーバーに接続できません',
//                         url: url,
//                         method: method,
//                         originalError: error,   
//                     };
//                     logApiError(method, url ,networkError);
//                     throw networkError;
//                 }

//                 if (error.status) {
//                     throw error;
//                 }

//                 const unexpectedError = {
//                     status: 500,
//                     message: error.message ||'予期しないエラーが発生しました',
//                     url: url,
//                     method: method,
//                     originalError: error,
//                 };
//                 logApiError(method, url, unexpectedError);
//                 throw unexpectedError;
//             }
//     } 

//     get(endpoint, options = {}) {
//         return this.request(endpoint, {
//             method: 'GET',
//             ...options,
//         })
//     }

//     post(endpoint,options =  {}) {
//         return this.request(endpoint, {
//             body: JSON.stringify(data),
//             ...options,
//         });
//     }
//     put(endpoint, options = {}) {
//         return this.request(endpoint, {
//             body: 
//         })
//     }   
//     }



        

export default function ProductCard({ product }) {
    
}