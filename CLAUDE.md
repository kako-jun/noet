# noet - 開発者向けドキュメント

このドキュメントは、noetを拡張または変更する開発者のための技術仕様と実装詳細を提供します。

## アーキテクチャ概要

### システム構成

```
┌─────────────────┐                    ┌─────────────────────────┐
│   noet (CLI)    │                    │  ブラウザ拡張            │
│   Rust          │                    │  (バックグラウンド常駐)   │
└────────┬────────┘                    └────────────┬────────────┘
         │                                          │
         │  1. リクエストファイル書き込み             │
         ▼                                          │
┌─────────────────┐                                 │
│ ~/.noet/request │ ◄───────────────────────────────┘
│   .json         │      2. ポーリングで読み取り (100-500ms)
└─────────────────┘
         │
         │  3. 拡張がNote.comを操作（fetch/DOM操作）
         │
         ▼
┌─────────────────┐
│ ~/.noet/response│ ◄─── 4. 生HTMLを書き込み
│   .json         │
└────────┬────────┘
         │
         │  5. CLIが読み取り
         ▼
┌─────────────────┐
│   noet (CLI)    │ ─── 6. HTML→MD変換、表示
└─────────────────┘
```

### 設計方針

- **APIは使わない**: Note.comの非公式APIは不安定で認証が困難
- **スクレイピングベース**: ブラウザ拡張がログイン済みセッションを利用
- **責務分離**:
  - 拡張: Note.comとの通信、生HTML取得/DOM操作のみ
  - CLI: HTML↔Markdown変換、UI、すべてのロジック
- **バージョン同期**: CLIと拡張は同一バージョンを強制（不整合時は更新を促す）

### モノレポ構成

```
noet/
├── apps/
│   ├── cli/                    # Rust CLI
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── cli.rs          # Clapコマンド定義
│   │   │   ├── config.rs       # 設定ファイル管理
│   │   │   ├── error.rs        # カスタムエラー型
│   │   │   ├── models.rs       # データ構造
│   │   │   ├── editor.rs       # エディタ起動
│   │   │   ├── rpc/            # 拡張との通信
│   │   │   │   ├── mod.rs
│   │   │   │   ├── client.rs   # リクエスト/レスポンス処理
│   │   │   │   └── commands.rs # RPCコマンド定義
│   │   │   ├── converters/     # HTML↔Markdown変換
│   │   │   │   ├── mod.rs
│   │   │   │   ├── html_to_md.rs
│   │   │   │   └── md_to_html.rs
│   │   │   └── commands/       # CLIコマンド実装
│   │   ├── Cargo.toml
│   │   └── README.md
│   │
│   └── extension/              # ブラウザ拡張 (Chrome/Firefox)
│       ├── src/
│       │   ├── background.js   # Service Worker（ポーリング）
│       │   └── content.js      # Note.comページ操作
│       ├── manifest.json
│       └── README.md
│
├── protocol.yaml               # 共有RPC仕様
├── .github/
│   └── workflows/
│       └── release.yml         # 両方まとめてビルド＆リリース
├── CLAUDE.md                   # このファイル
├── README.md                   # ユーザー向けドキュメント
└── LICENSE
```

## RPC プロトコル仕様

### 通信方式

- **ファイルベースRPC**: `~/.noet/request.json` と `~/.noet/response.json`
- **ポーリング**: 拡張が100-500ms間隔でリクエストファイルを監視
- **同期的**: CLIはレスポンスを待機（タイムアウトあり）

### リクエスト形式

```json
{
  "id": "uuid-v4",
  "command": "get_article",
  "params": {
    "key": "n1234567890"
  },
  "timestamp": 1699999999999
}
```

### レスポンス形式

```json
{
  "id": "uuid-v4",
  "status": "success",
  "data": {
    "html": "<html>...</html>",
    "title": "記事タイトル"
  },
  "timestamp": 1699999999999
}
```

### エラーレスポンス

```json
{
  "id": "uuid-v4",
  "status": "error",
  "error": {
    "code": "NOT_LOGGED_IN",
    "message": "Note.comにログインしていません"
  },
  "timestamp": 1699999999999
}
```

### コマンド一覧

#### ping
拡張の存在確認とバージョンチェック。

```yaml
command: ping
params: {}
returns:
  version: string      # 拡張のバージョン
  extension_id: string
```

#### check_auth
ログイン状態の確認。

```yaml
command: check_auth
params: {}
returns:
  logged_in: bool
  username: string | null
```

#### list_articles
公開記事一覧取得。

```yaml
command: list_articles
params:
  username: string      # 必須
  page: number          # default: 1
returns:
  articles:
    - key: string
      title: string
      updated_at: string
  has_next: bool
```

#### get_article
記事取得（生HTML）。

```yaml
command: get_article
params:
  key: string           # 必須（URLのn以降）
returns:
  html: string          # 生HTML（変換はCLI側）
  title: string
  tags: [string]        # ハッシュタグ一覧
  created_at: string
  updated_at: string
```

#### create_article
記事作成（常に公開）。

```yaml
command: create_article
params:
  title: string         # 必須
  body: string          # プレーンテキスト
  tags: [string]        # ハッシュタグ（#なし）
returns:
  key: string
  url: string
```

#### update_article
記事更新。

```yaml
command: update_article
params:
  key: string           # 必須
  title: string         # optional
  body: string          # optional
  tags: [string]        # optional
returns:
  success: bool
  url: string
```

#### delete_article
記事削除。

```yaml
command: delete_article
params:
  key: string
returns:
  success: bool
```

#### set_debug_mode
デバッグモード切り替え。ONにすると拡張の操作が可視化される。

```yaml
command: set_debug_mode
params:
  enabled: bool         # true: 操作を表示, false: 非表示
returns:
  success: bool
  debug_mode: bool
```

#### get_debug_mode
現在のデバッグモード状態を取得。

```yaml
command: get_debug_mode
params: {}
returns:
  debug_mode: bool
```

### エラーコード

| コード | 説明 |
|--------|------|
| NOT_LOGGED_IN | Note.comにログインしていません |
| NOT_FOUND | リソースが見つかりません |
| PERMISSION_DENIED | 権限がありません |
| NETWORK_ERROR | 通信エラー |
| TIMEOUT | タイムアウト |
| INVALID_PARAMS | パラメータが不正です |
| EXTENSION_NOT_FOUND | 拡張がインストールされていません |
| VERSION_MISMATCH | 拡張のバージョンが一致しません |
| UNKNOWN | 不明なエラー |

## ブラウザ拡張

### 役割

- `~/.noet/request.json` をポーリング監視
- Note.comへfetch（ブラウザセッション利用）
- Note.comのエディタをDOM操作（投稿時）
- 生HTMLをレスポンスファイルに書き込み
- **変換ロジックは一切持たない**

### 技術スタック

- **Manifest V3** (Chrome拡張)
- **Service Worker**: バックグラウンドでポーリング
- **Content Script**: Note.comページのDOM操作
- **chrome.scripting API**: スクリプト注入

### ファイル構成

```
extension/
├── manifest.json       # 拡張マニフェスト
├── src/
│   ├── background.js   # Service Worker
│   │   - ポーリング処理
│   │   - fetch代行（記事取得等）
│   │   - Content Scriptへのメッセージ送信
│   └── content.js      # Content Script
│       - Note.comエディタのDOM操作
│       - 記事投稿/更新処理
└── icons/              # 拡張アイコン
```

### 投稿フロー（DOM操作）

```
1. background.js: create_article コマンド受信
2. background.js: Note.com新規作成ページをタブで開く
3. background.js: content.js を注入
4. content.js: タイトル入力欄にテキスト設定
5. content.js: 本文入力欄にテキスト設定
6. content.js: 下書き保存/公開ボタンをクリック
7. content.js: 完了を background.js に通知
8. background.js: レスポンスファイル書き込み
```

## CLI (Rust)

### 役割

- ユーザーインターフェース（コマンドライン）
- リクエストファイル書き込み、レスポンス待機
- HTML→Markdown変換（記事取得時）
- Markdown→プレーンテキスト変換（記事投稿時）
- 設定管理、エディタ連携

### 起動時フロー

```
1. noet <command> 実行
2. ping コマンドで拡張存在確認（タイムアウト: 3秒）
3. 応答なし → 拡張インストールを促す
4. バージョン不一致 → 更新を促す
5. OK → 本来のコマンド実行
```

### 主要モジュール

| モジュール | 責務 |
|-----------|------|
| `rpc/client.rs` | 拡張との通信（ファイルI/O） |
| `rpc/commands.rs` | RPCコマンドの型定義 |
| `converters/html_to_md.rs` | HTML→Markdown変換 |
| `converters/md_to_html.rs` | Markdown→HTML変換（必要に応じて） |
| `commands/*.rs` | 各CLIコマンドの実装 |

## 配布

### GitHub Release

```
noet-v1.0.0/
├── noet-linux-x86_64.tar.gz
├── noet-darwin-x86_64.tar.gz
├── noet-darwin-aarch64.tar.gz
├── noet-windows-x86_64.zip
└── noet-extension.zip          # ブラウザ拡張
```

### インストール手順

1. Releaseからバイナリとextensionをダウンロード
2. バイナリをPATHに配置
3. `noet-extension.zip`を解凍
4. `chrome://extensions` を開く
5. 「デベロッパーモード」ON
6. 「パッケージ化されていない拡張機能を読み込む」
7. 解凍したフォルダを選択
8. `noet ping` で動作確認

### CI/CD (GitHub Actions)

```yaml
# リリース時に自動ビルド
- Rust: cross-compile for Linux/macOS/Windows
- Extension: zip packages/extension/ directory
- Upload all artifacts to Release
```

## 設定

### 設定ファイルの場所

- Linux: `~/.config/noet/config.toml`
- macOS: `~/Library/Application Support/noet/config.toml`
- Windows: `%APPDATA%\noet\config.toml`

### 設定構造

```toml
editor = "code -w"          # エディタコマンド
username = "your-username"   # デフォルトユーザー名
timeout_ms = 30000           # 拡張応答タイムアウト
```

### RPC通信ディレクトリ

- `~/.noet/request.json` - CLIからのリクエスト
- `~/.noet/response.json` - 拡張からのレスポンス

## 開発

### 環境セットアップ

```bash
git clone https://github.com/kako-jun/noet.git
cd noet

# CLI
cd apps/cli
cargo build

# 拡張
cd apps/extension
# (plain JS、ビルド不要)
```

### デバッグ

CLI:
```bash
RUST_LOG=debug cargo run -- <command>
```

拡張:
- `chrome://extensions` → 拡張の「Service Worker」リンク → DevTools

### テスト

```bash
# CLI
cd apps/cli
cargo test

# 拡張（手動テスト）
# 1. chrome://extensions から読み込み
# 2. Note.comにログイン
# 3. noet ping で疎通確認
```

## 移行計画

### Phase 1: モノレポ化
- [x] 設計決定
- [x] `apps/cli/` ディレクトリ作成、既存src移動
- [x] `apps/extension/` ディレクトリ作成
- [x] ルートのCargo.tomlをworkspace化
- [x] protocol.yaml作成

### Phase 2: 拡張開発
- [x] manifest.json作成
- [x] background.js: 雛形作成
- [x] content.js: 雛形作成
- [ ] Native Messaging実装
- [ ] Note.com DOM調査（Playwright）
- [ ] 各コマンドのDOM操作実装

### Phase 3: CLI書き換え
- [ ] `src/api/` を `src/rpc/` に置き換え
- [ ] NoteClient → RpcClient
- [ ] 認証関連コード削除（不要になる）

### Phase 4: 統合テスト
- [ ] 全コマンドの動作確認
- [ ] エラーハンドリング確認
- [ ] ドキュメント更新

## Note.com DOM構造（調査中）

### ログインページ
- URL: `https://note.com/login`
- メールアドレス入力: (調査中)
- パスワード入力: (調査中)
- ログインボタン: (調査中)

### エディタページ
- URL: `https://note.com/notes/new`
- タイトル入力: (調査中)
- 本文入力: (調査中)
- 下書き保存ボタン: (調査中)
- 公開ボタン: (調査中)

### 記事ページ
- URL: `https://note.com/{username}/n/{key}`
- 本文HTML: (調査中)
- メタデータ: `<script id="__NEXT_DATA__">` 内のJSON

## ライセンス

MIT License - LICENSEファイル参照
