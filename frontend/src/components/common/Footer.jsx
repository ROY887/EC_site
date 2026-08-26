// src/components/common/Footer.jsx
import { Github } from 'lucide-react';

export default function Footer({ onPrivacyClick }) {
  return (
    <footer className="bg-gray-900 text-gray-300 mt-16">
      <div className="max-w-7xl mx-auto px-4 py-10">
        <div className="grid gap-8 md:grid-cols-3">
          <div>
            <h3 className="text-white font-bold mb-3">このサイトについて</h3>
            <p className="text-sm leading-relaxed">
              Rust (axum) によるマイクロサービスを Kubernetes 上で動かす構成の
              学習・検証を目的としたデモサイトです。実際の商取引は行いません。
            </p>
          </div>

          <div>
            <h3 className="text-white font-bold mb-3">実装状況</h3>
            <ul className="text-sm space-y-1">
              <li>✅ 商品の一覧・検索</li>
              <li>✅ ユーザー登録・ログイン</li>
              <li>✅ カート操作</li>
              <li className="text-gray-500">❌ 注文確定・決済（未実装）</li>
              <li className="text-gray-500">❌ 配送・在庫引当（未実装）</li>
            </ul>
          </div>

          <div>
            <h3 className="text-white font-bold mb-3">情報の取り扱い</h3>
            <button
              onClick={onPrivacyClick}
              className="text-sm underline hover:text-white transition"
            >
              プライバシーポリシー
            </button>
            <p className="text-sm mt-3 flex items-center gap-2">
              <Github size={16} />
              ソースコードは公開リポジトリで閲覧できます
            </p>
          </div>
        </div>

        <div className="border-t border-gray-700 mt-8 pt-6 text-xs text-gray-500">
          デモサイト / 実在の企業・サービスとは関係ありません
        </div>
      </div>
    </footer>
  );
}
