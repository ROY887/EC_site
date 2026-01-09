// src/context/CartContext.jsx
import { createContext, useContext, useState, useEffect } from 'react';
import { cartApi, productApi } from '../api';
import { useAuth } from './AuthContext';

const CartContext = createContext();

export function useCart() {
  const context = useContext(CartContext);
  if (!context) {
    throw new Error('useCart must be used within CartProvider');
  }
  return context;
}

export function CartProvider({ children }) {
  const { user, isAuthenticated } = useAuth();
  const [cartItems, setCartItems] = useState([]);
  const [loading, setLoading] = useState(false);

  // ユーザーが変わったらカートを読み込む
  useEffect(() => {
    if (isAuthenticated && user) {
      loadCart();
    } else {
      setCartItems([]);
    }
  }, [isAuthenticated, user]);

  // カート取得（商品情報付き）
  const loadCart = async () => {
    if (!user) return;
    
    try {
      setLoading(true);
      const items = await cartApi.get(user.id);
      
      // 各アイテムの商品情報を取得
      const itemsWithProducts = await Promise.all(
        items.map(async (item) => {
          try {
            const product = await productApi.getById(item.product_id);
            return {
              ...item,
              product: product,
              name: product.name,
              price: product.price,
              image_url: product.image_url,
            };
          } catch (error) {
            console.error('商品情報取得エラー:', error);
            return item;
          }
        })
      );
      
      setCartItems(itemsWithProducts);
    } catch (error) {
      console.error('カート読み込みエラー:', error);
    } finally {
      setLoading(false);
    }
  };

  // カートに追加
  const addToCart = async (product, quantity = 1) => {
    if (!user) {
      throw new Error('ログインが必要です');
    }

    try {
      await cartApi.add(user.id, product.id, quantity);
      await loadCart();
    } catch (error) {
      throw new Error(error.message || 'カートへの追加に失敗しました');
    }
  };

  // 数量更新
  const updateQuantity = async (productId, quantity) => {
    if (!user) return;

    try {
      if (quantity <= 0) {
        await removeFromCart(productId);
        return;
      }
      await cartApi.update(user.id, productId, quantity);
      await loadCart();
    } catch (error) {
      console.error('数量更新エラー:', error);
      throw error;
    }
  };

  // カートから削除
  const removeFromCart = async (productId) => {
    if (!user) return;

    try {
      await cartApi.remove(user.id, productId);
      await loadCart();
    } catch (error) {
      console.error('削除エラー:', error);
      throw error;
    }
  };

  // カート全削除
  const clearCart = async () => {
    if (!user) return;

    try {
      await cartApi.clear(user.id);
      setCartItems([]);
    } catch (error) {
      console.error('カートクリアエラー:', error);
      throw error;
    }
  };

  // 合計金額
  const cartTotal = cartItems.reduce(
    (sum, item) => sum + (item.price || 0) * item.quantity,
    0
  );

  // 合計数量
  const cartCount = cartItems.reduce((sum, item) => sum + item.quantity, 0);

  const value = {
    cartItems,
    loading,
    cartTotal,
    cartCount,
    addToCart,
    updateQuantity,
    removeFromCart,
    clearCart,
    refreshCart: loadCart,
  };

  return <CartContext.Provider value={value}>{children}</CartContext.Provider>;
}