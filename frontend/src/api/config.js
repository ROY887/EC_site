

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


