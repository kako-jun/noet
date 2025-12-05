# noet - 開発者向けドキュメント

このドキュメントは、noetを拡張または変更する開発者のための技術仕様と実装詳細を提供します。

## 🚨 開発状況（最新）

**画像アップロード機能を実装中**（2025-12-05）

- ✅ CLI側: 画像抽出、base64エンコード、画像ダウンロード
- ✅ 拡張機能側: 画像アップロード（本文内・見出し画像）
- ⚠️ **統合テスト未実施**

**詳細は `docs/IMAGE_FEATURE_STATUS.md` を参照すること。**

## アーキテクチャ概要

### 設計原則

**Note.com APIは一切使用しない。** すべての操作はブラウザ拡張機能を経由してDOM操作で行う。

理由：
- Note.comの非公式APIは不安定で予告なく変更される
- ブラウザ拡張機能はユーザーセッションを使用するため認証が不要
- DOM操作はNote.comのUI変更に対してより堅牢

### システム構成

```
┌─────────────────────────────────────────────────────────────┐
│                         User                                 │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    CLI (noet)                                │
│  - Markdownファイルの読み書き                                 │
│  - 画像の抽出・base64エンコード・ダウンロード                  │
│  - WebSocketサーバー (ws://127.0.0.1:9876)                    │
│  - 拡張機能へのコマンド送信                                    │
└─────────────────────────────────────────────────────────────┘
                              │ WebSocket
                              ▼
┌─────────────────────────────────────────────────────────────┐
│               Browser Extension                              │
│  - Note.comページでのDOM操作                                  │
│  - 記事の取得・作成・更新・削除                                │
│  - 画像のアップロード（本文内・見出し画像）                     │
│  - ユーザー認証状態の確認                                     │
└─────────────────────────────────────────────────────────────┘
                              │ DOM操作
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Note.com                                  │
└─────────────────────────────────────────────────────────────┘
```

### モジュール構造

```
apps/
├── cli/
│   └── src/
│       ├── main.rs           # エントリーポイント
│       ├── cli.rs            # Clapコマンド定義
│       ├── error.rs          # カスタムエラー型
│       ├── editor.rs         # エディタ起動
│       ├── workspace.rs      # ワークスペース管理
│       ├── extension_client.rs # WebSocket経由で拡張機能と通信
│       ├── image_handler.rs  # 画像処理（抽出・エンコード）
│       ├── native_messaging.rs # Native Messaging対応（未使用）
│       ├── converters/
│       │   ├── mod.rs
│       │   └── html_to_md.rs # HTML→Markdown変換
│       └── commands/
│           ├── mod.rs
│           ├── extension.rs  # 拡張機能経由のコマンド
│           ├── setup.rs      # セットアップウィザード
│           ├── template.rs   # テンプレート管理
│           └── workspace.rs  # ワークスペース初期化
└── extension/
    └── src/
        ├── manifest.json     # Chrome拡張機能マニフェスト
        ├── background.js     # Service Worker
        ├── content.js        # Content Script（DOM操作）
        └── popup/            # ポップアップUI
```

## CLIコマンド

すべてのコマンドは拡張機能経由で実行されます。

```bash
noet setup                        # セットアップウィザード
noet init [path]                  # ワークスペース初期化
noet ping                         # 拡張機能との接続確認
noet auth                         # 認証状態確認
noet list                         # 記事一覧取得
noet get -u <user> <key>          # 記事取得（表示のみ）
noet get -u <user> <key> --save <file>  # 記事取得（画像ダウンロード＆保存）
noet create <file> [--draft]      # 記事作成（画像対応）
noet update <key> <file> [--draft]# 記事更新（画像対応）
noet delete <key>                 # 記事削除
noet template list                # テンプレート一覧
noet template add <name>          # テンプレート作成
noet template show <name>         # テンプレート表示
noet template remove <name>       # テンプレート削除
```

## ブラウザ拡張機能

### 通信プロトコル

CLIと拡張機能はWebSocket（ws://127.0.0.1:9876）で通信します。

#### リクエスト形式

```json
{
  "id": "uuid",
  "command": "コマンド名",
  "params": { ... }
}
```

#### レスポンス形式

```json
{
  "id": "uuid",
  "status": "success" | "error",
  "data": { ... },
  "error": { "code": "...", "message": "..." }
}
```

### サポートするコマンド

| コマンド | 説明 | パラメータ |
|---------|------|-----------|
| ping | 接続確認 | なし |
| check_auth | 認証状態確認 | なし |
| list_articles | 記事一覧取得 | なし |
| get_article | 記事取得 | username, key |
| create_article | 記事作成 | title, body, tags, draft, images?, header_image? |
| update_article | 記事更新 | key, title, body, tags, draft, images?, header_image? |
| delete_article | 記事削除 | key |

### DOM操作の詳細

#### 記事取得
- ダッシュボード（/dashboard/contents）から記事一覧をスクレイピング
- 個別記事ページから本文（HTML）を取得
- HTML→Markdown変換はCLI側で実施

#### 記事作成・更新
- `/publish/`または`/publish/?edit=<id>`ページにナビゲート
- 画像がある場合：
  - 見出し画像: `button[aria-label="画像を追加"]` からアップロード
  - 本文内画像: エディタ内で `button[aria-label="メニューを開く"]` → 「画像」からアップロード
  - DataTransfer APIを使用してfile inputにプログラマティックに設定
  - アップロード完了後、Note.com URLを取得
  - Markdown内の画像パスをNote.com URLに置換
- ProseMirrorエディタにMarkdownをペースト（ClipboardEvent使用）
- ハッシュタグ入力欄に直接入力
- マガジン選択（ドロップダウン操作）
- 公開/下書き保存ボタンをクリック

**重要**:
- ProseMirrorエディタはMarkdownをペーストすると自動認識します。HTMLに変換する必要はありません。
- 画像のMarkdown記法（`![](url)`）はサポートされていないため、先に画像をアップロードしてURLを取得する必要があります。

```javascript
// 正しい実装
const clipboardData = new DataTransfer();
clipboardData.setData('text/plain', markdownBody);
const pasteEvent = new ClipboardEvent('paste', {
  bubbles: true,
  cancelable: true,
  clipboardData: clipboardData
});
bodyEditor.dispatchEvent(pasteEvent);
```

## HTML→Markdown変換

`html2md`ライブラリを使用。Note.com特有のHTML構造に対応。

### Note.comのHTML構造

```html
<!-- 段落 -->
<p name="uuid" id="uuid">テキスト<br>改行</p>

<!-- 見出し -->
<h2 name="uuid" id="uuid">見出しテキスト</h2>

<!-- 画像 -->
<img src="https://assets.st-note.com/img/..." alt="" width="620" height="224">

<!-- 埋め込みコンテンツ -->
<figure name="uuid" data-src="..." embedded-service="external-article">
  <a href="..." rel="nofollow noopener" target="_blank">
    <strong>タイトル</strong>
    <em>説明</em>
  </a>
</figure>
```

## ワークスペース

`.noet/`ディレクトリでプロジェクト管理。

```
.noet/
├── templates/     # 記事テンプレート
└── config.toml    # ワークスペース設定（予約）
```

## セットアップ

1. `noet setup`を実行
2. 拡張機能がダウンロードされる（開発時はローカルの`apps/extension`を使用）
3. Native Messagingマニフェストが設定される
4. Chromeで拡張機能を読み込む（デベロッパーモード）

## 開発環境のセットアップ

```bash
git clone https://github.com/kako-jun/noet.git
cd noet

# CLIビルド
cd apps/cli
cargo build

# 拡張機能は直接Chromeに読み込む
# chrome://extensions → デベロッパーモード → apps/extension を読み込む
```

### コード整形とLint

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
```

### テスト

```bash
cargo test
```

## 依存関係

### CLI (Rust)

- `clap` - CLIフレームワーク
- `tokio` - 非同期ランタイム
- `tokio-tungstenite` - WebSocket
- `serde`/`serde_json` - シリアライズ
- `html2md` - HTML→Markdown変換
- `colored` - ターミナル色付け
- `dialoguer` - 対話型入力
- `reqwest` - HTTPクライアント（画像ダウンロード、セットアップ用）
- `zip` - ZIP解凍
- `base64` - 画像のbase64エンコード
- `regex` - 画像参照の抽出

### 拡張機能 (JavaScript)

- Chrome Extension Manifest V3
- Service Worker
- Content Scripts

## セキュリティ考慮事項

- WebSocket通信はローカルホスト（127.0.0.1）のみ
- ユーザー認証はNote.comのセッションに依存
- 認証情報はCLI側で管理しない

## コントリビューションガイドライン

1. リポジトリをフォーク
2. フィーチャーブランチを作成
3. 変更を加える
4. テストを追加
5. プルリクエストを提出

### コミットメッセージ形式

```
<type>(<scope>): <subject>
```

タイプ: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

## 画像アップロード機能（2025-12-05実装）

### Markdownフォーマット

```markdown
---
title: "記事タイトル"
tags: ["タグ1", "タグ2"]
header_image: ./images/thumbnail.jpg  # 見出し画像（オプション）
---

# 本文

![キャプション](./images/photo.jpg)
```

### 実装状況

- ✅ CLI側: 画像抽出、base64エンコード、ダウンロード
- ✅ 拡張機能側: 画像アップロード（DataTransfer API使用）
- ⚠️ 統合テスト未実施

### 詳細ドキュメント

- **現在の状況**: `docs/IMAGE_FEATURE_STATUS.md` ← **必読**
- **調査結果**: `docs/IMAGE_UPLOAD_INVESTIGATION.md`
- **DOM調査**: `docs/EDITOR_SELECTORS_INVESTIGATION.md`
- **使い方**: `docs/IMAGE_USAGE.md`
- **実装詳細**: `docs/IMAGE_IMPLEMENTATION_SUMMARY.md`

### 設計方針

- **メタデータ管理不要**: 毎回全画像を再アップロード
- **画像フォーマット**: 変換されない（JPG→JPG、PNG→PNG）
- **Note.com側のクリーンアップ**: 未参照画像は自動削除（想定）

## ライセンス

MIT License
