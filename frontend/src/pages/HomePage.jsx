// src/pages/HomePage.jsx
import { useState, useEffect } from 'react';
import { Loader, AlertCircle } from 'lucide-react';
import { productApi } from '../api';
import ProductCard from '../components/product/ProductCard';

export default function HomePage({ searchQuery }) {
  const [products, setProducts] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);





  useEffect(() => {
    loadProducts();
  }, [searchQuery]);

  const loadProducts = async () => {
    try {
      setLoading(true);
      setError(null);
      
      let data;
      if (searchQuery) {
        data = await productApi.search(searchQuery);
      } else {
        data = await productApi.getAll();
      }

      console.log('API Response:', data)
      
      // データが配列でない場合は、配列に変換
      const productsData = Array.isArray(data) ? data : (data?.data || data?.products || []);
      
      setProducts(productsData);
    } catch (err) {
      setError(err.message || '商品の読み込みに失敗しました');
      console.error('商品読み込みエラー:', err);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <Loader className="animate-spin text-blue-600" size={48} />
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen flex items-center justify-center p-4">
        <div className="bg-red-50 border border-red-200 rounded-lg p-6 max-w-md">
          <div className="flex items-center gap-3 text-red-800 mb-2">
            <AlertCircle size={24} />
            <h2 className="font-bold text-lg">エラーが発生しました</h2>
          </div>
          <p className="text-red-700">{error}</p>
          <button
            onClick={loadProducts}
            className="mt-4 bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700 transition"
          >
            再試行
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-100">
      {/* ヒーローバナー */}
      {!searchQuery && (
        <div className="bg-gradient-to-r from-blue-600 to-purple-600 text-white py-12">
          <div className="max-w-7xl mx-auto px-4">
            <h1 className="text-4xl font-bold mb-3">年末セール開催中！</h1>
            <p className="text-xl">人気商品が最大50%オフ。お見逃しなく！</p>
          </div>
        </div>
      )}

      {/* メインコンテンツ */}
      <div className="max-w-7xl mx-auto px-4 py-8">
        {/* ヘッダー */}
        <div className="mb-6">
          <h2 className="text-2xl font-bold mb-2">
            {searchQuery ? `"${searchQuery}" の検索結果` : 'おすすめ商品'}
          </h2>
          <p className="text-gray-600">
            {products.length}件の商品
          </p>
        </div>

        {/* 商品グリッド */}
        {products.length > 0 ? (
          <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6">
            {products.map((product) => (
              <ProductCard key={product.id} product={product} />
            ))}
          </div>
        ) : (
          <div className="text-center py-16">
            <p className="text-gray-600 text-lg mb-4">
              {searchQuery ? '検索結果が見つかりませんでした' : '商品がありません'}
            </p>
            {searchQuery && (
              <button
                onClick={() => window.location.href = '/'}
                className="text-blue-600 hover:underline"
              >
                すべての商品を見る
              </button>
            )}
          </div>
        )}
      </div>
    </div>
  );
}



// import {
//   API_URLS,
//   REQUEST_TIMEOUT,
//   logApiCall,
//   logApiResponse,
//   logApiError
// } from './config';

// class ApiService {
//   constructor(baseURL){
//     this.baseURL = baseURL;
//     this.token =  null;
//     this.defaultHeaders = {
//       'Content-Type': 'application/json',
//     };
//   }  

//   setToken(token) {
//     this.token = token;
//     return this;
//   }

//   clearToken(token){
//     this.token =  null;
//     return this;
//   }

//   getHeaders(customHeaders = {}) {
//     const headers = {
//       ...this.defaultHeaders,
//       ...customHeaders,
//     };
//   }

//   async request(endpoint ,options = {}) {

//   }
//}