import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import App from './App.jsx'
import './index.css'

//HTMLの中からrootidを探し、ルートコンテナを作成する。これにより最新の機能を使えるようになり並列レンダリングを行えるようになる。
createRoot(document.getElementById('root')).render(

  <StrictMode>
    <App />
  </StrictMode>,
)


