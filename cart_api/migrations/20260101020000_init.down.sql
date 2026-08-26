DROP TRIGGER IF EXISTS update_cart_items_updated_at ON cart_items;
DROP FUNCTION IF EXISTS update_cart_items_updated_at();
DROP INDEX IF EXISTS idx_cart_items_product_id;
DROP INDEX IF EXISTS idx_cart_items_user_id;
DROP TABLE IF EXISTS cart_items;
