-- cart-api 初期スキーマ
-- 注意: 3 サービスは同じ ec_db を共有しており、sqlx の適用履歴は
-- _sqlx_migrations テーブル 1 つで管理される。バージョン番号が重複すると
-- 「適用済みだが内容が変更されている」と判定されて失敗するため、
-- サービスごとに番号をずらしてある (product=000000 / user=010000 / cart=020000)。

-- gen_random_uuid() のために pgcrypto を有効化（PostgreSQL 13+ では標準で利用可）
CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS cart_items (
    id         UUID PRIMARY KEY,
    -- user_id / product_id は別サービスが所有するため外部キー制約は張らない
    user_id    UUID    NOT NULL,
    product_id UUID    NOT NULL,
    quantity   INTEGER NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT cart_items_quantity_check CHECK (quantity > 0),
    -- 同一ユーザーが同じ商品を複数行持たないことを保証する（upsert の前提）
    CONSTRAINT cart_items_user_product_unique UNIQUE (user_id, product_id)
);

CREATE INDEX IF NOT EXISTS idx_cart_items_user_id    ON cart_items(user_id);
CREATE INDEX IF NOT EXISTS idx_cart_items_product_id ON cart_items(product_id);

CREATE OR REPLACE FUNCTION update_cart_items_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS update_cart_items_updated_at ON cart_items;
CREATE TRIGGER update_cart_items_updated_at
    BEFORE UPDATE ON cart_items
    FOR EACH ROW
    EXECUTE FUNCTION update_cart_items_updated_at();
