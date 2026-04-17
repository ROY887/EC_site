// src/context/AuthContext.jsx
import { createContext, useContext, useState, useEffect } from 'react';
import { userApi } from '../api/';

const AuthContext = createContext();

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within AuthProvider');
  }
  return context;
}

export function AuthProvider({ children }) {
  const [user, setUser] = useState(null);
  const [loading, setLoading] = useState(true);

// アプリ起動時にlocalStorageからユーザー情報を復元する
  useEffect(() => {
    const savedUser = localStorage.getItem('user');
    if (savedUser) {
      try {
        setUser(JSON.parse(savedUser));
      } catch (error) {
        console.error('ユーザー情報の復元に失敗:', error);
        localStorage.removeItem('user');
      }
    }
    setLoading(false);
  }, []);

// ユーザー登録
  const signUp = async (username, email, password) => {
    try {
      const newUser = await userApi.signUp(username, email, password);
      setUser(newUser);
      localStorage.setItem('user', JSON.stringify(newUser));
      return newUser;
    } catch (error) {
      throw new Error(error.message || 'ユーザー登録に失敗しました');
    }
  };

// ログイン
  const login = async (email, password) => {
    try {
      const response = await userApi.login(email, password);
      const userData = response.user;
      setUser(userData);
      localStorage.setItem('user', JSON.stringify(userData));
      return userData;
    } catch (error) {
      throw new Error(error.message || 'ログインに失敗しました');
    }
  };

// ログアウト
  const logout = () => {
    setUser(null);
    localStorage.removeItem('user');
    userApi.logout();
  };

  const value = {
    user,
    loading,
    isAuthenticated: !!user,
    signUp,
    login,
    logout,
  };

  return (
    <AuthContext.Provider value={value}>
      {children}
    </AuthContext.Provider>
  );
}

