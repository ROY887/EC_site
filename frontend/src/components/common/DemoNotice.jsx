// src/components/common/DemoNotice.jsx
import { AlertTriangle } from 'lucide-react';

/**
 * デモサイトであることを常時明示するバー。
 * 決済・注文確定が未実装であることを利用者に伝える。
 */
export default function DemoNotice() {
  return (
    <div className="bg-amber-100 border-b border-amber-300 text-amber-900">
      <div className="max-w-7xl mx-auto px-4 py-2 flex items-center gap-2 text-sm">
        <AlertTriangle size={16} className="shrink-0" />
        <p>
          <span className="font-bold">これはポートフォリオ用のデモサイトです。</span>
          {' '}商品はサンプルで、実際の販売・注文確定・決済は行いません。
          ご登録の際は<strong>普段お使いでないメールアドレスとパスワード</strong>をご利用ください。
        </p>
      </div>
    </div>
  );
}
