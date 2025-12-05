# 画像アップロード機能実装サマリー

## 実装日: 2025-12-05

## 概要

Note.comのProseMirrorエディタにプログラマティックに画像をアップロードする機能を実装しました。

## 技術的課題と解決策

### 課題1: Markdownの画像記法がサポートされない

**問題**: ProseMirrorエディタは `![caption](url)` 形式の画像記法をサポートしていない

**解決**:
- DataTransfer APIとFile APIを使用してfile inputにプログラマティックにファイルを設定
- アップロード完了後、返されたNote.com URLでMarkdown内の画像パスを置換

### 課題2: file input要素へのセキュリティ制約

**問題**: セキュリティ上、file inputに直接パスを設定できない

**解決**:
- base64エンコードされた画像データからBlobオブジェクトを生成
- BlobからFileオブジェクトを作成
- DataTransferオブジェクトを使用してfile inputのfilesプロパティに設定

### 課題3: DOMセレクタの動的性

**問題**: 挿入メニューの＋ボタンはエディタ内をクリックしないと表示されない

**解決**:
- エディタの末尾にフォーカスを設定
- `button[aria-label="メニューを開く"]` セレクタで＋ボタンを検出
- メニュー内の「画像」ボタンをテキスト検索で特定

### 課題4: アップロード完了の検出

**問題**: 画像アップロードが非同期で、完了タイミングが不明

**解決**:
- MutationObserverパターンを実装
- 新しい `<figure>` 要素の出現を監視
- `img.src` が `blob:` でなく `st-note.com` を含むことで完了を確認

## 実装コンポーネント

### CLI側 (Rust)

#### apps/cli/src/image_handler.rs

```rust
// 主要な機能:
- extract_image_references(): Markdownから画像参照を抽出
- read_image_as_base64(): 画像ファイルをbase64エンコード
- process_images(): 画像データを処理してImageData構造体を生成
```

サポートフォーマット: JPEG, PNG, GIF, WebP

#### apps/cli/src/extension_client.rs

```rust
// 新規追加メソッド:
- create_article_with_images(): 画像付き記事作成
```

#### apps/cli/src/commands/extension.rs

```rust
// 更新:
- create_article(): 画像処理を統合
- parse_markdown_file(): header_imageをFrontmatterから抽出
```

### 拡張機能側 (JavaScript)

#### apps/extension/src/background.js

```javascript
// ユーティリティ関数:
- base64ToBlob(): base64をBlobに変換
- waitForCondition(): 条件が満たされるまで待機

// 本文内画像:
- uploadImage(): 単一の画像をアップロード

// 見出し画像:
- uploadHeaderImage(): 見出し画像をアップロード
- removeHeaderImage(): 見出し画像を削除

// フォーム操作:
- fillArticleFormWithImages(): 画像付き記事フォームを埋める
```

## アップロードフロー

### 本文内画像

```
1. CLI: Markdownから ![caption](./path/to/image.jpg) を抽出
2. CLI: 画像ファイルをbase64エンコード
3. CLI → 拡張機能: WebSocketでImageDataを送信
4. 拡張機能: base64 → Blob → File オブジェクトを生成
5. 拡張機能: エディタ末尾にフォーカス
6. 拡張機能: ＋ボタンをクリック
7. 拡張機能: 「画像」ボタンをクリック
8. 拡張機能: file inputにFileを設定
9. 拡張機能: 新しい<figure>要素の出現を待機
10. 拡張機能: img.srcが設定されるのを待機（Note.com URL）
11. 拡張機能: キャプションを設定（figcaption.textContent）
12. 拡張機能: Note.com URLを返す
13. 拡張機能: Markdown内のローカルパスをNote.com URLに置換
14. 拡張機能: 本文をエディタにペースト
```

### 見出し画像

```
1. CLI: Frontmatterから header_image: ./path/to/image.jpg を抽出
2. CLI: 画像ファイルをbase64エンコード
3. CLI → 拡張機能: WebSocketでImageDataを送信
4. 拡張機能: base64 → Blob → File オブジェクトを生成
5. 拡張機能: 「画像を追加」ボタンをクリック
6. 拡張機能: 「画像をアップロード」ボタンをクリック
7. 拡張機能: file inputにFileを設定
8. 拡張機能: トリミングUI表示後、「保存」ボタンをクリック（デフォルトトリミング）
9. 拡張機能: img[alt="eyecatch"]の出現を待機
10. 拡張機能: Note.com URLを返す
```

## Markdownフォーマット

### Frontmatter

```yaml
---
title: 記事タイトル
tags: [タグ1, タグ2]
header_image: ./images/thumbnail.jpg
---
```

### 本文内画像

```markdown
![キャプションテキスト](./images/photo.jpg)
```

## 使用例

```bash
# 画像を含む記事を下書きとして投稿
noet create article-with-images.md --draft
```

出力例:
```
拡張機能に接続中...
記事を下書きとして投稿中... (画像: 2枚 (見出し画像あり))
✓ 記事を下書きしました
  URL: https://note.com/username/n/n1234567890ab
  ステータス: draft
  アップロードされた画像:
    ./images/photo1.jpg → https://assets.st-note.com/img/xxx.jpg
    ./images/photo2.png → https://assets.st-note.com/img/yyy.png
  見出し画像: https://assets.st-note.com/production/uploads/images/zzz/eyecatch/aaa.jpg
```

## テスト対象

### 単体テスト

- [x] `extract_image_references()`: 画像参照抽出のテスト
- [x] パス解決ロジック（相対パス、絶対パス）
- [x] URL判定（http://, https://, st-note.com のスキップ）

### 統合テスト（実施予定）

- [ ] 単一画像のアップロード
- [ ] 複数画像のアップロード
- [ ] 見出し画像のみ
- [ ] 見出し画像 + 本文内画像
- [ ] 大きな画像ファイル（タイムアウト確認）
- [ ] サポートされていないフォーマット（エラーハンドリング）

## 制限事項と既知の問題

1. **画像フォーマット**: JPEG, PNG, GIF, WebPのみサポート
2. **見出し画像のトリミング**: デフォルトトリミングのみ（カスタムトリミング未対応）
3. **メタデータ管理**: まだ未実装（記事更新時に画像の再アップロードを防ぐため必要）
4. **記事更新**: update_articleコマンドでの画像対応は未実装

## 今後の改善

### 短期

- [ ] 実際の画像を使った統合テスト
- [ ] エラーハンドリングの改善（リトライロジック）
- [ ] タイムアウト時間の調整

### 中期

- [ ] update_articleでの画像対応
- [ ] メタデータ管理システム（.noet/articles/{key}.json）
- [ ] 画像の差分アップロード（変更がない画像の再利用）

### 長期

- [ ] カスタムトリミング設定（見出し画像）
- [ ] 画像最適化（自動圧縮）
- [ ] 複数画像の並列アップロード
- [ ] プログレスバー表示

## 参考資料

- [IMAGE_UPLOAD_INVESTIGATION.md](./IMAGE_UPLOAD_INVESTIGATION.md): 調査結果
- [EDITOR_SELECTORS_INVESTIGATION.md](./EDITOR_SELECTORS_INVESTIGATION.md): DOMセレクタ調査
- [IMAGE_USAGE.md](./IMAGE_USAGE.md): ユーザー向け使い方ガイド

## 関連ファイル

### CLI
- `apps/cli/src/image_handler.rs`: 画像処理ロジック
- `apps/cli/src/extension_client.rs`: WebSocket通信クライアント
- `apps/cli/src/commands/extension.rs`: コマンド実装
- `apps/cli/Cargo.toml`: 依存関係（base64, regex追加）

### 拡張機能
- `apps/extension/src/background.js`: Service Worker（画像アップロードロジック）

### ドキュメント
- `docs/IMAGE_UPLOAD_INVESTIGATION.md`: 技術調査
- `docs/EDITOR_SELECTORS_INVESTIGATION.md`: DOM調査
- `docs/IMAGE_USAGE.md`: 使い方ガイド
- `docs/IMAGE_IMPLEMENTATION_SUMMARY.md`: このファイル
