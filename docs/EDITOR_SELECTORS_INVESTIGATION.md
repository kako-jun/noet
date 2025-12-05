# Note.com エディタ セレクタ調査結果

## 調査日: 2025-12-05

## エディタURL

```
https://editor.note.com/notes/{note_id}/edit/
```

## 見出し画像（Header Image / Eyecatch）

### 見出し画像エリア

```html
<div class="sc-ebe7c9bf-0 ipNIAf">
  <img src="https://assets.st-note.com/..." alt="eyecatch" class="h-auto w-full align-top">
</div>
```

**セレクタ**:
- 画像: `img[alt="eyecatch"]`
- コンテナ: `div.sc-ebe7c9bf-0.ipNIAf`

### 見出し画像 削除ボタン

```html
<button class="sc-fd3d5259-2 hlOnlO" aria-label="削除"></button>
```

**セレクタ**: `button[aria-label="削除"]` (見出し画像エリア内)

### 見出し画像の状態

- **画像あり**: 右上に×ボタン（削除ボタン）が表示される
- **画像なし**: クリックでアップロードメニューが開く

### 見出し画像アップロードメニュー

メニュー項目（画像エリアクリック時に表示）:
1. 画像をアップロード
2. 記事にある画像を選ぶ
3. Adobe Expressで画像をつくる

## 本文エディタ

### エディタ本体

```html
<div class="ProseMirror note-common-styles__textnote-body" contenteditable="true">
  <!-- 本文内容 -->
</div>
```

**セレクタ**: `.ProseMirror.note-common-styles__textnote-body`

## 挿入メニュー

### ＋ボタン（メニューを開く）

エディタ内の行をクリックすると、その行の左側に表示される。

```html
<button aria-label="メニューを開く" class="sc-6fa32351-1 dLHPMB">
  <svg viewBox="0 0 24 24" fill="currentColor" xmlns="http://www.w3.org/2000/svg" data-id="Icon" class="inline-flex " focusable="false" width="1em" height="1em" color="var(--color-text-secondary)" aria-hidden="true" role="img" style="font-size: 1.5rem;">
    <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2Z"></path>
  </svg>
</button>
```

**セレクタ**: `button[aria-label="メニューを開く"]`

### 画像挿入ボタン

挿入メニュー内の「画像」項目。

```html
<button data-selected="false" aria-selected="false" class="sc-6fa32351-4 eYVdgL">
  <svg viewBox="0 0 24 24" fill="currentColor" xmlns="http://www.w3.org/2000/svg" data-id="Icon" class="inline-flex " focusable="false" width="1em" height="1em" color="var(--color-text-clickable-icon)" aria-hidden="true" role="img" style="font-size: 1.25rem;">
    <path d="M19 5v14H5V5h14Zm0-2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2Zm-4.86 8.86-3 3.87L9 13.14 6 17h12l-3.86-5.14Z"></path>
  </svg>
  <div class="sc-6fa32351-5 eLWIaJ">画像</div>
</button>
```

**セレクタ**:
- クラス: `button.sc-6fa32351-4.eYVdgL`
- またはテキスト: `button:has-text("画像")` (Playwrightセレクタ)

## 本文内画像

### 画像要素（figure）

```html
<figure name="uuid" id="uuid" class="figcaption-placeholder">
  <img src="https://assets.st-note.com/img/..." alt="" width="620" height="620" contenteditable="false" draggable="false">
  <figcaption>
    <br class="ProseMirror-trailingBreak">
  </figcaption>
</figure>
```

**セレクタ**:
- figure: `.ProseMirror figure`
- 画像: `.ProseMirror figure img`
- figureクラス: `figcaption-placeholder` (キャプションが空の場合)

### キャプション入力欄

```html
<figcaption contenteditable="inherit">
  <!-- キャプションテキスト -->
</figcaption>
```

**セレクタ**: `.ProseMirror figure figcaption`

**注意**: `figcaption` は `contentEditable="inherit"` で、ProseMirrorエディタの一部として編集可能。`textarea` や `input` ではない。

### キャプション設定方法

```javascript
const figcaption = figure.querySelector('figcaption');
figcaption.textContent = 'キャプション文字列';
figcaption.dispatchEvent(new Event('input', { bubbles: true }));
```

## 画像ツールバー

画像を選択すると、上部にツールバーが表示される。

```html
<div role="toolbar" class="sc-fd3d5259-0 caPDjy" id="desktop-toolbar">
  <button aria-label="リンク" class="sc-fd3d5259-2 hlOnlO">...</button>
  <button aria-label="代替テキスト" class="sc-fd3d5259-2 hlOnlO">...</button>
  <button aria-label="縮小" class="sc-fd3d5259-2 hlOnlO">...</button>
  <button aria-label="画像の配置" class="sc-fd3d5259-2 hlOnlO">...</button>
  <button aria-label="削除" class="sc-fd3d5259-2 hlOnlO">...</button>
</div>
```

**セレクタ**: `[role="toolbar"]#desktop-toolbar`

ツールバーボタン:
- リンク: `button[aria-label="リンク"]`
- 代替テキスト: `button[aria-label="代替テキスト"]`
- 縮小: `button[aria-label="縮小"]`
- 画像の配置: `button[aria-label="画像の配置"]`
- 削除: `button[aria-label="削除"]` (画像を削除)

## file input

**TODO**: 「画像」ボタンクリック後のfile input要素の調査が必要。

## 実装への影響

### 見出し画像の更新フロー

1. 既存の見出し画像がある場合は削除ボタンをクリック
2. 見出し画像エリアをクリック
3. 「画像をアップロード」を選択
4. file inputでファイルを選択
5. トリミングUIで範囲を選択
6. 「保存」ボタンをクリック

### 本文内画像の挿入フロー

1. エディタ内の挿入位置をクリック
2. ＋ボタン (`button[aria-label="メニューを開く"]`) をクリック
3. 「画像」ボタン (`button.sc-6fa32351-4.eYVdgL`) をクリック
4. file inputでファイルを選択
5. アップロード完了を待機（新しい`<figure>`要素の出現を監視）
6. キャプションを設定（`figcaption.textContent = "..."`)

### キャプションの設定

```javascript
const newFigure = document.querySelectorAll('.ProseMirror figure')[figureIndex];
const figcaption = newFigure.querySelector('figcaption');
figcaption.textContent = 'キャプション';
figcaption.dispatchEvent(new Event('input', { bubbles: true }));
```

## 注意事項

1. **クラス名の安定性**: `sc-xxxxxx-x` 形式のクラス名は styled-components によって生成されており、ビルドごとに変わる可能性がある。`aria-label` やテキスト内容での検索が推奨される。

2. **動的UI**: 挿入メニューの＋ボタンは、エディタ内の行をクリックしないと表示されない。

3. **contentEditable**: キャプションは `contentEditable` 領域なので、通常の input/textarea とは異なる操作が必要。

4. **見出し画像の差し替え**: 直接差し替え機能はなく、削除してから再アップロードが必要。

5. **file input**: セキュリティ制約により、プログラマティックなファイルパス設定は困難。代替案として Drag & Drop イベントのシミュレートを検討。
