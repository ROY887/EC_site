-- Add missing columns and constraints to cart_items table
-- Migration UP: 20240101000001_alter_cart_items.up.sql
-- PostgreSQL 標準構文版

-- 1. created_at カラムを追加
ALTER TABLE cart_items 
  ADD COLUMN IF NOT EXISTS created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP;

-- 2. updated_at カラムを追加
ALTER TABLE cart_items 
  ADD COLUMN IF NOT EXISTS updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP;

-- 3. 重複防止のための UNIQUE 制約を追加
-- 同じユーザーが同じ商品を複数のカートアイテムとして持てないようにする
DO $$ 
BEGIN
    IF NOT EXISTS ( 
        SELECT 1 FROM pg_constraint 
        WHERE conname = 'cart_items_user_product_unique'
    ) THEN
        ALTER TABLE cart_items 
          ADD CONSTRAINT cart_items_user_product_unique 
          UNIQUE (user_id, product_id);
    END IF;
END $$;

-- 4. インデックスを追加（既に存在する可能性があるため IF NOT EXISTS を使用）
CREATE INDEX IF NOT EXISTS idx_cart_items_user_id ON cart_items(user_id);
CREATE INDEX IF NOT EXISTS idx_cart_items_product_id ON cart_items(product_id);
CREATE INDEX IF NOT EXISTS idx_cart_items_created_at ON cart_items(created_at DESC);

-- 5. 既存データに対してタイムスタンプを設定
UPDATE cart_items 
SET 
  created_at = COALESCE(created_at, CURRENT_TIMESTAMP),
  updated_at = COALESCE(updated_at, CURRENT_TIMESTAMP)
WHERE created_at IS NULL OR updated_at IS NULL;

-- 6. updated_at 自動更新用のトリガー関数を作成
CREATE OR REPLACE FUNCTION update_cart_items_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- 7. トリガーを設定
DROP TRIGGER IF EXISTS update_cart_items_updated_at ON cart_items;
CREATE TRIGGER update_cart_items_updated_at
    BEFORE UPDATE ON cart_items
    FOR EACH ROW
    EXECUTE FUNCTION update_cart_items_updated_at();

-- 8. 重複データがある場合の処理（念のため）
-- 重複がある場合は quantity を合算して1つにまとめる
WITH duplicates AS (
    SELECT 
        user_id, 
        product_id,
        array_agg(id ORDER BY created_at) as ids,
        SUM(quantity) as total_quantity
    FROM cart_items
    GROUP BY user_id, product_id
    HAVING COUNT(*) > 1
)
UPDATE cart_items c
SET quantity = d.total_quantity
FROM duplicates d
WHERE c.id = d.ids[1];

-- 9. 重複の2つ目以降を削除
WITH duplicates AS (
    SELECT 
        user_id, 
        product_id,
        array_agg(id ORDER BY created_at) as ids
    FROM cart_items
    GROUP BY user_id, product_id
    HAVING COUNT(*) > 1
)
DELETE FROM cart_items c
USING duplicates d
WHERE c.id = ANY(d.ids[2:]);





