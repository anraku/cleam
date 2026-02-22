## 1. データモデルの拡張

- [x] 1.1 `StatefulList<T>` に `visible_indices: Option<Vec<usize>>` フィールドを追加する
- [x] 1.2 `StatefulList::next()` / `previous()` を `visible_indices` を考慮した実装に更新する
- [x] 1.3 `StatefulList::selected()` / `selected_index()` を `visible_indices` 経由で正しいアイテムを返すよう更新する
- [x] 1.4 `App` 構造体に `main_search_query: String` と `main_search_active: bool` を追加する
- [x] 1.5 `App::new()` で新フィールドを初期化する

## 2. フィルタリングロジックの実装

- [x] 2.1 `App` に `apply_main_search()` メソッドを追加する（クエリに基づいて `log_groups.visible_indices` または `log_streams.visible_indices` を更新する）
- [x] 2.2 大文字小文字を区別しない部分文字列マッチを実装する（`to_lowercase()` を使用）
- [x] 2.3 ページネーション（`load_more_groups` / `load_more_streams`）後に `apply_main_search()` を呼び出し、新アイテムをフィルタ対象に含める

## 3. キーハンドリングの実装

- [x] 3.1 `handle_main_key` を検索モード中のキー処理に対応させる（文字入力・Backspace・Esc）
- [x] 3.2 通常モードで `/` キーを押したとき `main_search_active = true` にする処理を追加する
- [x] 3.3 `Esc` キーで検索モード終了・クエリクリア・`visible_indices` リセットを実装する
- [x] 3.4 h/l でパネル切り替え時に検索状態をリセットする処理を追加する

## 4. UI描画の実装

- [x] 4.1 `main_screen.rs` の `draw` 関数にて、`main_search_active` が true のときパネル下部（フッター上）に検索バーを表示する
- [x] 4.2 検索バーに `Search: <query>` を表示し、カーソル位置を示す `_` を末尾に追加する
- [x] 4.3 グループ・ストリームパネルの描画で `visible_indices` がある場合はフィルタ済みアイテムのみ `ListItem` に変換して表示する
- [x] 4.4 フィルタ結果が0件の場合に「No matches」メッセージを表示する
- [x] 4.5 フッターのキーヒントに `[/] Search` を追加する
