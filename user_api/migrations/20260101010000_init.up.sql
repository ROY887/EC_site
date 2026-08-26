-- user-api 初期スキーマ
-- 注意: 3 サービスは同じ ec_db を共有しており、sqlx の適用履歴は
-- _sqlx_migrations テーブル 1 つで管理される。バージョン番号が重複すると
-- 「適用済みだが内容が変更されている」と判定されて失敗するため、
-- サービスごとに番号をずらしてある (product=000000 / user=010000 / cart=020000)。

CREATE TABLE IF NOT EXISTS users (
    id            UUID PRIMARY KEY,
    username      VARCHAR(50)  NOT NULL,
    email         VARCHAR(254) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    created_at    TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at    TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at DESC);

-- updated_at の自動更新
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS update_users_updated_at ON users;
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
