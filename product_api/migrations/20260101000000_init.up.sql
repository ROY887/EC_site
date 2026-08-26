-- product-api 初期スキーマ
-- 空の DB から再現できるよう CREATE TABLE から定義する。

CREATE TABLE IF NOT EXISTS products (
    id          UUID PRIMARY KEY,
    name        VARCHAR(255)   NOT NULL,
    description TEXT,
    price       NUMERIC(10, 2) NOT NULL,
    stock       INTEGER        NOT NULL DEFAULT 0,
    category    VARCHAR(100),
    image_url   TEXT,
    created_at  TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT products_price_check CHECK (price >= 0),
    CONSTRAINT products_stock_check CHECK (stock >= 0)
);

CREATE INDEX IF NOT EXISTS idx_products_category   ON products(category);
CREATE INDEX IF NOT EXISTS idx_products_created_at ON products(created_at DESC);

-- デモ用のサンプル商品。
-- 商品登録 API は公開していないため、データはここで投入する。
INSERT INTO products (id, name, description, price, stock, category, image_url) VALUES
    ('11111111-1111-4111-8111-111111111101', 'ワイヤレスノイズキャンセリングヘッドホン', '長時間の装着でも疲れにくい軽量設計。最大30時間再生。', 24800.00, 42, '家電',        '/images/product-1.svg'),
    ('11111111-1111-4111-8111-111111111102', 'メカニカルキーボード 91キー',             '静音赤軸。日本語配列でホットスワップ対応。',              13200.00, 18, 'コンピュータ', '/images/product-2.svg'),
    ('11111111-1111-4111-8111-111111111103', '4K モバイルモニター 15.6インチ',           'USB-C 一本で給電と映像出力。スタンドカバー付属。',        31900.00,  7, 'コンピュータ', '/images/product-3.svg'),
    ('11111111-1111-4111-8111-111111111104', 'ステンレス真空断熱タンブラー 470ml',       '結露しにくく保冷12時間。食洗機対応。',                    3480.00, 63, 'キッチン',     '/images/product-4.svg'),
    ('11111111-1111-4111-8111-111111111105', '全自動コーヒーメーカー',                   'ミル内蔵で豆から挽ける。タイマー予約対応。',              18700.00,  0, 'キッチン',     '/images/product-5.svg'),
    ('11111111-1111-4111-8111-111111111106', 'エルゴノミクスワイヤレスマウス',           '静音クリック。3台までマルチペアリング可能。',              6980.00, 25, 'コンピュータ', '/images/product-6.svg'),
    ('11111111-1111-4111-8111-111111111107', 'ロボット掃除機 マッピング機能付き',        '間取りを記憶して効率よく清掃。アプリから操作可能。',      54800.00,  4, '家電',        '/images/product-1.svg'),
    ('11111111-1111-4111-8111-111111111108', 'アロマディフューザー 超音波式',            '静音運転でライト機能付き。タンク容量300ml。',              4290.00, 31, '家電',        '/images/product-4.svg')
ON CONFLICT (id) DO NOTHING;
