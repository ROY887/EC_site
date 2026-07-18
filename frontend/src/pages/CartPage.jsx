// src/pages/CartPage.jsx
import { ShoppingCart, Trash2, Minus, Plus, Loader } from 'lucide-react';
import { useCart } from '../context/CartContext';
import { useAuth } from '../context/AuthContext';

export default function CartPage({ onClose }) {
  const { cartItems, cartTotal, cartCount, updateQuantity, removeFromCart, loading } = useCart();
  const { isAuthenticated } = useAuth();

  if (!isAuthenticated) {
    return (
      <div className="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center p-4">
        <div className="bg-white rounded-lg p-8 max-w-md w-full text-center">
          <h2 className="text-2xl font-bold mb-4">ログインが必要です</h2>
          <p className="text-gray-600 mb-6">
            カートを利用するにはログインしてください
          </p>
          <button
            onClick={onClose}
            className="bg-yellow-400 hover:bg-yellow-500 px-6 py-2 rounded-lg font-medium"
          >
            閉じる
          </button>
        </div>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center">
        <div className="bg-white rounded-lg p-8">
          <Loader className="animate-spin text-blue-600" size={48} />
        </div>
      </div>
    );
  }

  if (cartItems.length === 0) {
    return (
      <div className="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center p-4">
        <div className="bg-white rounded-lg p-8 max-w-md w-full">
          <div className="flex justify-between items-center mb-6">
            <h2 className="text-2xl font-bold">ショッピングカート</h2>
            <button
              onClick={onClose}
              className="text-gray-500 hover:text-gray-700 text-2xl leading-none"
            >
              
            </button>
          </div>
          <div className="text-center py-8">
            <ShoppingCart size={64} className="mx-auto text-gray-300 mb-4" />
            <p className="text-gray-600 mb-6">カートは空です</p>
            <button
              onClick={onClose}
              className="bg-yellow-400 hover:bg-yellow-500 px-6 py-2 rounded-lg font-medium"
            >
              買い物を続ける
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center p-4 overflow-y-auto">
      <div className="bg-white rounded-lg max-w-4xl w-full my-8">
        {/* ヘッダー */}
        <div className="p-6 border-b flex justify-between items-center">
          <h2 className="text-2xl font-bold">ショッピングカート</h2>
          <button
            onClick={onClose}
            className="text-gray-500 hover:text-gray-700 text-2xl leading-none"
          >
            
          </button>
        </div>

        {/* カートアイテム */}
        <div className="p-6 max-h-96 overflow-y-auto">
          {cartItems.map((item) => (
            <div key={item.product_id} className="flex gap-4 py-4 border-b last:border-b-0">
              {/* 商品画像 */}
              <img
                src={
                  item.image_url && !item.image_url.includes('via.placeholder.com')
                    ? item.image_url
                    : 'https://placehold.co/100x100'
                }
                alt={item.name}
                className="w-24 h-24 object-cover rounded-lg"
              />

              {/* 商品情報 */}
              <div className="flex-1">
                <h3 className="font-medium mb-1">{item.name}</h3>
                <p className="text-sm text-gray-600 mb-2">
                  ¥{item.price.toLocaleString()}
                </p>

                {/* 数量コントロール */}
                <div className="flex items-center gap-4">
                  <div className="flex items-center border rounded-lg">
                    <button
                      onClick={() => updateQuantity(item.product_id, item.quantity - 1)}
                      className="px-3 py-1 hover:bg-gray-100 transition"
                    >
                      <Minus size={16} />
                    </button>
                    <span className="px-4 py-1 border-x min-w-[3rem] text-center">
                      {item.quantity}
                    </span>
                    <button
                      onClick={() => updateQuantity(item.product_id, item.quantity + 1)}
                      className="px-3 py-1 hover:bg-gray-100 transition"
                    >
                      <Plus size={16} />
                    </button>
                  </div>

                  {/* 削除ボタン */}
                  <button
                    onClick={() => removeFromCart(item.product_id)}
                    className="text-red-600 hover:text-red-700 flex items-center gap-1 text-sm transition"
                  >
                    <Trash2 size={16} />
                    削除
                  </button>
                </div>
              </div>

              {/* 小計 */}
              <div className="text-right">
                <div className="text-xl font-bold">
                  ¥{(item.price * item.quantity).toLocaleString()}
                </div>
                <div className="text-sm text-gray-500">
                  ¥{item.price.toLocaleString()} × {item.quantity}
                </div>
              </div>
            </div>
          ))}
        </div>

        {/* フッター（合計と購入ボタン） */}
        <div className="p-6 bg-gray-50 border-t">
          <div className="flex justify-between items-center mb-6">
            <span className="text-lg font-medium">
              小計 ({cartCount}点):
            </span>
            <span className="text-3xl font-bold text-red-600">
              ¥{cartTotal.toLocaleString()}
            </span>
          </div>

          <button className="w-full bg-yellow-400 hover:bg-yellow-500 py-3 rounded-lg font-bold text-lg mb-3 transition">
            レジに進む
          </button>

          <button
            onClick={onClose}
            className="w-full border border-gray-300 hover:bg-gray-50 py-2 rounded-lg transition"
          >
            買い物を続ける
          </button>
        </div>
      </div>
    </div>
  );
}