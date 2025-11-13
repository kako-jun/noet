# noet - 開発者向けドキュメント

このドキュメントは、noetを拡張または変更する開発者のための技術仕様と実装詳細を提供します。

## アーキテクチャ概要

### モジュール構造

```
src/
├── main.rs           # エントリーポイント、引数パース、エラーハンドリング
├── cli.rs            # Clapコマンド定義
├── config.rs         # 設定ファイル管理
├── auth.rs           # システムキーリングによる認証情報保存
├── error.rs          # thiserrorによるカスタムエラー型
├── models.rs         # API用のSerdeデータ構造
├── editor.rs         # エディタ起動と設定管理
├── tui_diff.rs       # TUI差分表示
├── workspace.rs      # ワークスペース管理
├── api/              # Note APIクライアント実装
│   ├── mod.rs
│   ├── client.rs     # reqwestベースのHTTPクライアント
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
    ├── export.rs     # エクスポート機能
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
- **CSRFトークン**: `X-CSRF-Token` ヘッダー (オプションだが推奨)

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

## 認証情報の保存

認証情報はシステムキーリングに安全に保存されます：

- **macOS**: Keychain
- **Linux**: Secret Service (libsecret)
- **Windows**: Credential Manager

`keyring`クレートでクロスプラットフォーム対応。

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
- CSRFトークン挿入
- プロキシサポート（HTTP_PROXY/HTTPS_PROXY環境変数）
- カスタムエラーハンドリング

## レート制限のベストプラクティス

IPバンを避けるために：

1. リクエスト間に遅延を追加（100-500ms）
2. エラー時は指数バックオフを実装
3. 可能な場合はAPIレスポンスをキャッシュ
4. HTTP 429 (Too Many Requests)レスポンスを尊重

レート制限の例：

```rust
use tokio::time::{sleep, Duration};

pub struct RateLimiter {
    delay: Duration,
}

impl RateLimiter {
    pub fn new(requests_per_second: u64) -> Self {
        Self {
            delay: Duration::from_millis(1000 / requests_per_second),
        }
    }

    pub async fn wait(&self) {
        sleep(self.delay).await;
    }
}
```

## 実装済み機能

### v0.1.0で実装済み

- [x] ワークスペース機能（`.noet/`ディレクトリでプロジェクト管理）
- [x] テンプレート機能（記事テンプレートの作成・管理・使用）
- [x] エクスポート機能（Note.comから記事をMarkdownでダウンロード）
- [x] TUI差分表示（公開前に変更内容を並列比較）
- [x] インタラクティブモード（メニュー駆動のUI）
- [x] エディタ統合（設定可能なエディタ自動起動）

## 今後の改善案

### 高優先度

- [ ] レート制限実装
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
  - システムキーリングに保存（OSレベルで暗号化）
  - macOS: AES-256、Linux: libsecret、Windows: DPAPI
  - アプリケーションレベルでの追加暗号化は行っていない（OSの暗号化で十分）
- **CSRF保護**: 可能な場合は常にCSRFトークンを含める
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
