// Investigate publish dialog on Note.com
// Run with: xvfb-run npx playwright test --headed

const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

const SCREENSHOTS_DIR = path.join(__dirname, '../screenshots/publish-dialog');

async function main() {
  // Create screenshots directory
  if (!fs.existsSync(SCREENSHOTS_DIR)) {
    fs.mkdirSync(SCREENSHOTS_DIR, { recursive: true });
  }

  const browser = await chromium.launch({
    headless: false,
    slowMo: 500
  });

  const context = await browser.newContext({
    viewport: { width: 1280, height: 800 }
  });

  const page = await context.newPage();

  try {
    // Step 1: Login
    console.log('Step 1: Logging in...');
    await page.goto('https://note.com/login');
    await page.waitForLoadState('networkidle');
    await page.screenshot({ path: path.join(SCREENSHOTS_DIR, '01_login_page.png') });

    await page.fill('#email', 'kako_jun');
    await page.fill('#password', '14avanstrashabx');
    await page.click('button:has-text("ログイン")');

    await page.waitForURL('https://note.com/**', { timeout: 10000 });
    await page.waitForLoadState('networkidle');
    await page.screenshot({ path: path.join(SCREENSHOTS_DIR, '02_after_login.png') });
    console.log('Logged in successfully');

    // Step 2: Navigate to new article
    console.log('Step 2: Going to editor...');
    await page.goto('https://note.com/notes/new');
    await page.waitForURL('**/editor.note.com/**', { timeout: 15000 });
    await page.waitForLoadState('networkidle');

    // Wait for editor to load
    await page.waitForTimeout(5000);
    await page.screenshot({ path: path.join(SCREENSHOTS_DIR, '03_editor_initial.png') });

    // Wait for title input
    const titleInput = await page.waitForSelector('textarea[placeholder="記事タイトル"]', { timeout: 15000 });
    await page.screenshot({ path: path.join(SCREENSHOTS_DIR, '04_editor_loaded.png') });
    console.log('Editor loaded');

    // Step 3: Fill in content
    console.log('Step 3: Filling content...');
    await titleInput.fill('テスト記事タイトル');
    await page.waitForTimeout(500);

    // Fill body
    const bodyEditor = await page.waitForSelector('.ProseMirror.note-common-styles__textnote-body');
    await bodyEditor.click();
    await page.keyboard.type('これはテスト本文です。');
    await page.waitForTimeout(500);
    await page.screenshot({ path: path.join(SCREENSHOTS_DIR, '05_content_filled.png') });
    console.log('Content filled');

    // Step 4: Click "公開に進む" button
    console.log('Step 4: Clicking publish button...');
    const publishBtn = await page.waitForSelector('button:has-text("公開に進む")');
    await publishBtn.click();
    await page.waitForTimeout(2000);
    await page.screenshot({ path: path.join(SCREENSHOTS_DIR, '06_publish_dialog.png') });
    console.log('Publish dialog opened');

    // Step 5: Get dialog HTML structure
    console.log('Step 5: Analyzing dialog structure...');

    const dialogInfo = await page.evaluate(() => {
      const modal = document.querySelector('[role="dialog"], .ReactModal__Content, [class*="modal"], [class*="Modal"]');
      if (!modal) {
        // Look for any overlay or popup
        const overlays = document.querySelectorAll('[class*="overlay"], [class*="Overlay"], [class*="popup"], [class*="Popup"]');
        return {
          found: false,
          overlayCount: overlays.length,
          bodyHTML: document.body.innerHTML.substring(0, 5000)
        };
      }

      // Get all input elements in dialog
      const inputs = modal.querySelectorAll('input, textarea');
      const inputInfo = Array.from(inputs).map(inp => ({
        type: inp.type,
        placeholder: inp.placeholder,
        name: inp.name,
        className: inp.className,
        id: inp.id
      }));

      // Get all buttons in dialog
      const buttons = modal.querySelectorAll('button');
      const buttonInfo = Array.from(buttons).map(btn => ({
        text: btn.textContent?.trim(),
        className: btn.className,
        type: btn.type
      }));

      return {
        found: true,
        modalClass: modal.className,
        modalRole: modal.getAttribute('role'),
        inputs: inputInfo,
        buttons: buttonInfo,
        innerHTML: modal.innerHTML.substring(0, 3000)
      };
    });

    // Save dialog info
    fs.writeFileSync(
      path.join(SCREENSHOTS_DIR, '07_dialog_structure.json'),
      JSON.stringify(dialogInfo, null, 2)
    );
    console.log('Dialog structure saved');
    console.log(JSON.stringify(dialogInfo, null, 2));

    // Step 6: Look for tag input specifically
    const tagInputInfo = await page.evaluate(() => {
      // Common selectors for tag inputs
      const selectors = [
        'input[placeholder*="タグ"]',
        'input[placeholder*="ハッシュタグ"]',
        'input[placeholder*="tag"]',
        'input[name*="tag"]',
        '[class*="tag"] input',
        '[class*="Tag"] input',
        '[data-testid*="tag"]'
      ];

      for (const sel of selectors) {
        const el = document.querySelector(sel);
        if (el) {
          return {
            found: true,
            selector: sel,
            placeholder: el.placeholder,
            className: el.className
          };
        }
      }

      return { found: false, triedSelectors: selectors };
    });

    console.log('Tag input info:', JSON.stringify(tagInputInfo, null, 2));
    fs.writeFileSync(
      path.join(SCREENSHOTS_DIR, '08_tag_input_info.json'),
      JSON.stringify(tagInputInfo, null, 2)
    );

    // Take one more screenshot after some time
    await page.waitForTimeout(1000);
    await page.screenshot({ path: path.join(SCREENSHOTS_DIR, '09_dialog_final.png'), fullPage: true });

    console.log('Investigation complete!');
    console.log(`Screenshots saved to: ${SCREENSHOTS_DIR}`);

  } catch (error) {
    console.error('Error:', error);
    await page.screenshot({ path: path.join(SCREENSHOTS_DIR, 'error.png') });
  } finally {
    await browser.close();
  }
}

main();
