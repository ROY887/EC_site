// src/context/CartContext.jsx
import { createContext, useContext, useState, useEffect, useCallback } from 'react';
import { cartApi, productApi } from '../api';
import { useAuth } from './AuthContext';

const CartContext = createContext();

// eslint-disable-next-line react-refresh/only-export-components
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

  // カート取得（商品情報を付与する）
  const loadCart = useCallback(async () => {
    if (!isAuthenticated) return;

    try {
      setLoading(true);
      const items = await cartApi.getCart();

      // 各アイテムの商品情報を取得する。
      // 商品が削除済みでもカート全体が壊れないよう個別に握りつぶす。
      const itemsWithProducts = await Promise.all(
        items.map(async (item) => {
          try {
            const product = await productApi.getById(item.product_id);
            return {
              ...item,
              product,
              name: product.name,
              price: Number(product.price) || 0,
              image_url: product.image_url,
            };
          } catch (error) {
            console.error('商品情報取得エラー:', error);
            return { ...item, name: '(商品情報を取得できません)', price: 0 };
          }
        })
      );

      setCartItems(itemsWithProducts);
    } catch (error) {
      console.error('カート読み込みエラー:', error);
    } finally {
      setLoading(false);
    }
  }, [isAuthenticated]);

  // ログイン状態が変わったらカートを読み込む
  useEffect(() => {
    if (isAuthenticated && user) {
      loadCart();
    } else {
      setCartItems([]);
    }
  }, [isAuthenticated, user, loadCart]);

  const addToCart = async (product, quantity = 1) => {
    if (!isAuthenticated) {
      throw new Error('ログインが必要です');
    }

    try {
      await cartApi.add(product.id, quantity);
      await loadCart();
    } catch (error) {
      throw new Error(error.message || 'カートへの追加に失敗しました');
    }
  };

  const updateQuantity = async (productId, quantity) => {
    if (!isAuthenticated) return;

    try {
      if (quantity <= 0) {
        await removeFromCart(productId);
        return;
      }
      await cartApi.update(productId, quantity);
      await loadCart();
    } catch (error) {
      console.error('数量更新エラー:', error);
      throw error;
    }
  };

  const removeFromCart = async (productId) => {
    if (!isAuthenticated) return;

    try {
      await cartApi.remove(productId);
      await loadCart();
    } catch (error) {
      console.error('削除エラー:', error);
      throw error;
    }
  };

  const clearCart = async () => {
    if (!isAuthenticated) return;

    try {
      await cartApi.clear();
      setCartItems([]);
    } catch (error) {
      console.error('カートクリアエラー:', error);
      throw error;
    }
  };

  // 合計金額
  const cartTotal = cartItems.reduce(
    (sum, item) => sum + (Number(item.price) || 0) * item.quantity,
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
