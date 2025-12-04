# Note.com DOM セレクター調査結果

## 調査日: 2025-12-04

---

## 1. ログインページ

**URL**: `https://note.com/login`

### セレクター

| 要素 | セレクター | 備考 |
|------|-----------|------|
| メールアドレス/ID入力 | `#email` | placeholder="mail@example.com or note ID" |
| パスワード入力 | `#password` | type="password" |
| ログインボタン | `button.a-button:last-of-type` または `button:has-text("ログイン")` | type="button"（submitではない） |
| Googleログイン | `.a-button.p-0:first-of-type` | |
| Xログイン | `.a-button.p-0:nth-of-type(2)` | |
| Appleログイン | `.a-button.p-0:nth-of-type(3)` | |

### ログイン情報
- ユーザーID形式: `kako_jun`（アンダースコア）
- 表示名: `kako-jun`（ハイフン）
- **注意**: ログイン時はアンダースコア形式を使用

### 認証後のCookie
- セッションCookieは`httpOnly`で`document.cookie`からは見えない
- `note.com`と`editor.note.com`間でセッション共有される（リンク経由で遷移すれば）

---

## 2. ダッシュボード / ホームページ（ログイン後）

**URL**: `https://note.com/` （ログイン後）

### セレクター

| 要素 | セレクター | 備考 |
|------|-----------|------|
| 投稿ボタン | `a[href="/notes/new"]` | class="a-split-button__left" |
| プロフィールメニュー | `[aria-label="メニュー"]` | 右上のアバター横 |
| 通知アイコン | ヘッダー内のベルアイコン | |

### ユーザーメニュー（プロフィールアイコンクリック後）

| 要素 | セレクター/URL | 備考 |
|------|---------------|------|
| 自分の記事 | `a[href="/notes"]` → `https://note.com/notes` | |
| ダッシュボード | `a[href="/sitesettings/stats"]` | 統計ページ |
| 設定 | `a[href="/settings/account"]` | |
| ログアウト | メニュー内 | |

---

## 3. 自分の記事一覧ページ

**URL**: `https://note.com/notes`

### セレクター

| 要素 | セレクター | 備考 |
|------|-----------|------|
| 記事数表示 | - | "104 記事" のような表示 |
| 公開ステータスフィルター | - | ドロップダウン |
| 期間フィルター | - | ドロップダウン |
| マガジンフィルター | - | ドロップダウン |
| インポートボタン | `button:has-text("インポート")` | |
| エクスポートボタン | `button:has-text("エクスポート")` | |

### 記事リストアイテム

| 要素 | セレクター | 備考 |
|------|-----------|------|
| 記事行の三点メニュー | `[aria-label="その他"]` | class="o-articleList__more" |
| チェックボックス | 記事行の左側 | 複数選択用 |

### 記事メニュー（三点リーダークリック後）

| 要素 | セレクター | 備考 |
|------|-----------|------|
| 編集ボタン | `button.m-basicBalloonList__button:has-text("編集")` | |
| 複製ボタン | `button.m-basicBalloonList__button:has-text("複製")` | |
| 共有リンクコピー | `button.m-basicBalloonList__button:has-text("共有用リンクをコピー")` | |
| 削除ボタン | `button.m-basicBalloonList__button:has-text("削除")` | 赤字 |

### 記事ステータス表示
- `● 下書き` - グレーの丸
- `● 公開中` - 緑の丸

---

## 4. 記事閲覧ページ（公開記事）

**URL**: `https://note.com/{username}/n/{article_key}`

### セレクター

| 要素 | セレクター | 備考 |
|------|-----------|------|
| 記事タイトル | `h1.o-noteContentHeader__title` | |
| 記事本文 | `.note-common-styles__textnote-body` | HTML形式 |
| 公開日時 | `time[datetime]` | ISO 8601形式 |
| ハッシュタグ | `a[href*="/hashtag/"]` | "#タグ名" |
| いいね数 | 記事下部 | |
| 著者名 | ヘッダー内 | |

### 本文HTML構造
```html
<!-- 段落 -->
<p name="uuid" id="uuid">テキスト<br>改行</p>

<!-- 見出し -->
<h2 name="uuid" id="uuid" tabindex="-1">見出しテキスト</h2>

<!-- 画像 -->
<figure name="uuid" id="uuid">
  <a href="..."><img src="..." alt="画像" width="620" height="224"></a>
  <figcaption>キャプション</figcaption>
</figure>

<!-- 外部リンク埋め込み -->
<figure name="uuid" id="uuid" data-src="URL" embedded-service="external-article">
  <div data-name="embedContainer">
    <div data-embed-service="external-article">
      <span>
        <div class="external-article-widget">
          <a href="URL">
            <strong class="external-article-widget-title">タイトル</strong>
            <em class="external-article-widget-description">説明</em>
            <em class="external-article-widget-url">ドメイン</em>
          </a>
        </div>
      </span>
    </div>
  </div>
</figure>
```

---

## 5. 記事エディターページ

**URL**: `https://editor.note.com/new` (新規) または `https://editor.note.com/notes/{id}/edit` (編集)

### 重要な発見
- エディターは別ドメイン `editor.note.com` でホストされている
- Next.jsで構築されている
- `note.com`からリンク経由で遷移する必要がある（直接アクセスだと認証が通らない場合あり）

### セレクター（調査中）
- 編集ボタンクリック後の遷移先を調査する必要あり

---

## 6. URL構造

| ページ | URL パターン |
|--------|-------------|
| ログイン | `https://note.com/login` |
| ホーム | `https://note.com/` |
| ユーザーページ | `https://note.com/{username}` |
| 記事閲覧 | `https://note.com/{username}/n/{article_key}` |
| 自分の記事一覧 | `https://note.com/notes` |
| 新規作成 | `https://note.com/notes/new` → `https://editor.note.com/new` |
| 記事編集 | `https://editor.note.com/notes/{id}/edit` (推定) |
| 設定 | `https://note.com/settings/account` |
| 統計 | `https://note.com/sitesettings/stats` |

---

## 7. 技術スタック

- **メインサイト** (`note.com`): Vue.js (data-v-* 属性)
- **エディター** (`editor.note.com`): Next.js (__NEXT_DATA__)
- **スタイリング**: Tailwind CSS (一部), 独自クラス

---

## 8. 拡張機能実装上の注意点

1. **ログイン状態確認**: 右上の「投稿」ボタンまたはプロフィールアイコンの存在で判定
2. **エディターへのアクセス**: `note.com`ドメインからリンク経由で遷移すること
3. **記事編集**: 「自分の記事」→ 三点メニュー → 「編集」の順でアクセス
4. **DOM待機**: Vue/Next.jsのSPA構造のため、要素の読み込み待機が必要
5. **Bot検出**: HeadlessChrome検出の可能性あり。実際のブラウザ拡張なら問題なし

---

## 9. 遷移時のウェイト（推奨値）

| 操作 | 待機時間 | 備考 |
|------|---------|------|
| ログインボタンクリック後 | 4000ms | ログイン処理＋リダイレクト |
| ページ遷移後 | 3000ms | SPAのコンテンツ読み込み |
| メニュークリック後 | 500ms | アニメーション完了 |
| エディターページ読み込み | 10000ms以上 | Next.jsの初期化、認証確認 |
| フォーム入力後 | 500ms | バリデーション処理 |

### 待機の実装例（JavaScript）
```javascript
// 固定待機
await new Promise(resolve => setTimeout(resolve, 3000));

// 要素待機（推奨）
await page.waitForSelector('.note-common-styles__textnote-body', { timeout: 10000 });

// 複数条件
await Promise.race([
  page.waitForSelector('#editor'),
  page.waitForSelector('.error-message'),
  new Promise(resolve => setTimeout(resolve, 15000))
]);
```

---

## 10. 記事エディターページ（追加調査済み）

**URL**: `https://editor.note.com/notes/{note_id}/edit/`

### セレクター

| 要素 | セレクター | 備考 |
|------|-----------|------|
| タイトル入力 | `textarea[placeholder="記事タイトル"]` | class="sc-80832eb4-0 heevId" |
| 本文エディター | `.ProseMirror.note-common-styles__textnote-body` | contenteditable="true", role="textbox" |
| 閉じるボタン | `button:has-text("閉じる")` | ヘッダー左 |
| 下書き保存ボタン | `button:has-text("下書き保存")` | ヘッダー右 |
| 公開に進むボタン | `button:has-text("公開に進む")` | ヘッダー右端 |
| 文字数カウンター | テキスト「X 文字」 | 右上 |
| AIアシスタント | `button[aria-label="AIアシスタント"]` | 左サイドバー |

### 本文エディター詳細（ProseMirror）
```html
<div translate="no"
     role="textbox"
     aria-multiline="true"
     contenteditable="true"
     class="ProseMirror note-common-styles__textnote-body">
  <!-- 本文HTML -->
</div>
```

### 公開フロー
1. タイトル・本文が空の場合、「公開に進む」クリックでエラーダイアログ表示
2. エラーダイアログ: 「タイトル、本文を入力してください」+ 「閉じる」ボタン
3. モーダルセレクター: `.ReactModal__Content`

---

## 11. 記事取得フロー（実装優先）

### 手順
1. `https://note.com/login` でログイン
2. `https://note.com/notes` で自分の記事一覧を取得
3. 各記事の三点メニュー → 編集 → エディターページでHTML取得

### 記事一覧取得
```javascript
// 1. note.com/notes にアクセス
// 2. 記事リストを取得
const articles = Array.from(document.querySelectorAll('.o-articleList__item')).map(item => ({
  title: item.querySelector('.o-articleList__title')?.textContent?.trim(),
  status: item.querySelector('.o-articleList__status')?.textContent?.trim(), // "下書き" or "公開中"
  date: item.querySelector('.o-articleList__date')?.textContent?.trim(),
  moreButton: item.querySelector('[aria-label="その他"]')
}));
```

### 単一記事のHTML取得（エディター経由）
```javascript
// エディターページで本文HTMLを取得
const bodyHtml = document.querySelector('.ProseMirror.note-common-styles__textnote-body')?.innerHTML;
const title = document.querySelector('textarea[placeholder="記事タイトル"]')?.value;
```

### 公開記事からの取得（ログイン不要）
```javascript
// https://note.com/{username}/n/{article_key}
const title = document.querySelector('h1.o-noteContentHeader__title')?.textContent?.trim();
const bodyHtml = document.querySelector('.note-common-styles__textnote-body')?.innerHTML;
const tags = Array.from(document.querySelectorAll('a[href*="/hashtag/"]')).map(a => a.textContent?.trim());
const publishDate = document.querySelector('time[datetime]')?.getAttribute('datetime');
```

---

## 次に調査が必要な項目

- [ ] タグ入力の仕組み（公開設定画面）
- [ ] 公開ボタンの最終セレクター
- [ ] 記事作成フロー（新規投稿）
