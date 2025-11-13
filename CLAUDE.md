# noet - 開発者向けドキュメント

このドキュメントは、noetを拡張または変更する開発者のための技術仕様と実装詳細を提供します。

## アーキテクチャ概要

### モジュール構造

```
src/
├── main.rs           # エントリーポイント、引数パース、エラーハンドリング
├── cli.rs            # Clapコマンド定義
├── config.rs         # 設定ファイル管理
├── auth.rs           # 環境変数による認証情報管理
├── error.rs          # thiserrorによるカスタムエラー型
├── models.rs         # API用のSerdeデータ構造
├── editor.rs         # エディタ起動と設定管理
├── tui_diff.rs       # TUI差分表示
├── workspace.rs      # ワークスペース管理
├── rate_limiter.rs   # レート制限（500ms固定）
├── api/              # Note APIクライアント実装
│   ├── mod.rs
│   ├── client.rs     # reqwestベースのHTTPクライアント（レート制限統合）
│   ├── article.rs    # 記事CRUD操作
│   ├── tag.rs        # ハッシュタグ操作
│   ├── magazine.rs   # マガジン管理操作
│   ├── engagement.rs # いいね・コメント操作
│   ├── user.rs       # ユーザープロフィール操作
│   └── analytics.rs  # 統計情報操作
└── commands/         # CLIコマンド実装
    ├── mod.rs
    ├── article.rs    # 記事管理コマンド
    ├── tag.rs        # タグ管理コマンド
    ├── magazine.rs   # マガジン管理コマンド
    ├── engagement.rs # エンゲージメントコマンド
    ├── user.rs       # ユーザー情報コマンド
    ├── auth.rs       # 認証コマンド
    ├── export.rs     # エクスポート機能（上書き警告付き）
    ├── template.rs   # テンプレート管理
    ├── workspace.rs  # ワークスペース初期化
    └── interactive.rs # インタラクティブモード
```

## Note非公式API仕様

### ベースURL

```
https://note.com
```

### 認証

Cookieベースの認証を使用：
- **セッションCookie**: `_note_session_v5` (必須)
- **XSRFトークン**: `X-XSRF-TOKEN` ヘッダー (オプションだが推奨)

### APIエンドポイント

#### 記事 (v1, v2, v3)

```
POST   /api/v1/text_notes                    # 記事作成
PUT    /api/v1/text_notes/{id}               # 記事更新
DELETE /api/v1/text_notes/{id}               # 記事削除
POST   /api/v1/text_notes/draft_save?id={id} # 下書き保存
GET    /api/v2/creators/{username}/contents  # ユーザー記事一覧
GET    /api/v3/notes/{key}                   # 記事詳細取得
GET    /api/v3/searches?context=note&q={q}   # 記事検索
```

#### タグ (v2)

```
GET    /api/v2/hashtags?page={page}          # ハッシュタグ一覧
GET    /api/v2/hashtags/{name}               # ハッシュタグ詳細
```

#### マガジン (v1, v3)

```
GET    /api/v1/magazines/{key}                              # マガジン取得
POST   /api/v1/our/magazines/{key}/notes                    # 記事追加
DELETE /api/v1/our/magazines/{key}/notes/{note_key}         # 記事削除
GET    /api/v3/searches?context=magazine&q={q}              # マガジン検索
```

#### エンゲージメント (v1, v3)

```
POST   /api/v3/notes/{key}/likes             # いいね
DELETE /api/v3/notes/{key}/likes             # いいね解除
GET    /api/v3/notes/{key}/likes             # いいね一覧
GET    /api/v1/note/{id}/comments            # コメント取得
POST   /api/v1/note/{id}/comments            # コメント投稿
```

#### ユーザー (v1, v2, v3)

```
GET    /api/v2/creators/{username}           # ユーザープロフィール
GET    /api/v1/followings/{username}/list    # フォロー一覧
GET    /api/v1/followers/{username}/list     # フォロワー一覧
POST   /api/v3/users/{key}/following         # フォロー
GET    /api/v3/searches?context=user&q={q}   # ユーザー検索
```

#### 統計 (v1, v3)

```
GET    /api/v1/stats/pv?page={page}          # PV統計
GET    /api/v3/notice_counts                 # 通知数
```

## データモデル

### Article

```rust
pub struct Article {
    pub id: Option<String>,          // 内部ID
    pub key: Option<String>,         // URL用のkey
    pub name: String,                // タイトル
    pub body: String,                // 本文（Markdown）
    pub status: Option<ArticleStatus>, // published/draft/scheduled
    pub hashtag_notes: Option<Vec<Hashtag>>,
    pub publish_at: Option<DateTime<Utc>>,
    pub like_count: Option<u32>,
    pub comment_count: Option<u32>,
    pub read_count: Option<u32>,
}
```

### ArticleStatus

```rust
pub enum ArticleStatus {
    Published,  // 公開済み
    Draft,      // 下書き
    Scheduled,  // 予約投稿
}
```

## APIテスト結果（2025-11-14）

実際のNote.com APIを使用したテスト結果を記録します。

### ✅ 正常動作するAPI

#### 認証確認 (`auth status`)
- 環境変数から認証情報を読み込み
- セッションクッキーとXSRFトークンをマスク表示
- **動作**: 正常

#### 記事一覧取得 (`list {username}`)
- エンドポイント: `GET /api/v2/creators/{username}/contents?kind=note&page={page}`
- ユーザーID（URLname）: `kako_jun`（アンダースコア）
- ユーザー名（表示名）: `kako-jun`（ハイフン）
- **動作**: 正常（6件の公開記事を取得成功）

### ❌ 動作しない・問題があるAPI

#### 記事作成 (`create_article`)
- エンドポイント: `POST /api/v1/text_notes`
- **問題点**:
  - APIレスポンス自体は返ってくる（IDとkeyが取得できる）
  - しかしNote.comのGUI上で記事が確認できない
  - レスポンスの`status`フィールドが空文字列`""`
  - レスポンスの`body`フィールドが空文字列`""`
  - `is_draft: false`, `is_published: false`の状態
- **対応**: ArticleStatusに空文字列を`Draft`として扱うカスタムデシリアライザーを実装
- **要調査**: 本文を保存する正しいAPIエンドポイント・パラメータ

#### 記事取得 (`get_article`)
- エンドポイント: `GET /api/v3/notes/{key}`
- **問題点**:
  - APIレスポンスは返ってくる
  - frontmatter（タイトル、ステータス、ID、keyなど）は取得できる
  - しかし`body`フィールドが常に空文字列
- **要調査**: 本文を取得する別のエンドポイントまたはパラメータの存在

#### 記事更新 (`update_article`)
- エンドポイント: `PUT /api/v1/text_notes/{id}`
- **問題点**: 422エラー「不正なパラメータが渡されました」
- **要調査**: 正しいパラメータ形式、必須フィールド

#### タグ一覧 (`tag list`)
- エンドポイント: `GET /api/v2/hashtags?page={page}`
- **問題点**: レスポンス形式エラー: `invalid type: map, expected a sequence`
- **要調査**: 実際のレスポンス形式とデータ構造

#### ユーザー情報 (`user`)
- エンドポイント: `GET /api/v2/creators/{username}`
- **問題点**: 404エラー「リソースが見つかりません」
- **テスト値**: `kako-jun`（ハイフン）でテスト、`kako_jun`（アンダースコア）が正しい可能性
- **要調査**: 正しいエンドポイントパス、パラメータ形式

### 実装上の対応

1. **ArticleStatusデシリアライゼーション**: 空文字列を`Draft`として処理
2. **メッセージ改善**: 状態に応じたメッセージ表示（下書き保存中/公開中/予約投稿中）
3. **デバッグログ追加**: 問題調査を容易にするログ出力

### 今後の調査が必要な項目

1. **記事本文の保存**:
   - `POST /api/v1/text_notes/draft_save?id={id}` の使用方法
   - create後に即座にupdateが必要か
   - 別の本文保存用エンドポイントの存在

2. **APIパラメータ形式**:
   - 各エンドポイントの正確な必須/オプションパラメータ
   - リクエストボディの正しいJSON構造

3. **レスポンス形式**:
   - 各エンドポイントの実際のレスポンススキーマ
   - エラーレスポンスの形式

4. **認証とセッション**:
   - セッションクッキーの有効期限
   - XSRFトークンの取得と更新方法

## 設定

### 設定ファイルの場所

- Linux: `~/.config/noet/config.toml`
- macOS: `~/Library/Application Support/noet/config.toml`
- Windows: `%APPDATA%\noet\config.toml`

### 設定構造

```toml
default_status = "draft"
default_tags = []
editor = "code -w"          # エディタコマンド（オプション）
username = "your-username"   # ユーザー名（オプション）
base_url = "https://note.com"
```

### エディタ設定

エディタは以下の優先順位で決定されます：

1. `config.toml`の`editor`フィールド
2. 環境変数`$VISUAL`
3. 環境変数`$EDITOR`
4. プラットフォームデフォルト（vim/notepad/open -e）

**VSCodeの場合**: `editor = "code -w"` (`-w`は編集完了まで待機)

## 認証情報の管理

認証情報は環境変数で管理されます：

- **セッションCookie**: `NOET_SESSION_COOKIE` (必須)
- **XSRFトークン**: `NOET_XSRF_TOKEN` (オプション)

ユーザーはシェル設定ファイル（`~/.zshrc`、`~/.bashrc`など）に環境変数を設定します。

## プロキシ対応

環境変数で設定：

```bash
export HTTP_PROXY=http://proxy.example.com:8080
export HTTPS_PROXY=https://proxy.example.com:8080
```

`NoteClient`の初期化時に自動的に読み込まれ、reqwestのプロキシ設定に適用されます。

## 新機能の追加

### 新しいコマンドの追加

1. `src/cli.rs`でコマンド定義：

```rust
#[derive(Subcommand)]
pub enum Commands {
    // ... 既存のコマンド

    /// 新しいコマンドの説明
    MyCommand {
        /// 引数の説明
        arg: String,
    },
}
```

2. `src/commands/`に実装を作成：

```rust
// src/commands/my_command.rs
use crate::api::NoteClient;
use crate::auth::Credentials;
use crate::config::Config;
use crate::error::Result;

pub async fn my_command(arg: &str) -> Result<()> {
    let config = Config::load()?;
    let credentials = Credentials::load()?;
    let client = NoteClient::new(config, credentials)?;

    // 実装
    Ok(())
}
```

3. `src/commands/mod.rs`に追加：

```rust
pub mod my_command;
```

4. `src/main.rs`で配線：

```rust
Commands::MyCommand { arg } => {
    commands::my_command::my_command(&arg).await?;
}
```

### 新しいAPIエンドポイントの追加

1. `src/api/`の該当モジュールにメソッド追加：

```rust
// src/api/article.rs
impl NoteClient {
    pub async fn my_new_api(&self, param: &str) -> Result<SomeType> {
        let path = format!("/api/v1/some_endpoint/{}", param);
        let response = self.get(&path).await?;
        let data: SomeType = response.json().await?;
        Ok(data)
    }
}
```

2. 必要に応じて`src/models.rs`にモデル追加：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SomeType {
    pub field: String,
}
```

### 新しいエラー型の追加

`src/error.rs`の`NoetError`にバリアントを追加：

```rust
#[derive(Error, Debug)]
pub enum NoetError {
    // ... 既存のバリアント

    #[error("My error: {0}")]
    MyError(String),
}
```

## 開発環境のセットアップ

```bash
# リポジトリをクローン
git clone https://github.com/kako-jun/noet.git
cd noet

# 依存関係をインストール（cargo-huskyが自動的にgit hooksをインストール）
cargo build
```

`cargo build`を実行すると、`cargo-husky`が自動的にGit hooksをインストールします。

### コード整形とLint

```bash
# コードを整形
cargo fmt

# Lintチェック
cargo clippy --all-targets --all-features -- -D warnings
```

### Git Hooks

`cargo-husky`により、コミット時に以下が自動実行されます：

- **pre-commit**: `cargo fmt --check` と `cargo clippy`

これにより、フォーマットされていないコードやlint警告のあるコードはコミットできません。

## テスト戦略

### 手動テスト

実際のNote APIでテスト：

```bash
# 認証
cargo run -- auth login
cargo run -- auth status

# 記事操作
cargo run -- new "テスト記事"
cargo run -- publish test-article.md --draft
cargo run -- list your-username

# タグ操作
cargo run -- tag list
cargo run -- tag suggest "rust"
```

### 統合テスト

実際のAPI呼び出しなしでテストする場合、HTTPクライアントをモック：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_article_creation() {
        // APIエンドポイントをモック
        let _m = mock("POST", "/api/v1/text_notes")
            .with_status(200)
            .with_body(r#"{"data": {"id": "123"}}"#)
            .create();

        // テスト実装
    }
}
```

## HTTPクライアント設定

`NoteClient`は`reqwest`を使用し、以下の設定：
- 30秒のタイムアウト
- Cookieストレージ
- XSRFトークン挿入
- プロキシサポート（HTTP_PROXY/HTTPS_PROXY環境変数）
- カスタムエラーハンドリング
- **レート制限（500ms固定）** - すべてのHTTPメソッド（GET/POST/PUT/DELETE）で自動適用

## レート制限

**実装状況**: ✅ 実装済み（v0.1.0）

IPバンを防ぐため、`rate_limiter.rs`モジュールで固定レート制限を実装：

### 仕様
- **固定ウェイト**: 500ms（2リクエスト/秒）
- **適用範囲**: すべてのAPI呼び出し（GET/POST/PUT/DELETE）
- **スレッドセーフ**: Mutexによる同期
- **自動適用**: `NoteClient`に統合、ユーザーの手動制御不要

### 実装

```rust
// src/rate_limiter.rs
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub struct RateLimiter {
    last_request: Mutex<Option<Instant>>,
    delay: Duration,
}

impl RateLimiter {
    pub fn new(delay_ms: u64) -> Self {
        Self {
            last_request: Mutex::new(None),
            delay: Duration::from_millis(delay_ms),
        }
    }

    pub async fn wait(&self) {
        // ロック保持中はawaitしない実装
        // 詳細はsrc/rate_limiter.rsを参照
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(500) // 500ms delay
    }
}
```

### NoteClientとの統合

```rust
// src/api/client.rs
pub struct NoteClient {
    client: Client,
    base_url: String,
    config: Config,
    credentials: Credentials,
    rate_limiter: RateLimiter, // 自動レート制限
}

pub async fn get(&self, path: &str) -> Result<Response> {
    self.rate_limiter.wait().await; // すべてのリクエスト前に自動待機
    // ... リクエスト処理
}
```

### 連続リクエストの例

```bash
# エクスポート時: 100記事 = 約50秒（100 × 500ms）
noet export --all --username myuser

# 一覧取得時: ページごとに500ms待機
noet list myuser
```

## 実装済み機能

### v0.1.0で実装済み

- [x] ワークスペース機能（`.noet/`ディレクトリでプロジェクト管理）
- [x] テンプレート機能（記事テンプレートの作成・管理・使用）
- [x] エクスポート機能（Note.comから記事をMarkdownでダウンロード）
  - [x] 上書き警告（既存ファイル保護）
- [x] TUI差分表示（公開前に変更内容を並列比較）
- [x] インタラクティブモード（メニュー駆動のUI）
- [x] エディタ統合（設定可能なエディタ自動起動）
- [x] **レート制限（500ms固定、IPバン防止）**
- [x] **完全日本語化UI**

## 今後の改善案

### 高優先度

- [ ] より良いエラーメッセージと提案
- [ ] 一括操作（一括アップロード/削除）

### 中優先度

- [ ] 画像アップロード対応（要API調査）
  - Note.comの画像管理仕様が不明確
  - 画像削除可否が不明（ゴミ画像が溜まるリスク）
  - 代替案: Web UIでアップロード→URLをMarkdownに貼り付け
- [ ] 記事バージョン管理/履歴

### 低優先度

- [ ] 記事分析ダッシュボード
- [ ] 予約投稿
- [ ] Webhook通知
- [ ] プラグインシステム

### 不要と判断した機能

- ~~下書き自動保存機能~~ → ローカルファイル管理で十分（ローカル = 下書き）
- ~~Markdownプレビュー~~ → VSCode等のエディタで可能
- ~~記事検索機能~~ → エクスポートフォルダを`grep`すればよい

## 既知の問題と制限事項

### API制限

- **非公式API**: 予告なく壊れる可能性
- **画像アップロードなし**: 現在未実装
- **有料コンテンツなし**: プレミアムコンテンツAPIは未文書化
- **レート制限**: 公式な制限は不明

### 実装制限

- **並行アップロードなし**: 順次処理のみ
- **エラー回復限定**: ネットワークエラーは手動再試行が必要
- **プログレスバーなし**: 長時間操作用
- **Markdownパース**: 基本的なfrontmatterのみ

## デバッグ

デバッグログを有効化：

```bash
RUST_LOG=debug cargo run -- <command>
```

HTTPリクエストのトレースログ：

```bash
RUST_LOG=noet=trace,reqwest=trace cargo run -- <command>
```

## ビルドとリリース

### 開発ビルド

```bash
cargo build
```

### リリースビルド

```bash
cargo build --release
```

### クロスコンパイル

Linuxターゲット：

```bash
cargo build --release --target x86_64-unknown-linux-gnu
```

macOSターゲット：

```bash
cargo build --release --target x86_64-apple-darwin
```

Windowsターゲット：

```bash
cargo build --release --target x86_64-pc-windows-gnu
```

## セキュリティ考慮事項

- **Cookie保存**: セッションCookieをログや出力に含めない
  - 環境変数で管理（シェル設定ファイル）
  - 設定ファイルのパーミッションを適切に設定（`chmod 600`推奨）
  - Gitなどでコミットしないように`.gitignore`に追加
- **XSRF保護**: 可能な場合は常にXSRFトークンを含める
- **入力検証**: API呼び出し前にユーザー入力をサニタイズ
- **レート制限**: 不正利用とIPバンを防ぐ
- **エラーメッセージ**: 機密情報を漏らさない

## コントリビューションガイドライン

1. リポジトリをフォーク
2. フィーチャーブランチを作成
3. 変更を加える
4. 該当する場合はテストを追加
5. ドキュメントを更新
6. プルリクエストを提出

### コードスタイル

- Rustの規約に従う（rustfmt）
- lintingにclippyを使用
- 複雑なロジックにはコメントを追加
- 説明的なコミットメッセージを書く

### コミットメッセージ形式

```
<type>(<scope>): <subject>

<body>

<footer>
```

タイプ: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

## ライセンス

MIT License - LICENSEファイル参照

## 参考資料

- [Note.com](https://note.com)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Clap Documentation](https://docs.rs/clap/)
- [Reqwest Documentation](https://docs.rs/reqwest/)
- [Tokio Documentation](https://docs.rs/tokio/)
