## Context

`src/app.rs` の `handle_main_key` にて、`main_search_active: bool` フラグで「検索入力モード中かどうか」を管理している。Enter 押下時は `clear_main_search()` を呼び出しており、これがクエリと `visible_indices` を両方クリアしてしまう。

現在 `main_search_active` は「検索バーにカーソルがある入力状態」と「絞り込み結果が適用された状態」の両方を兼ねているため、検索確定後も絞り込み結果を維持する仕組みがない。

## Goals / Non-Goals

**Goals:**
- Enter で検索入力モードを抜けつつ、クエリと絞り込み結果を保持する
- 空クエリで Enter を押すと初期状態（全件表示）に戻る
- 検索確定後も j/k ナビゲーションと Enter による項目選択が通常通り動作する

**Non-Goals:**
- 検索 UI・表示の変更（検索バーのスタイル等）
- パネル切り替え時の挙動変更（既存の `clear_main_search` はそのまま維持）
- Esc の挙動変更

## Decisions

### 決定: Enter の挙動を「入力モード終了」と「選択実行」に分離する

**現状**: Enter → `clear_main_search()` → 選択処理（クエリ・結果が消える）

**変更後**:
- クエリが空 → `clear_main_search()` のみ（初期状態に戻る）
- クエリが非空 → `main_search_active = false` のみ（クエリと `visible_indices` は保持）、選択処理は **しない**

選択（画面遷移）は検索モード外の通常 Enter ハンドラに委ねる。これにより、検索確定後に j/k でカーソルを動かしてから Enter で選択するという自然な操作フローが成立する。

**代替案**: Enter で即選択も同時に行う → 検索確定と選択が一度に起きてしまい、「絞り込み結果を眺めてから選ぶ」ユースケースに対応できない。

### 決定: `main_search_active` の意味を「入力フォーカス」に限定する

`main_search_active = false` でも `main_search_query` と `visible_indices` が残る状態を許容する。検索バーの表示は `main_search_active` ではなく `!main_search_query.is_empty()` を条件にすることで、確定後も絞り込み状態であることをユーザーに示す。

## Risks / Trade-offs

- [Risk] 検索確定後に検索バーが表示され続けることでユーザーが混乱する可能性 → 検索バーの表示条件を `main_search_active` から `!main_search_query.is_empty()` に変更し、「絞り込み中」であることを常に明示することで対処
- [Risk] `clear_main_search` の呼び出し箇所（パネル切り替え等）への影響 → Enter での呼び出しのみ変更し、他の呼び出し箇所はそのまま維持するため影響なし
