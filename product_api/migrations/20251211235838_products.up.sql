-- Add missing columns to products table
-- Migration: 20240101000001_alter_products.sql
-- PostgreSQL 標準構文版

-- 1. price を integer から numeric(10,2) に変更
-- 既存データを保持しながら型変更
ALTER TABLE products 
  ALTER COLUMN price TYPE NUMERIC(10, 2) USING price::numeric;

-- 2. 不足しているカラムを追加
ALTER TABLE products 
  ADD COLUMN IF NOT EXISTS description TEXT;

ALTER TABLE products 
  ADD COLUMN IF NOT EXISTS stock INTEGER NOT NULL DEFAULT 0;

ALTER TABLE products 
  ADD COLUMN IF NOT EXISTS category VARCHAR(100);

ALTER TABLE products 
  ADD COLUMN IF NOT EXISTS image_url TEXT;

ALTER TABLE products 
  ADD COLUMN IF NOT EXISTS created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP;

-- 3. stock に CHECK 制約を追加（既に存在する場合はスキップ）
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint 
        WHERE conname = 'products_stock_check'
    ) THEN
        ALTER TABLE products 
          ADD CONSTRAINT products_stock_check CHECK (stock >= 0);
    END IF;
END $$;

-- 4. price に CHECK 制約を追加（既に存在する場合はスキップ）
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint 
        WHERE conname = 'products_price_check'
    ) THEN
        ALTER TABLE products 
          ADD CONSTRAINT products_price_check CHECK (price >= 0);
    END IF;
END $$;

-- 5. インデックスを追加（検索パフォーマンス向上）
CREATE INDEX IF NOT EXISTS idx_products_category ON products(category);
CREATE INDEX IF NOT EXISTS idx_products_created_at ON products(created_at DESC);

-- 6. 既存データに対してデフォルト値を設定
UPDATE products 
SET 
  description = COALESCE(description, '商品説明なし'),
  stock = COALESCE(stock, 0),
  category = COALESCE(category, 'その他'),
  image_url = COALESCE(image_url, 'https://via.placeholder.com/300'),
  created_at = COALESCE(created_at, CURRENT_TIMESTAMP)
WHERE description IS NULL 
   OR stock IS NULL 
   OR category IS NULL 
   OR image_url IS NULL 
   OR created_at IS NULL;