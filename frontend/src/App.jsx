// src/App.jsx
import { useState } from 'react';
import { AuthProvider } from './context/AuthContext';
import { CartProvider } from './context/CartContext';
import Header from './components/common/Header';
import DemoNotice from './components/common/DemoNotice';
import Footer from './components/common/Footer';
import HomePage from './pages/HomePage';
import LoginPage from './pages/LoginPage';
import CartPage from './pages/CartPage';
import PrivacyPage from './pages/PrivacyPage';

function App() {
  const [showLogin, setShowLogin] = useState(false);
  const [showCart, setShowCart] = useState(false);
  const [showPrivacy, setShowPrivacy] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');

  return (
    <AuthProvider>
      <CartProvider>
        <div className="min-h-screen bg-gray-100 flex flex-col">
          {/* デモサイトであることの明示 */}
          <DemoNotice />

          <Header
            onSearch={setSearchQuery}
            onCartClick={() => setShowCart(true)}
            onLoginClick={() => setShowLogin(true)}
          />

          <main className="flex-1">
            <HomePage searchQuery={searchQuery} />
          </main>

          <Footer onPrivacyClick={() => setShowPrivacy(true)} />

          {showLogin && <LoginPage onClose={() => setShowLogin(false)} />}
          {showCart && <CartPage onClose={() => setShowCart(false)} />}
          {showPrivacy && <PrivacyPage onClose={() => setShowPrivacy(false)} />}
        </div>
      </CartProvider>
    </AuthProvider>
  );
}

export default App;
