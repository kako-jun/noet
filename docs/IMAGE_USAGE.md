# 画像アップロード機能の使い方

## 概要

noet CLIは、Markdownファイル内の画像参照を自動的に検出し、Note.comにアップロードする機能をサポートしています。

## Markdown記法

### 本文内画像

Markdown標準の画像記法を使用します：

```markdown
![キャプション](./images/photo.jpg)
```

- `キャプション`: 画像の下に表示されるキャプションテキスト
- パス: 相対パスまたは絶対パスで画像ファイルを指定

### 見出し画像（サムネイル）

Frontmatterで指定します：

```markdown
---
title: 記事タイトル
tags: [タグ1, タグ2]
header_image: ./images/thumbnail.jpg
---

記事本文...
```

## サポートする画像フォーマット

- JPEG (`.jpg`, `.jpeg`)
- PNG (`.png`)
- GIF (`.gif`)
- WebP (`.webp`)

**注意**: Note.comにアップロードされた画像は元のフォーマットのまま保存されます。

## 記事投稿の例

### 例1: 本文内画像のみ

```markdown
---
title: 料理の記録
tags: [料理, レシピ]
---

## 今日のランチ

美味しいパスタを作りました！

![完成したパスタ](./images/pasta.jpg)

材料は以下の通り...
```

### 例2: 見出し画像と本文内画像

```markdown
---
title: 旅行記：京都編
tags: [旅行, 京都]
header_image: ./photos/kinkakuji.jpg
---

京都に行ってきました。

## 金閣寺

![金閣寺の写真](./photos/kinkakuji-detail.jpg)

素晴らしい景色でした。

## 清水寺

![清水寺の舞台](./photos/kiyomizu.jpg)

高さに驚きました。
```

## 投稿コマンド

```bash
# 下書きとして投稿
noet create article.md --draft

# 公開記事として投稿
noet create article.md
```

## 処理フロー

1. **CLI側**:
   - Markdownファイルを読み込み
   - `![caption](path)` 形式の画像参照を検出
   - Frontmatterから `header_image` を抽出
   - 画像ファイルをbase64エンコード
   - 拡張機能に送信

2. **拡張機能側**:
   - Note.comの編集ページを開く
   - タイトルを入力
   - 見出し画像がある場合、先にアップロード
   - 本文内画像を順番にアップロード
   - 各画像のNote.com URL（`https://assets.st-note.com/img/...`）を取得
   - Markdown内の画像パスをNote.com URLに置換
   - 本文を投稿

3. **結果**:
   - アップロードされた画像のローカルパス → Note.com URL対応が表示される
   - 記事URLが返される

## 出力例

```
拡張機能に接続中...
WebSocket サーバーを起動しました (ws://127.0.0.1:9876)
ブラウザ拡張機能からの接続を待っています...
拡張機能が接続しました: 127.0.0.1:xxxxx
記事を公開として投稿中... (画像: 2枚 (見出し画像あり))
✓ 記事を公開しました
  URL: https://note.com/username/n/n1234567890ab
  ステータス: published
  アップロードされた画像:
    ./images/pasta.jpg → https://assets.st-note.com/img/1234567890ab-pasta.jpg
    ./photos/kinkakuji-detail.jpg → https://assets.st-note.com/img/abcdef123456-kinkakuji-detail.jpg
  見出し画像: https://assets.st-note.com/production/uploads/images/123456789/eyecatch/abcdef.jpg
```

## 注意事項

### URLのスキップ

すでにアップロード済みのNote.com URLや外部URLは自動的にスキップされます：

```markdown
![外部画像](https://example.com/image.jpg)
<!-- スキップされる -->

![すでにアップロード済み](https://assets.st-note.com/img/123.webp)
<!-- スキップされる -->

![ローカル画像](./local.jpg)
<!-- アップロードされる -->
```

### パス解決

- 相対パス（`./images/photo.jpg`, `../photos/pic.png`）: Markdownファイルからの相対パスとして解決
- 絶対パス（`/home/user/images/photo.jpg`）: そのまま使用

### 画像が見つからない場合

存在しない画像パスを指定した場合、警告が表示されますが投稿は継続されます：

```
Warning: Image file not found: /path/to/missing.jpg (referenced as ./missing.jpg)
```

### エラーハンドリング

- サポートされていない画像フォーマット: エラーで中断
- 画像ファイルが読み込めない: エラーで中断
- アップロード失敗: エラーメッセージを表示して中断

## 今後の予定

- [ ] 記事更新時の画像ハンドリング
- [ ] メタデータファイルによる画像URL管理
- [ ] 画像の差分アップロード（変更がない画像の再利用）
- [ ] トリミング設定（見出し画像のカスタムトリミング）

## トラブルシューティング

### 「Unsupported image format」エラー

サポートされていない画像フォーマットです。JPEG、PNG、GIF、WebPのいずれかを使用してください。

### 「Image file not found」警告

指定されたパスに画像ファイルが存在しません。パスが正しいか確認してください。

### アップロードがタイムアウトする

画像サイズが大きすぎる可能性があります。画像を圧縮するか、サイズを小さくしてください。
