// src/pages/PrivacyPage.jsx

/**
 * プライバシーポリシー。
 * デモであっても、メールアドレスとパスワードを預かる以上は掲示が必要。
 */
export default function PrivacyPage({ onClose }) {
  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center p-4">
      <div className="bg-white rounded-lg shadow-xl max-w-2xl w-full max-h-[85vh] flex flex-col">
        <div className="flex justify-between items-center p-6 border-b shrink-0">
          <h2 className="text-2xl font-bold">プライバシーポリシー</h2>
          <button
            onClick={onClose}
            aria-label="閉じる"
            className="text-gray-500 hover:text-gray-700 text-3xl leading-none px-2"
          >
            ×
          </button>
        </div>

        <div className="p-6 overflow-y-auto text-sm leading-relaxed space-y-5">
          <section>
            <h3 className="font-bold text-base mb-1">1. 本サイトの位置づけ</h3>
            <p>
              本サイトは、Web アプリケーションおよびインフラ構成の学習・技術検証を目的とした
              個人のデモサイトです。掲載している商品はすべてサンプルであり、
              実際の販売、注文の確定、決済、配送は一切行いません。
            </p>
          </section>

          <section>
            <h3 className="font-bold text-base mb-1">2. 取得する情報</h3>
            <p>会員登録をされた場合、以下の情報を取得・保存します。</p>
            <ul className="list-disc list-inside mt-2 space-y-1">
              <li>ユーザー名</li>
              <li>メールアドレス</li>
              <li>パスワード（bcrypt によりハッシュ化して保存し、元の文字列は保持しません）</li>
              <li>カートに追加した商品と数量</li>
              <li>アクセスログ（リクエスト日時、パス、応答時間、エラー情報）</li>
            </ul>
            <p className="mt-2">
              クレジットカード情報、住所、電話番号など、上記以外の個人情報は取得しません。
            </p>
          </section>

          <section>
            <h3 className="font-bold text-base mb-1">3. 利用目的</h3>
            <p>
              取得した情報は、ログイン状態の維持およびカート機能の提供のみに利用します。
              広告配信、第三者への提供・販売は行いません。
            </p>
          </section>

          <section>
            <h3 className="font-bold text-base mb-1">4. Cookie / ローカルストレージ</h3>
            <p>
              ログイン状態を保持するため、ブラウザのローカルストレージに認証トークンと
              ユーザー情報を保存します。ログアウト時に削除されます。
              アクセス解析ツールや広告用のトラッキングは使用していません。
            </p>
          </section>

          <section>
            <h3 className="font-bold text-base mb-1">5. 保存期間と削除</h3>
            <p>
              アカウント情報は削除操作を行うまで保存されます。
              また、本サイトはデモ環境のため、予告なくデータベースを初期化する場合があります。
              重要な情報を登録しないでください。
            </p>
          </section>

          <section>
            <h3 className="font-bold text-base mb-1">6. セキュリティ上のお願い</h3>
            <p className="font-medium text-red-700">
              本サイトは個人が運用する学習用環境です。他のサービスで使用している
              パスワードは絶対に使用しないでください。
            </p>
          </section>

          <section>
            <h3 className="font-bold text-base mb-1">7. お問い合わせ</h3>
            <p>
              本ポリシーに関するお問い合わせ、およびデータの削除依頼は、
              リポジトリの Issue よりご連絡ください。
            </p>
          </section>
        </div>

        <div className="p-4 border-t shrink-0">
          <button
            onClick={onClose}
            className="w-full bg-gray-900 text-white py-2 rounded-lg font-medium hover:bg-gray-800 transition"
          >
            閉じる
          </button>
        </div>
      </div>
    </div>
  );
}
