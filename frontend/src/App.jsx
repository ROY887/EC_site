// src/App.jsx
import { useState } from 'react';
import { AuthProvider } from './context/AuthContext';
import { CartProvider } from './context/CartContext';
import Header from './components/common/Header';
import HomePage from './pages/HomePage';
import LoginPage from './pages/LoginPage';
import CartPage from './pages/CartPage';

function App() {
  const [showLogin, setShowLogin] = useState(false);
  const [showCart, setShowCart] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');

  const handleSearch = (query) => {
    setSearchQuery(query);
  };

  const handleOpenLogin = () => {
    setShowLogin(true);
  };

  const handleCloseLogin = () => {
    setShowLogin(false);
  };

  const handleOpenCart = () => {
    setShowCart(true);
  };

  const handleCloseCart = () => {
    setShowCart(false);
  };

  return (
    <AuthProvider>
      <CartProvider>
        <div className="min-h-screen bg-gray-100">
          {/* ヘッダー */}
          <Header
            onSearch={handleSearch}
            onCartClick={handleOpenCart}
            onLoginClick={handleOpenLogin}
          />

          {/* メインコンテンツ */}
          <HomePage searchQuery={searchQuery} />

          {/* ログインモーダル */}
          {showLogin && (
            <LoginPage onClose={handleCloseLogin} />
          )}

          {/* カートモーダル */}
          {showCart && (
            <CartPage onClose={handleCloseCart} />
          )}
        </div>
      </CartProvider>
    </AuthProvider>
  );
}

export default App;

