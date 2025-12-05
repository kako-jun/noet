# 画像機能の実装状況（2025-12-05時点）

## 🎯 実装方針の決定事項

### ✅ 確定した仕様

1. **画像フォーマット**: 変換されない。JPG→JPG、PNG→PNG、元のまま保存される
2. **メタデータ管理**: 不要。毎回全画像を再アップロード
3. **Note.com側のクリーンアップ**: 未参照画像は自動削除される（想定）
4. **シンプルな実装**: 差分計算なし、画像の同一性チェックなし

### ❌ 不要になった機能

- ~~画像の同一性確認（ハッシュ比較）~~
- ~~メタデータファイル（.noet/articles/{key}.json）~~
- ~~画像の差分アップロード~~

## 📋 実装完了（コンパイル済み、テスト未実施）

### CLI側 (Rust)

#### 1. 画像のアップロード（create/update）

**ファイル**: `apps/cli/src/image_handler.rs`
- ✅ `extract_image_references()`: Markdown内の `![caption](path)` を抽出
- ✅ `read_image_as_base64()`: 画像ファイルをbase64エンコード
- ✅ `process_images()`: ImageData構造体を生成

**ファイル**: `apps/cli/src/commands/extension.rs`
- ✅ `create_article()`: 画像付き新規投稿
- ✅ `update_article()`: 画像付き記事更新

**ファイル**: `apps/cli/src/extension_client.rs`
- ✅ `create_article_with_images()`: WebSocket経由で画像データ送信
- ✅ `update_article_with_images()`: WebSocket経由で画像データ送信

**Frontmatter対応**:
```yaml
---
title: "記事タイトル"
tags: ["タグ1", "タグ2"]
header_image: ./images/thumbnail.jpg  # 見出し画像（オプション）
---
```

**サポートフォーマット**: JPEG, PNG, GIF, WebP

#### 2. 画像のダウンロード（get）

**ファイル**: `apps/cli/src/commands/extension.rs`
- ✅ `get_article()`: `--save` オプションでファイル保存
- ✅ `download_images_and_replace_urls()`: 画像ダウンロード＆URL置換
- ✅ `download_image()`: 個別画像ダウンロード

**コマンド**:
```bash
# 表示のみ
noet get -u username article-key

# ファイル保存（画像もダウンロード）
noet get -u username article-key --save article.md
```

**生成される構造**:
```
./
├── article.md
└── images/
    ├── image1.jpg
    └── image2.png
```

### 拡張機能側 (JavaScript)

**ファイル**: `apps/extension/src/background.js`

#### ユーティリティ関数
- ✅ `base64ToBlob()`: base64 → Blob変換
- ✅ `waitForCondition()`: 条件待機

#### 本文内画像
- ✅ `uploadImage()`: 単一画像アップロード
  - DataTransfer APIでfile inputに設定
  - 新しい `<figure>` 要素の出現を監視
  - `img.src` が st-note.com URLになるまで待機
  - キャプション設定

#### 見出し画像
- ✅ `uploadHeaderImage()`: 見出し画像アップロード
  - `button[aria-label="画像を追加"]` をクリック
  - 「画像をアップロード」ボタンをクリック
  - トリミングUI表示後、「保存」ボタンを自動クリック
  - `img[alt="eyecatch"]` の出現を待機

- ✅ `removeHeaderImage()`: 見出し画像削除
  - `[role="img"][aria-label="削除"]` をクリック

#### フォーム操作
- ✅ `fillArticleFormWithImages()`: 画像付きフォーム入力
  - 見出し画像アップロード（あれば）
  - 本文内画像を順次アップロード
  - Markdown内の画像パスをNote.com URLに置換
  - 本文をエディタに設定

#### コマンドハンドラ
- ✅ `handleCreateArticle()`: 画像対応済み
- ✅ `handleUpdateArticle()`: 画像対応済み

## ⚠️ 未テスト項目（実装済み、動作未確認）

### 必須テスト

1. **新規投稿（create）**:
   - [ ] 本文内画像のみ
   - [ ] 見出し画像のみ
   - [ ] 本文内画像 + 見出し画像
   - [ ] 画像なし（既存機能の退行テスト）

2. **記事更新（update）**:
   - [ ] 本文内画像のみ
   - [ ] 見出し画像のみ
   - [ ] 本文内画像 + 見出し画像
   - [ ] 画像なし（既存機能の退行テスト）

3. **記事取得（get）**:
   - [ ] `--save` なし（表示のみ）
   - [ ] `--save` あり（画像ダウンロード）
   - [ ] 画像なし記事の取得

4. **エラーハンドリング**:
   - [ ] 存在しない画像パス
   - [ ] サポートされていないフォーマット
   - [ ] ネットワークエラー（アップロード失敗）
   - [ ] タイムアウト

5. **エッジケース**:
   - [ ] 大きな画像ファイル（5MB以上）
   - [ ] 多数の画像（10枚以上）
   - [ ] 日本語ファイル名
   - [ ] スペース含むファイル名

## 🔧 既知の制限事項

1. **見出し画像のトリミング**: デフォルトトリミングのみ（カスタム不可）
2. **画像の同一性チェック**: なし（毎回再アップロード）
3. **並列アップロード**: 実装なし（順次処理）
4. **プログレスバー**: なし
5. **画像最適化**: なし（自動圧縮など）

## 📝 次にやるべきこと（優先順位順）

### 1. 統合テスト（最優先）

実際の画像ファイルを用意してテスト：

```bash
# テスト用ディレクトリ作成
mkdir -p test-article/images
cd test-article

# テスト用画像を用意（ダミー画像をダウンロードなど）
# ...

# テスト用Markdownファイル作成
cat > article.md << 'EOF'
---
title: "画像テスト記事"
tags: ["テスト"]
header_image: ./images/header.jpg
---

# 本文

![キャプション1](./images/photo1.jpg)

本文テキスト。

![キャプション2](./images/photo2.png)
EOF

# 下書きとして投稿
noet create article.md --draft

# 結果確認
# - ブラウザで下書き記事を開く
# - 画像が正しく表示されるか
# - キャプションが正しいか
# - 見出し画像が正しいか
```

### 2. エラーケースのテスト

```bash
# 存在しない画像
echo '![test](./missing.jpg)' >> article.md
noet create article.md --draft  # エラーメッセージ確認

# サポートされていないフォーマット
echo '![test](./file.bmp)' >> article.md
noet create article.md --draft  # エラーメッセージ確認
```

### 3. get機能のテスト

```bash
# 既存の画像付き記事を取得
noet get -u <username> <key> --save downloaded.md

# 確認事項:
# - images/ ディレクトリが作成されるか
# - 画像がダウンロードされるか
# - Markdown内のURLがローカルパスに置換されているか
# - Frontmatterが正しいか
```

### 4. バグ修正（テストで見つかった問題）

テスト中に見つかったバグを修正する。

### 5. ドキュメント更新

テスト完了後、以下を更新：
- README.md に画像機能の説明を追加
- IMAGE_USAGE.md にテスト結果を反映
- 既知の問題を更新

### 6. 将来的な改善（低優先度）

- [ ] 並列画像アップロード（パフォーマンス改善）
- [ ] プログレスバー表示
- [ ] 画像の自動圧縮・最適化
- [ ] カスタムトリミング設定（見出し画像）

## 📚 関連ドキュメント

- **調査結果**: `IMAGE_UPLOAD_INVESTIGATION.md`
- **DOM調査**: `EDITOR_SELECTORS_INVESTIGATION.md`
- **使い方ガイド**: `IMAGE_USAGE.md`
- **実装サマリー**: `IMAGE_IMPLEMENTATION_SUMMARY.md`
- **このファイル**: `IMAGE_FEATURE_STATUS.md`

## 🐛 トラブルシューティング

### ビルドエラー

```bash
cd apps/cli
cargo build
```

警告（`full_match` is never read）は無視してOK。

### 拡張機能の再読み込み

1. `chrome://extensions`
2. 拡張機能の「再読み込み」ボタンをクリック

### WebSocketサーバーのポート競合

```bash
# 既存のプロセスを確認
lsof -i :9876

# 必要に応じてプロセスをkill
kill -9 <PID>
```

## 💡 開発のヒント

1. **デバッグログ**: `RUST_LOG=debug noet create article.md`
2. **拡張機能のログ**: ブラウザの開発者ツール → Console
3. **ファイル構造確認**: `tree -L 2 apps/`

## ✅ 完了の定義

画像機能が「完了」と言えるのは：

1. ✅ 実装完了（コンパイル成功）← **現在ここ**
2. ⬜ 統合テスト完了（実際の画像で動作確認）
3. ⬜ エッジケーステスト完了
4. ⬜ ドキュメント更新完了
5. ⬜ リリース準備完了（CHANGELOG更新など）

---

**次回起動時の最初の作業**: このファイルを読んでから、「1. 統合テスト」を実行すること。
