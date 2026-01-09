-- Add down migration script here
ALTER TABLE cart_items
    DELETE COLUMN IF EXISTS created_at;

ALTER TABLE cart_items 
    DELETE COLUMN IF EXISTS updated_at;

