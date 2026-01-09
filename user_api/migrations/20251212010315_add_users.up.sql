-- Add up migration script here

-- Add missing columns to users table
-- Migration UP: 20240101000001_alter_users.up.sql
-- PostgreSQL 標準構文版

-- 1. created_at カラムを追加
ALTER TABLE users 
  ADD COLUMN IF NOT EXISTS created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP;

-- 2. updated_at カラムを追加（更新日時追跡用）
ALTER TABLE users 
  ADD COLUMN IF NOT EXISTS updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP;

-- 3. インデックスを追加
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at DESC);

-- 4. email のインデックスが既にあることを確認（あれば何もしない）
-- UNIQUE制約は既にあるのでスキップ

-- 5. 既存ユーザーに対してタイムスタンプを設定
UPDATE users 
SET 
  created_at = COALESCE(created_at, CURRENT_TIMESTAMP),
  updated_at = COALESCE(updated_at, CURRENT_TIMESTAMP)
WHERE created_at IS NULL OR updated_at IS NULL;

-- 6. updated_at 自動更新用のトリガー関数を作成
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- 7. トリガーを設定
DROP TRIGGER IF EXISTS update_users_updated_at ON users;
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();