-- Add down migration script here

ALTER TABLE products 
    ALTER COLUMN price TYPE INTEGER;

ALTER TABLE products
    DROP COLUMN IF EXISTS description,
    DROP COLUMN IF EXISTS stock,
    DROP COLUMN IF EXISTS category,
    DROP COLUMN IF EXISTS image_url,
    DROP COLUMN IF EXISTS created_at;

ALTER TABLE products
    DROP CONSTRAINT products_stock_check;

ALTER TABLE products
    DROP CONSTRAINT products_price_check;

DROP INDEX IF EXISTS idx_products_category;
DROP INDEX IF EXISTS idx_products_created_at;

UPDATE products
SET
    description IS NULL,
    stock IS NULL,
    category IS NULL,
    image_url IS NULL,
    created_at IS NULL

WHERE description = COALESCE
