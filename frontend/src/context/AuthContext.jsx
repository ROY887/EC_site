// src/context/AuthContext.jsx
import { createContext, useContext, useState, useEffect, useCallback } from 'react';
import { userApi } from '../api/';
import { setUnauthorizedHandler } from '../api/ApiService';
import { USER_STORAGE_KEY } from '../api/config';

const AuthContext = createContext();

// Provider と同じファイルに置くため Fast Refresh の警告が出るが、
// 呼び出し側の import を 1 箇所にまとめる利点を優先する。
// eslint-disable-next-line react-refresh/only-export-components
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

  const clearSession = useCallback(() => {
    setUser(null);
    localStorage.removeItem(USER_STORAGE_KEY);
    userApi.logout();
  }, []);

  // アプリ起動時に localStorage からユーザー情報を復元する
  useEffect(() => {
    const savedUser = localStorage.getItem(USER_STORAGE_KEY);
    if (savedUser) {
      try {
        setUser(JSON.parse(savedUser));
      } catch (error) {
        console.error('ユーザー情報の復元に失敗:', error);
        localStorage.removeItem(USER_STORAGE_KEY);
      }
    }
    setLoading(false);
  }, []);

  // トークン期限切れ（401）を検知したらセッションを破棄する
  useEffect(() => {
    setUnauthorizedHandler(() => clearSession());
    return () => setUnauthorizedHandler(null);
  }, [clearSession]);

  // ユーザー登録（登録後そのままログインしてトークンを取得する）
  const signUp = async (username, email, password) => {
    try {
      await userApi.signUp(username, email, password);
      return await login(email, password);
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
      localStorage.setItem(USER_STORAGE_KEY, JSON.stringify(userData));
      return userData;
    } catch (error) {
      throw new Error(error.message || 'ログインに失敗しました');
    }
  };

  // ログアウト
  const logout = () => {
    clearSession();
  };

  const value = {
    user,
    loading,
    isAuthenticated: !!user,
    signUp,
    login,
    logout,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}
