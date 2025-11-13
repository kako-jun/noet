# noet

[note.com](https://note.com) の記事を管理するためのコマンドラインツール（Rust製）。

`noet`を使うと、Hugoなどの静的サイトジェネレータのように、Markdownファイルでターミナルから記事の作成・投稿・管理ができます。

## 特徴

- **記事管理**: 記事の作成、投稿、編集、削除、一覧表示
- **タグ管理**: ハッシュタグの一覧表示、検索、サジェスト
- **マガジン管理**: 記事のマガジン追加・削除
- **エンゲージメント**: いいね/いいね解除、コメント閲覧
- **ユーザー情報**: ユーザープロフィールと統計情報の取得
- **安全な認証**: 認証情報はシステムキーリングに安全に保存
- **Hugoライクなインターフェース**: frontmatter付きMarkdownベースの記事作成

## インストール

### ソースからビルド

```bash
git clone https://github.com/kako-jun/noet.git
cd noet
cargo install --path .
```

### 必要要件

- Rust 1.70以上
- note.comアカウント

## クイックスタート

### 1. 認証

まず、Noteアカウントで認証します：

```bash
noet auth login
```

以下の手順が必要です：
1. ブラウザで https://note.com にログイン
2. 開発者ツール (F12) → Application/Storage → Cookies を開く
3. `_note_session_v5` クッキーの値をコピー
4. プロンプトに貼り付け

認証状態を確認：

```bash
noet auth status
```

### 2. 新しい記事を作成

```bash
noet new "初めての記事"
```

frontmatter付きのMarkdownファイルが作成されます：

```markdown
---
title: 初めての記事
status: draft
tags: []
---

# 初めての記事

ここに記事を書いてください...
```

### 3. 記事を投稿

```bash
noet publish my-first-article.md
```

下書きとして投稿する場合：

```bash
noet publish my-first-article.md --draft
```

### 4. 記事の一覧表示

```bash
noet list your-username
```

## コマンド

### 記事管理

```bash
# 新しい記事を作成
noet new [TITLE]

# 記事を投稿
noet publish <FILE> [--draft]

# 既存の記事を編集
noet edit <ARTICLE_ID> <FILE>

# 記事を削除
noet delete <ARTICLE_ID> [--force]

# ユーザーの記事一覧
noet list <USERNAME> [--page <PAGE>]
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

設定ファイルは `~/.config/noet/config.toml` に保存されます：

```toml
default_status = "draft"
default_tags = []
base_url = "https://note.com"
```

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

## プロジェクト構造

詳細な実装仕様については [CLAUDE.md](CLAUDE.md) を参照してください。

## コントリビューション

プルリクエストを歓迎します！

## ライセンス

MIT License - 詳細は [LICENSE](LICENSE) を参照してください。

## 謝辞

- Hugoなどの静的サイトジェネレータからインスピレーションを得ました
- [clap](https://github.com/clap-rs/clap), [reqwest](https://github.com/seanmonstar/reqwest), [tokio](https://github.com/tokio-rs/tokio) を使用しています
- Rustコミュニティに感謝

## 免責事項

これは**非公式ツール**であり、note.comやnote株式会社と提携、承認、または関連はありません。自己責任でご使用ください。
