## 1. `src/app.rs` - Enter キーハンドラの変更

- [x] 1.1 `handle_main_key` の `main_search_active` ブロック内の `KeyCode::Enter` 分岐を修正する：クエリが空なら `clear_main_search()` を呼ぶ（現状維持）、非空なら `self.main_search_active = false` のみ実行してクエリと `visible_indices` を保持する

## 2. `src/ui/main_screen.rs` - 検索バー表示条件の変更

- [x] 2.1 レイアウト制約の条件 `if app.main_search_active` を `if !app.main_search_query.is_empty() || app.main_search_active` に変更し、確定後も検索バー用の行を確保する
- [x] 2.2 検索バーのテキスト生成で、`main_search_active` が true のときだけ末尾にカーソル文字（`_`）を付加し、false（確定済み）のときは `_` なしで `Search: <query>` を表示する
- [x] 2.3 フッター描画のチャンクインデックス計算（`if app.main_search_active { 3 } else { 2 }`）をレイアウト条件変更後の判定式に合わせて修正する
