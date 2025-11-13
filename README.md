# noet

> note.comをターミナルから操作する、Rust製の高速CLIツール

[![CI](https://github.com/kako-jun/noet/actions/workflows/ci.yml/badge.svg)](https://github.com/kako-jun/noet/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

[note.com](https://note.com) の記事を管理するためのコマンドラインツール（Rust製）。

`noet`を使うと、Hugoなどの静的サイトジェネレータのように、Markdownファイルでターミナルから記事の作成・投稿・管理ができます。

<!-- スクリーンショット予定地 -->
<!-- ![Demo](docs/images/demo.gif) -->

## 目次

- [特徴](#特徴)
- [インストール](#インストール)
- [クイックスタート](#クイックスタート)
- [コマンド](#コマンド)
- [使用例](#使用例)
- [設定](#設定)
- [トラブルシューティング](#トラブルシューティング)
- [開発](#開発)
- [今後の予定](#今後の予定)
- [FAQ](#faq)
- [ライセンス](#ライセンス)

## 特徴

- **記事管理**: 記事の作成、投稿、編集、削除、一覧表示
- **インタラクティブモード**: 引数なしで`noet`を実行すると、メニュー駆動のUIで操作可能
- **TUI差分表示**: 公開前に変更内容を左右並列で確認
- **ワークスペース機能**: Gitリポジトリのように`.noet/`でプロジェクト管理
- **テンプレート機能**: 記事テンプレートの作成・管理・使用
- **エクスポート機能**: Note.comから記事をMarkdownでダウンロード
- **エディタ統合**: 新規作成時に設定したエディタを自動起動
- **タグ管理**: ハッシュタグの一覧表示、検索、サジェスト
- **マガジン管理**: 記事のマガジン追加・削除
- **エンゲージメント**: いいね/いいね解除、コメント閲覧
- **ユーザー情報**: ユーザープロフィールと統計情報の取得
- **安全な認証**: 認証情報はシステムキーリングに安全に保存
- **Hugoライクなインターフェース**: frontmatter付きMarkdownベースの記事作成

## インストール

### バイナリをダウンロード（推奨）

[GitHub Releases](https://github.com/kako-jun/noet/releases)から、お使いのOSに対応したバイナリをダウンロードしてください。

**Linux (x86_64)**
```bash
wget https://github.com/kako-jun/noet/releases/latest/download/noet-linux-amd64.tar.gz
tar xzf noet-linux-amd64.tar.gz
sudo mv noet /usr/local/bin/
```

**macOS (Intel)**
```bash
curl -LO https://github.com/kako-jun/noet/releases/latest/download/noet-macos-amd64.tar.gz
tar xzf noet-macos-amd64.tar.gz
sudo mv noet /usr/local/bin/
```

**macOS (Apple Silicon)**
```bash
curl -LO https://github.com/kako-jun/noet/releases/latest/download/noet-macos-arm64.tar.gz
tar xzf noet-macos-arm64.tar.gz
sudo mv noet /usr/local/bin/
```

**Windows**

[noet-windows-amd64.zip](https://github.com/kako-jun/noet/releases/latest/download/noet-windows-amd64.zip)をダウンロードして展開し、パスを通してください。

### Cargoからインストール（近日公開予定）

```bash
cargo install noet
```

### ソースからビルド

```bash
git clone https://github.com/kako-jun/noet.git
cd noet
cargo install --path .
```

### 必要要件

- note.comアカウント
- （ソースビルドの場合）Rust 1.70以上

## クイックスタート

### 1. ワークスペースの初期化（オプション）

記事をGitで管理したい場合、まずワークスペースを初期化します：

```bash
mkdir ~/my-articles
cd ~/my-articles
git init
noet init  # .noet/ディレクトリを作成
```

### 2. 認証

Noteアカウントで認証します：

```bash
noet auth login
```

<!-- スクリーンショット予定地 -->
<!-- ![Login](docs/images/login.png) -->

以下の手順が必要です：
1. ブラウザで https://note.com にログイン
2. 開発者ツール (F12) → Application/Storage → Cookies を開く
3. `_note_session_v5` クッキーの値をコピー
4. プロンプトに貼り付け

認証状態を確認：

```bash
noet auth status
```

### 3. インタラクティブモードで使う（推奨）

引数なしで`noet`を実行すると、メニュー駆動のUIが起動します：

```bash
noet
```

メニューから以下の操作ができます：
1. 📝 新規記事を作成（タイトル入力 → テンプレート選択 → エディタ自動起動）
2. ✏️  既存記事を編集（ファイル選択 → エディタ起動）
3. 📤 記事を公開（ファイル選択 → 差分表示 → 公開確認）
4. 📋 自分の記事一覧
5. 🚪 終了

### 4. コマンドラインから使う

従来通り、コマンドラインからも操作できます：

```bash
# 新しい記事を作成
noet new "初めての記事"

# 記事を投稿（既存記事の場合は差分表示）
noet publish my-first-article.md

# 下書きとして投稿
noet publish my-first-article.md --draft

# 差分表示のみ（公開しない）
noet diff my-first-article.md

# 記事の一覧表示
noet list your-username
```

## コマンド

### インタラクティブモード

```bash
# 引数なしで実行（推奨）
noet
```

メニュー駆動のUIで記事管理が可能です。

### ワークスペース

```bash
# ワークスペースを初期化（.noet/ディレクトリを作成）
noet init [PATH]
```

ワークスペース機能により、記事をGitで管理できます。

### 記事管理

```bash
# 新しい記事を作成
noet new [TITLE] [--template <NAME>]

# 記事を投稿（既存記事の場合は差分を表示）
noet publish <FILE> [--draft] [--force]

# 差分表示のみ（公開しない）
noet diff <FILE>

# 既存の記事を編集
noet edit <ARTICLE_ID> <FILE>

# 記事を削除
noet delete <ARTICLE_ID> [--force]

# ユーザーの記事一覧
noet list <USERNAME> [--page <PAGE>]
```

### エクスポート

```bash
# 単一記事をエクスポート
noet export <ARTICLE_KEY> -o article.md

# 全記事をエクスポート
noet export --all --username <USER> -o ./exports/
```

### テンプレート

```bash
# テンプレート一覧
noet template list

# テンプレート作成
noet template add <NAME>

# テンプレート表示
noet template show <NAME>

# テンプレート削除
noet template remove <NAME>
```

### タグ管理

```bash
# ハッシュタグ一覧を表示
noet tag list [--page <PAGE>]

# キーワードでタグをサジェスト
noet tag suggest <KEYWORD>
```

### マガジン管理

```bash
# 記事をマガジンに追加
noet magazine add <MAGAZINE_KEY> --note-id <NOTE_ID> --note-key <NOTE_KEY>

# 記事をマガジンから削除
noet magazine remove <MAGAZINE_KEY> <NOTE_KEY>
```

### エンゲージメント

```bash
# 記事にいいね
noet like <ARTICLE_KEY>

# いいねを解除
noet unlike <ARTICLE_KEY>

# コメントを表示
noet comments <ARTICLE_ID>
```

### ユーザー情報

```bash
# ユーザープロフィールを表示
noet user <USERNAME>
```

### 認証

```bash
# Noteにログイン
noet auth login

# 認証状態を確認
noet auth status

# 認証を更新
noet auth refresh

# 保存済み認証情報をクリア
noet auth clear
```

## 設定

設定ファイルは以下の場所に保存されます：
- Linux: `~/.config/noet/config.toml`
- macOS: `~/Library/Application Support/noet/config.toml`
- Windows: `%APPDATA%\noet\config.toml`

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

## Frontmatter形式

記事はYAML frontmatterでメタデータを指定します：

```markdown
---
title: 記事タイトル
status: draft  # または "published"
tags: rust, cli, tutorial
---

Markdown形式の記事内容...
```

## プロキシ対応

環境変数でプロキシを設定できます：

```bash
export HTTP_PROXY=http://proxy.example.com:8080
export HTTPS_PROXY=https://proxy.example.com:8080
```

## 重要な注意事項

### 非公式API警告

このツールはNoteの**非公式API**を使用しています。以下の点に注意してください：

- APIエンドポイントは予告なく変更される可能性があります
- 過度な使用はレート制限やIP制限の対象となる可能性があります
- Noteのシステム更新により機能が動作しなくなる可能性があります
- このツールはnote.comから公式にサポートされていません

責任を持って使用し、過度なAPI呼び出しは避けてください。

### セキュリティ

- **認証情報の保存**: システムキーリングに保存（macOS: Keychain、Linux: Secret Service、Windows: Credential Manager）
- **暗号化**: OSレベルで自動的に暗号化されます
  - macOS: AES-256
  - Linux: OSレベルの暗号化（libsecret）
  - Windows: DPAPI（Data Protection API）
- **平文保存なし**: セッションクッキーは平文ファイルに保存されません
- **CSRF保護**: CSRFトークンは自動的に管理されます

## 開発

### ビルド

```bash
cargo build
```

### テスト実行

```bash
cargo test
```

### ローカル実行

```bash
cargo run -- <COMMAND>
```

## 使用例

### 日常的なワークフロー

```bash
# 新しい記事を作成
noet new "Rustでコマンドラインツールを作る"

# エディタで編集
vim rustでコマンドラインツールを作る.md

# 下書きとして投稿
noet publish rustでコマンドラインツールを作る.md --draft

# レビュー後、公開状態に更新
# （markdownのstatusをpublishedに変更してから）
noet edit <ARTICLE_ID> rustでコマンドラインツールを作る.md

# 記事一覧を確認
noet list your-username
```

### タグ付けのベストプラクティス

```markdown
---
title: Rustでコマンドラインツールを作る
status: published
tags: Rust, CLI, プログラミング, チュートリアル
---
```

タグサジェスト機能を使うと便利です：

```bash
noet tag suggest rust
# → 関連するハッシュタグの候補が表示されます
```

## トラブルシューティング

### 認証エラー

```
Error: Not authenticated. Please run 'noet auth login' first.
```

**解決方法:**
1. `noet auth clear` で既存の認証情報をクリア
2. `noet auth login` で再ログイン
3. セッションクッキーが有効か確認（ブラウザでログイン状態を確認）

### プロキシ環境での接続エラー

```
Error: HTTP request failed: ...
```

**解決方法:**
```bash
export HTTP_PROXY=http://proxy.example.com:8080
export HTTPS_PROXY=https://proxy.example.com:8080
noet auth status  # 接続テスト
```

### Linux での keyring エラー

```
Error: Keyring error: No keyring found
```

**解決方法（Ubuntu/Debian）:**
```bash
sudo apt-get install gnome-keyring libsecret-1-0
```

**解決方法（Arch Linux）:**
```bash
sudo pacman -S gnome-keyring libsecret
```

### macOS での権限エラー

初回起動時にキーチェーンへのアクセス許可を求められます。「許可」を選択してください。

## 開発

### ビルド

```bash
cargo build
```

### テスト実行

```bash
cargo test
```

### ローカル実行

```bash
cargo run -- <COMMAND>

# 例：
cargo run -- auth status
cargo run -- new "テスト記事"
```

### コントリビューション

プルリクエストを歓迎します！バグ報告や機能リクエストは[Issues](https://github.com/kako-jun/noet/issues)にお願いします。

開発手順：
1. フォークする
2. フィーチャーブランチを作成 (`git checkout -b feature/amazing-feature`)
3. コミット (`git commit -m 'Add amazing feature'`)
4. プッシュ (`git push origin feature/amazing-feature`)
5. プルリクエストを作成

コミット前に自動的に `cargo fmt` と `cargo clippy` が実行されます。

## 実装済み機能（v0.1.0）

- [x] ワークスペース機能（`.noet/`ディレクトリでプロジェクト管理）
- [x] テンプレート機能（記事テンプレートの作成・管理・使用）
- [x] エクスポート機能（Note.comから記事をMarkdownでダウンロード）
- [x] TUI差分表示（公開前に変更内容を並列比較）
- [x] インタラクティブモード（メニュー駆動のUI）
- [x] エディタ統合（設定可能なエディタ自動起動）

## 今後の予定

以下の機能を実装予定です：

- [ ] プレビュー機能（ブラウザでプレビュー表示）
- [ ] 画像アップロード機能
  - Note.comの画像管理仕様が不明確（要調査）
  - 画像削除可否が不明（ゴミ画像が溜まるリスク）
  - 代替案: Web UIでアップロード→URLをMarkdownに貼り付け
- [ ] 記事の統計情報表示（PV、いいね数など）
- [ ] 記事の検索機能強化
- [ ] crates.ioへの公開

## FAQ

### Q: セッションクッキーはどこに保存されますか？

A: OSのキーリングに安全に保存されます：
- **macOS**: Keychain (AES-256で暗号化)
- **Linux**: Secret Service (libsecret)
- **Windows**: Credential Manager (DPAPI)

平文ファイルには保存されません。

### Q: note.comの公式ツールですか？

A: いいえ、これは**非公式ツール**です。note.comやnote株式会社とは提携していません。

### Q: APIレート制限はありますか？

A: 非公式APIを使用しているため、詳細な制限は不明です。過度な使用は避けてください。

### Q: 複数アカウントを管理できますか？

A: 現在は1アカウントのみです。複数アカウント対応は今後検討します。

### Q: 画像のアップロードはできますか？

A: 現在未対応です。Note.comの画像管理仕様が不明確なため、調査が必要です。
   代替案として、Web UIで画像をアップロードしてURLを取得し、Markdownに貼り付けることができます。

### Q: 記事のバックアップは取れますか？

A: はい、`noet export`コマンドで記事をMarkdown形式でエクスポートできます：
   - 単一記事: `noet export <ARTICLE_KEY> -o article.md`
   - 全記事: `noet export --all --username <USER> -o ./exports/`

## プロジェクト構造

詳細な実装仕様については [CLAUDE.md](CLAUDE.md) を参照してください。

## ライセンス

MIT License - 詳細は [LICENSE](LICENSE) を参照してください。

## 謝辞

- Hugoなどの静的サイトジェネレータからインスピレーションを得ました
- [clap](https://github.com/clap-rs/clap), [reqwest](https://github.com/seanmonstar/reqwest), [tokio](https://github.com/tokio-rs/tokio) を使用しています
- Rustコミュニティに感謝

## 免責事項

これは**非公式ツール**であり、note.comやnote株式会社と提携、承認、または関連はありません。自己責任でご使用ください。
