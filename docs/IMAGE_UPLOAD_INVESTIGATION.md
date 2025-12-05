# Note.com 画像アップロード調査結果

## 調査日: 2025-12-05

## 注意: 画像フォーマットについて

**誤解**: JPGをアップロードするとWebPに変換される
**訂正**: 画像フォーマットは変換されない。JPG、PNG、WebPなど元のフォーマットのままNote.comに保存される。

## 重要な発見

### ❌ base64自動アップロードは機能しない

**結論**: ProseMirrorエディタはMarkdown画像記法 `![](url)` をサポートしておらず、base64画像の自動アップロードも行われない。

### Markdownサポート範囲

ProseMirrorエディタがサポートするMarkdown記法は限定的：
- 箇条書き
- 水平線（hr）
- その他基本的なテキスト装飾

**画像のMarkdown記法はサポートされていない**

### テスト結果

#### 1. Markdownペースト
```markdown
![test](data:image/jpeg;base64,...)
```
→ **失敗**: テキストとしてそのまま表示される

#### 2. HTML直接挿入
```html
<img src="data:image/jpeg;base64,...">
```
→ **失敗**: base64のまま残る（アップロードされない）

### 画像アップロードUI（詳細調査済み）

#### 挿入メニューの開き方

1. エディタ内の挿入したい位置をクリック
2. 左側に表示される＋ボタンをクリック
   - セレクタ: `button[aria-label="メニューを開く"]`
3. 挿入メニューが開く

#### 画像挿入ボタン

挿入メニュー内の「画像」ボタン：
- セレクタ: `button.sc-6fa32351-4.eYVdgL`
- またはテキスト検索: `button:has-text("画像")`

クリックするとfile選択ダイアログが開く（推定: `input[type="file"][accept="image/*"]`）

## 実装への影響

### メタデータ管理が必要

base64方式による自動アップロードができないため、以下の対応が必要：

1. **画像ファイルを個別にアップロード**
   - UIボタン経由
   - またはドラッグ&ドロップ
   
2. **アップロード後のURL管理**
   - ローカルファイル → Note.com URL の対応関係を保存
   - メタデータファイルが必要

### 実装方針（改訂）

```
1. CLI: Markdown内の画像パス ![](./local/image.jpg) を検出
2. CLI: 画像ファイルをbase64エンコードまたはバイナリで読み込み
3. 拡張機能: 画像を一つずつアップロード
   - 方法A: file inputに直接ファイルデータを設定
   - 方法B: ドラッグ&ドロップイベントをシミュレート
4. 拡張機能: アップロード後のURL（https://assets.st-note.com/img/...）を取得
5. CLI: ローカルパス → URL の対応をメタデータファイルに保存
6. 拡張機能: Markdown内の画像パスをNote.com URLに置換してから本文を設定
```

### 解決済み

- [x] 挿入メニューの＋ボタンセレクタ: `button[aria-label="メニューを開く"]`
- [x] 画像挿入ボタンセレクタ: `button.sc-6fa32351-4.eYVdgL`
- [x] アップロード完了の検出方法: 新しい `<figure>` 要素の出現を監視
- [x] アップロード後URL取得: `figure img` の `src` 属性から取得
- [x] キャプション設定方法: `figcaption.textContent` に直接設定

### 未解決事項

- [x] file inputへのプログラマティックなファイル設定方法 → **解決**: DataTransfer APIとFile APIを使用
- [x] 見出し画像のトリミングUI操作 → **解決**: デフォルトトリミングで「保存」ボタンをクリック

### 実装完了事項（2025-12-05）

- [x] **本文内画像アップロード**:
  - DataTransfer APIを使用してfile inputにプログラマティックにファイルを設定
  - base64データからBlobとFileオブジェクトを作成
  - アップロード完了検出（新しい`<figure>`要素の出現を監視）
  - アップロード後URL取得（`img.src`が`blob:`でなく`st-note.com`を含むことを確認）
  - キャプション設定（`figcaption.textContent`に直接設定）

- [x] **見出し画像アップロード**:
  - `button[aria-label="画像を追加"]`をクリック
  - 「画像をアップロード」ボタンを検索してクリック
  - DataTransfer APIでfile inputに設定
  - トリミングUI表示後、「保存」ボタンを自動クリック（デフォルトトリミングを受け入れ）
  - `img[alt="eyecatch"]`の出現を監視してURL取得

- [x] **見出し画像削除**:
  - `[role="img"][aria-label="削除"]`のアイコンを検索
  - 最も近いbuttonをクリック
  - 画像削除を確認

## CLI側実装完了（2025-12-05）

### 実装内容

1. **image_handler.rs**:
   - `extract_image_references()`: Markdown内の `![caption](path)` パターンを抽出
   - `read_image_as_base64()`: 画像ファイルをbase64エンコード
   - `process_images()`: Markdownファイルから画像を処理してImageData構造体を生成

2. **extension_client.rs**:
   - `create_article_with_images()`: 画像付き記事作成メソッドを追加

3. **commands/extension.rs**:
   - `create_article()`: 画像処理機能を統合
   - `parse_markdown_file()`: Frontmatterから `header_image` を抽出するように拡張

### 使用方法

Markdownファイル例：

```markdown
---
title: テスト記事
tags: [テスト]
header_image: ./images/thumbnail.jpg
---

# 本文

![キャプション](./images/photo.jpg)
```

投稿コマンド：

```bash
noet create article.md --draft
```

### 次のステップ

- [ ] 実際の画像を使ってテスト
- [ ] update_article での画像対応
- [ ] メタデータ管理システム（.noet/articles/{key}.json）
- [ ] エラーハンドリングの改善

## 結論

当初想定していたbase64による簡易実装は不可能だったが、DataTransfer APIとFile APIを使用することで、
プログラマティックな画像アップロードが可能になった。

CLI側とブラウザ拡張機能側の実装が完了し、統合テストの段階に入った。

---

**次回作業時は `IMAGE_FEATURE_STATUS.md` を確認すること。**
現在の実装状況、テスト項目、次にやるべきことが全て記載されている。

