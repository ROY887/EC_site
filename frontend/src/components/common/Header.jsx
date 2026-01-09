// src/components/common/Header.jsx
import { useState } from 'react';
import { ShoppingCart, Search, User, LogOut } from 'lucide-react';
import { useAuth } from '../../context/AuthContext';
import { useCart } from '../../context/CartContext';

export default function Header({ onSearch, onCartClick, onLoginClick }) {
  const { user, isAuthenticated, logout } = useAuth();
  const { cartCount } = useCart();
  const [searchQuery, setSearchQuery] = useState('');

  const handleSearch = (e) => {
    e.preventDefault();
    if (onSearch) {
      onSearch(searchQuery);
    }
  };

  const handleLogout = () => {
    logout();
    window.location.href = '/';
  };

  return (
    <header className="bg-gray-900 text-white sticky top-0 z-50 shadow-lg">
      {/* メインヘッダー */}
      <div className="bg-gray-800 py-3 px-4">
        <div className="max-w-7xl mx-auto flex items-center justify-between gap-4">
          {/* ロゴ */}
          <div className="flex items-center gap-6">
            <a href="/" className="text-2xl font-bold hover:text-orange-400 transition">
              <span className="text-orange-400">ama</span>zon.jp
            </a>
          </div>

          {/* 検索バー */}
          <form onSubmit={handleSearch} className="flex-1 max-w-2xl">
            <div className="flex">
              <select className="bg-gray-200 text-gray-800 px-3 rounded-l border-r border-gray-400 text-sm focus:outline-none">
                <option>すべて</option>
                <option>家電</option>
                <option>コンピュータ</option>
                <option>キッチン</option>
              </select>
              <input
                type="text"
                placeholder="商品を検索"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="flex-1 px-4 py-2 text-gray-900 focus:outline-none"
              />
              <button
                type="submit"
                className="bg-orange-400 px-4 hover:bg-orange-500 transition"
              >
                <Search size={20} />
              </button>
            </div>
          </form>

          {/* 右側メニュー */}
          <div className="flex items-center gap-4">
            {/* ユーザーメニュー */}
            {isAuthenticated ? (
              <div className="relative group">
                <button className="flex items-center gap-2 hover:border border-white p-2 rounded transition">
                  <User size={20} />
                  <div className="text-left">
                    <div className="text-xs">こんにちは</div>
                    <div className="font-bold">{user?.username || user?.email}</div>
                  </div>
                </button>
                {/* ドロップダウン */}
                <div className="absolute right-0 mt-1 w-48 bg-white text-gray-900 rounded shadow-lg opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all">
                  <button
                    onClick={handleLogout}
                    className="w-full px-4 py-2 hover:bg-gray-100 flex items-center gap-2 text-left"
                  >
                    <LogOut size={16} />
                    ログアウト
                  </button>
                </div>
              </div>
            ) : (
              <button
                onClick={onLoginClick}
                className="hover:border border-white p-2 rounded transition"
              >
                <div className="text-xs">こんにちは、ゲスト</div>
                <div className="font-bold">ログイン</div>
              </button>
            )}

            {/* カート */}
            <button
              onClick={onCartClick}
              className="relative hover:border border-white p-2 rounded transition flex items-center gap-2"
            >
              <div className="relative">
                <ShoppingCart size={32} />
                {cartCount > 0 && (
                  <span className="absolute -top-2 -right-2 bg-orange-500 text-white rounded-full w-6 h-6 flex items-center justify-center text-xs font-bold">
                    {cartCount}
                  </span>
                )}
              </div>
              <span className="font-bold hidden sm:inline">カート</span>
            </button>
          </div>
        </div>
      </div>

      {/* サブナビゲーション */}
      <div className="bg-gray-700 px-4 py-2">
        <div className="max-w-7xl mx-auto flex items-center gap-4 text-sm">
          <a href="/" className="hover:border border-white p-1 rounded transition">
            ホーム
          </a>
          <a href="#" className="hover:border border-white p-1 rounded transition">
            タイムセール
          </a>
          <a href="#" className="hover:border border-white p-1 rounded transition">
            新着商品
          </a>
          <a href="#" className="hover:border border-white p-1 rounded transition">
            ランキング
          </a>
        </div>
      </div>
    </header>
  );
}