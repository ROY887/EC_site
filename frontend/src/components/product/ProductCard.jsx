// src/components/product/ProductCard.jsx
import { useState } from 'react';
import { Star, ShoppingCart } from 'lucide-react';
import { useAuth } from '../../context/AuthContext';
import { useCart } from '../../context/CartContext';

export default function ProductCard({ product }) {
  const { isAuthenticated } = useAuth();
  const { addToCart } = useCart();
  const [isAdding, setIsAdding] = useState(false);
  const [error, setError] = useState(null);

  const handleAddToCart = async () => {
    if (!isAuthenticated) {
      alert('カートに追加するにはログインが必要です');
      return;
    }

    try {
      setIsAdding(true);
      setError(null);
      await addToCart(product, 1);
      setTimeout(() => setIsAdding(false), 1000);
    } catch (err) {
      setError('追加に失敗しました');
      setIsAdding(false);
    }
  };

  return (
    <div className="bg-white rounded-lg border border-gray-200 hover:shadow-xl transition-shadow duration-300 overflow-hidden group">
      {/* 商品画像 */}
      <div className="relative overflow-hidden bg-gray-100">
        <img
          src={product.image_url || 'https://via.placeholder.com/300'}
          alt={product.name}
          className="w-full h-56 object-cover group-hover:scale-110 transition-transform duration-300"
        />
        {product.stock !== undefined && product.stock < 10 && product.stock > 0 && (
          <div className="absolute top-2 right-2 bg-red-500 text-white px-2 py-1 rounded text-xs font-bold">
            残り{product.stock}点
          </div>
        )}
        {product.stock === 0 && (
          <div className="absolute inset-0 bg-black bg-opacity-50 flex items-center justify-center">
            <span className="text-white font-bold text-lg">在庫切れ</span>
          </div>
        )}
      </div>

      {/* 商品情報 */}
      <div className="p-4">
        {/* カテゴリ */}
        {product.category && (
          <div className="text-xs text-blue-600 font-semibold mb-1">
            {product.category}
          </div>
        )}

        {/* 商品名 */}
        <h3 className="font-medium text-sm mb-2 line-clamp-2 h-10 group-hover:text-blue-600 transition">
          {product.name}
        </h3>

        {/* 説明 */}
        {product.description && (
          <p className="text-xs text-gray-600 mb-2 line-clamp-2">
            {product.description}
          </p>
        )}

        {/* 評価 */}
        {product.rating && (
          <div className="flex items-center gap-2 mb-2">
            <div className="flex items-center text-orange-500">
              <Star size={16} fill="currentColor" />
              <span className="ml-1 text-sm font-bold">{product.rating}</span>
            </div>
            {product.review_count && (
              <span className="text-xs text-gray-500">({product.review_count})</span>
            )}
          </div>
        )}

        {/* 価格 */}
        <div className="flex items-baseline gap-2 mb-3">
          <span className="text-2xl font-bold text-gray-900">
            ¥{product.price.toLocaleString()}
          </span>
        </div>

        {/* エラーメッセージ */}
        {error && (
          <div className="text-xs text-red-600 mb-2">{error}</div>
        )}

        {/* カートに追加ボタン */}
        <button
          onClick={handleAddToCart}
          disabled={isAdding || product.stock === 0}
          className={`w-full py-2 rounded-lg font-medium transition-all flex items-center justify-center gap-2 ${
            isAdding
              ? 'bg-green-600 text-white'
              : product.stock === 0
              ? 'bg-gray-300 text-gray-500 cursor-not-allowed'
              : 'bg-yellow-400 hover:bg-yellow-500 text-gray-900 hover:shadow-md'
          }`}
        >
          <ShoppingCart size={18} />
          {isAdding ? 'カートに追加しました！' : product.stock === 0 ? '在庫切れ' : 'カートに入れる'}
        </button>
      </div>
    </div>
  );
}