// noet Browser Extension - Background Service Worker
//
// Communicates with CLI via Native Messaging or WebSocket
// Executes Note.com operations using DOM scraping (no API)

const VERSION = "0.1.7";
const NATIVE_HOST_NAME = "com.noet.host";
const WEBSOCKET_URL = "ws://127.0.0.1:9876";

// Debug mode: when true, opens tabs visibly for DOM operations
let debugMode = false;

// WebSocket connection
let ws = null;
let wsReconnectTimer = null;

/**
 * Human-like behavior utilities
 * Mimics natural human browsing patterns to avoid bot detection
 */

// Random delay between min and max milliseconds (human-like variation)
function randomDelay(min, max) {
  const delay = Math.floor(Math.random() * (max - min + 1)) + min;
  return new Promise(resolve => setTimeout(resolve, delay));
}

// Typing delay simulation (humans don't type instantly)
function humanTypingDelay(textLength) {
  // Average human typing: 40-60 WPM = 200-300ms per character with variation
  const baseDelay = textLength * 80; // ~80ms per char average
  const variation = Math.random() * 0.4 + 0.8; // 0.8x to 1.2x variation
  return Math.floor(baseDelay * variation);
}

// Page load wait with natural variation (humans don't react instantly)
async function humanPageLoadWait() {
  // Humans take 1-3 seconds to start interacting after page load
  await randomDelay(1500, 3500);
}

// Short pause between actions (humans pause to read/think)
async function humanActionPause() {
  await randomDelay(300, 800);
}

// Native messaging port
let port = null;

/**
 * Connect to Native Messaging host
 */
function connectToNativeHost() {
  try {
    port = chrome.runtime.connectNative(NATIVE_HOST_NAME);

    port.onMessage.addListener((message) => {
      console.log("[noet] Received from host:", message);
      handleHostMessage(message);
    });

    port.onDisconnect.addListener(() => {
      console.log("[noet] Disconnected from host");
      if (chrome.runtime.lastError) {
        console.error("[noet] Disconnect error:", chrome.runtime.lastError.message);
      }
      port = null;
      // Reconnect after a delay
      setTimeout(connectToNativeHost, 1000);
    });

    console.log("[noet] Connected to Native Host");
  } catch (e) {
    console.error("[noet] Failed to connect to Native Host:", e);
  }
}

/**
 * Send message to Native Host
 */
function sendToHost(message) {
  if (port) {
    port.postMessage(message);
  } else {
    console.error("[noet] Not connected to host");
  }
}

/**
 * Handle message from Native Host (CLI requests)
 */
async function handleHostMessage(request) {
  const { id, command, params = {} } = request;

  try {
    let result;

    switch (command) {
      case "ping":
        result = await handlePing();
        break;

      case "check_auth":
        result = await handleCheckAuth();
        break;

      case "list_articles":
        result = await handleListArticles(params);
        break;

      case "get_article":
        result = await handleGetArticle(params);
        break;

      case "create_article":
        result = await handleCreateArticle(params);
        break;

      case "update_article":
        result = await handleUpdateArticle(params);
        break;

      case "delete_article":
        result = await handleDeleteArticle(params);
        break;

      case "set_debug_mode":
        debugMode = params.enabled;
        result = { success: true, debug_mode: debugMode };
        break;

      case "get_debug_mode":
        result = { debug_mode: debugMode };
        break;

      default:
        throw new Error(`Unknown command: ${command}`);
    }

    sendToHost({
      id,
      status: "success",
      data: result
    });

  } catch (e) {
    sendToHost({
      id,
      status: "error",
      error: {
        code: e.code || "UNKNOWN",
        message: e.message
      }
    });
  }
}

/**
 * Command handlers - All use DOM scraping, no API calls
 */

async function handlePing() {
  return {
    version: VERSION,
    extension_id: chrome.runtime.id
  };
}

async function handleCheckAuth() {
  // Check login by visiting note.com and looking for login indicators
  return await executeInTab("https://note.com/", async (tabId) => {
    await waitForTabLoad(tabId);
    await humanPageLoadWait(); // Human-like wait after page load

    const result = await chrome.scripting.executeScript({
      target: { tabId },
      func: scrapeLoginStatus
    });

    return result[0].result;
  });
}

async function handleListArticles(params) {
  // Scrape own articles from /notes page
  return await executeInTab("https://note.com/notes", async (tabId) => {
    await waitForTabLoad(tabId);
    await humanPageLoadWait();
    await randomDelay(500, 1500); // Extra wait for SPA content

    const result = await chrome.scripting.executeScript({
      target: { tabId },
      func: scrapeArticleList
    });

    return result[0].result;
  });
}

async function handleGetArticle(params) {
  const { key, username } = params;

  if (!username || !key) {
    const error = new Error("username and key are required");
    error.code = "INVALID_PARAMS";
    throw error;
  }

  // Scrape article from public page
  const articleUrl = `https://note.com/${username}/n/${key}`;
  return await executeInTab(articleUrl, async (tabId) => {
    await waitForTabLoad(tabId);
    await humanPageLoadWait();
    await randomDelay(300, 1000); // Natural reading delay

    const result = await chrome.scripting.executeScript({
      target: { tabId },
      func: scrapeArticlePage
    });

    return result[0].result;
  });
}

async function handleCreateArticle(params) {
  const { title, body, tags = [], magazines = [], draft = false, images = [], header_image = null } = params;

  // Navigate via note.com/notes/new which redirects to editor.note.com
  return await executeInTab("https://note.com/notes/new", async (tabId) => {
    await waitForTabLoad(tabId);
    await humanPageLoadWait();

    // Wait for editor page to load (it's a redirect to editor.note.com)
    await randomDelay(3000, 5000);

    // Wait for editor elements
    await waitForElement(tabId, 'textarea[placeholder="記事タイトル"]', 15000);
    await randomDelay(500, 1000);

    // Step 1: Fill the form (with or without images)
    let fillResult;
    if (images.length > 0 || header_image) {
      fillResult = await chrome.scripting.executeScript({
        target: { tabId },
        func: fillArticleFormWithImages,
        args: [title, body, images, header_image]
      });
    } else {
      fillResult = await chrome.scripting.executeScript({
        target: { tabId },
        func: fillArticleForm,
        args: [title, body]
      });
    }

    if (!fillResult[0].result.success) {
      return fillResult[0].result;
    }

    const uploadedImages = fillResult[0].result.uploaded_images || [];
    const headerImageUrl = fillResult[0].result.header_image_url || null;

    await randomDelay(500, 1500);

    // Step 2: Save as draft or proceed to publish
    if (draft) {
      // Click draft save button
      const draftResult = await chrome.scripting.executeScript({
        target: { tabId },
        func: clickDraftSaveButton
      });

      if (!draftResult[0].result.success) {
        return draftResult[0].result;
      }

      // Wait for save to complete
      await randomDelay(2000, 3000);

      return {
        success: true,
        status: "draft",
        message: "Article saved as draft",
        uploaded_images: uploadedImages,
        header_image_url: headerImageUrl
      };
    } else {
      // Click "公開に進む" button - this navigates to /publish/ page (not a dialog!)
      const publishResult = await chrome.scripting.executeScript({
        target: { tabId },
        func: clickPublishProceedButton
      });

      if (!publishResult[0].result.success) {
        return publishResult[0].result;
      }

      // Wait for navigation to /publish/ page
      await waitForTabLoad(tabId);
      await randomDelay(2000, 4000);

      // Wait for publish page elements
      await waitForElement(tabId, 'input[placeholder="ハッシュタグを追加する"]', 15000);
      await randomDelay(500, 1000);

      // Step 3: Fill tags, add to magazines, and click final publish on /publish/ page
      const finalResult = await chrome.scripting.executeScript({
        target: { tabId },
        func: handlePublishPage,
        args: [tags, magazines]
      });

      if (!finalResult[0].result.success) {
        return finalResult[0].result;
      }

      // Wait for publish to complete and redirect
      await randomDelay(3000, 5000);

      // Try to get the new article URL
      const urlResult = await chrome.scripting.executeScript({
        target: { tabId },
        func: () => window.location.href
      });

      return {
        success: true,
        status: "published",
        url: urlResult[0].result,
        message: "Article published successfully",
        uploaded_images: uploadedImages,
        header_image_url: headerImageUrl
      };
    }
  });
}

async function handleUpdateArticle(params) {
  const { key, title, body, tags, magazines = [], draft = false, images = [], header_image = null } = params;

  // First go to /notes, find the article, click edit
  return await executeInTab("https://note.com/notes", async (tabId) => {
    await waitForTabLoad(tabId);
    await humanPageLoadWait();
    await randomDelay(500, 1500); // Wait for article list to render

    // Step 1: Find and click the article's more menu
    const findResult = await chrome.scripting.executeScript({
      target: { tabId },
      func: findArticleAndClickMore,
      args: [key]
    });

    if (!findResult[0].result.success) {
      return findResult[0].result;
    }

    // Wait for menu to appear
    await randomDelay(500, 800);

    // Step 2: Click edit button in menu
    const editResult = await chrome.scripting.executeScript({
      target: { tabId },
      func: clickEditInMenu
    });

    if (!editResult[0].result.success) {
      return editResult[0].result;
    }

    // Wait for navigation to editor
    await waitForTabLoad(tabId);
    await humanPageLoadWait();
    await randomDelay(2000, 4000);

    // Wait for editor elements
    await waitForElement(tabId, 'textarea[placeholder="記事タイトル"]', 15000);
    await randomDelay(500, 1000);

    // Step 3: Fill the form with new content (with or without images)
    let fillResult;
    if (images.length > 0 || header_image) {
      fillResult = await chrome.scripting.executeScript({
        target: { tabId },
        func: fillArticleFormWithImages,
        args: [title, body, images, header_image]
      });
    } else {
      fillResult = await chrome.scripting.executeScript({
        target: { tabId },
        func: fillArticleForm,
        args: [title, body]
      });
    }

    if (!fillResult[0].result.success) {
      return fillResult[0].result;
    }

    const uploadedImages = fillResult[0].result.uploaded_images || [];
    const headerImageUrl = fillResult[0].result.header_image_url || null;

    await randomDelay(500, 1500);

    // Step 4: Save changes
    if (draft) {
      const draftResult = await chrome.scripting.executeScript({
        target: { tabId },
        func: clickDraftSaveButton
      });

      if (!draftResult[0].result.success) {
        return draftResult[0].result;
      }

      await randomDelay(2000, 3000);

      return {
        success: true,
        status: "draft",
        message: "Article updated and saved as draft",
        uploaded_images: uploadedImages,
        header_image_url: headerImageUrl
      };
    } else {
      // Click "公開に進む" - navigates to /publish/ page
      const publishResult = await chrome.scripting.executeScript({
        target: { tabId },
        func: clickPublishOrUpdateButton
      });

      if (!publishResult[0].result.success) {
        return publishResult[0].result;
      }

      // Wait for navigation to /publish/ page
      await waitForTabLoad(tabId);
      await randomDelay(2000, 4000);

      // Wait for publish page elements
      await waitForElement(tabId, 'input[placeholder="ハッシュタグを追加する"]', 15000);
      await randomDelay(500, 1000);

      // Handle publish page
      const finalResult = await chrome.scripting.executeScript({
        target: { tabId },
        func: handlePublishPage,
        args: [tags || [], magazines]
      });

      await randomDelay(3000, 5000);

      return {
        success: true,
        status: "updated",
        message: "Article updated successfully",
        uploaded_images: uploadedImages,
        header_image_url: headerImageUrl
      };
    }
  });
}

async function handleDeleteArticle(params) {
  const { key } = params;

  return await executeInTab("https://note.com/notes", async (tabId) => {
    await waitForTabLoad(tabId);
    await humanPageLoadWait();
    await randomDelay(500, 1500); // Wait for article list to render

    // Step 1: Find and click the article's more menu
    const findResult = await chrome.scripting.executeScript({
      target: { tabId },
      func: findArticleAndClickMore,
      args: [key]
    });

    if (!findResult[0].result.success) {
      return findResult[0].result;
    }

    // Wait for menu to appear
    await randomDelay(500, 800);

    // Step 2: Click delete button in menu
    const deleteResult = await chrome.scripting.executeScript({
      target: { tabId },
      func: clickDeleteInMenu
    });

    if (!deleteResult[0].result.success) {
      return deleteResult[0].result;
    }

    // Wait for confirmation dialog
    await randomDelay(500, 800);

    // Step 3: Confirm deletion
    const confirmResult = await chrome.scripting.executeScript({
      target: { tabId },
      func: confirmDeleteDialog
    });

    if (!confirmResult[0].result.success) {
      return confirmResult[0].result;
    }

    // Wait for deletion to complete
    await randomDelay(2000, 3000);

    return {
      success: true,
      message: "Article deleted successfully"
    };
  });
}

/**
 * Execute operation in a tab
 */
async function executeInTab(url, operation) {
  const tab = await chrome.tabs.create({
    url,
    active: debugMode
  });

  try {
    const result = await operation(tab.id);

    if (!debugMode) {
      await chrome.tabs.remove(tab.id);
    }

    return result;
  } catch (e) {
    if (!debugMode) {
      try {
        await chrome.tabs.remove(tab.id);
      } catch (_) {}
    }
    throw e;
  }
}

/**
 * Wait for tab to finish loading
 */
function waitForTabLoad(tabId) {
  return new Promise((resolve) => {
    chrome.tabs.onUpdated.addListener(function listener(id, info) {
      if (id === tabId && info.status === "complete") {
        chrome.tabs.onUpdated.removeListener(listener);
        resolve();
      }
    });
  });
}

/**
 * Wait for element to appear in tab
 */
async function waitForElement(tabId, selector, timeout = 10000) {
  const startTime = Date.now();

  while (Date.now() - startTime < timeout) {
    try {
      const result = await chrome.scripting.executeScript({
        target: { tabId },
        func: (sel) => !!document.querySelector(sel),
        args: [selector]
      });

      if (result[0].result) {
        return true;
      }
    } catch (e) {
      // Tab might not be ready yet
    }

    await randomDelay(200, 500);
  }

  throw new Error(`Element ${selector} not found within ${timeout}ms`);
}

/**
 * DOM Scraping Functions - Injected into pages
 * Based on Playwright DOM investigation results
 */

// Check if user is logged in
function scrapeLoginStatus() {
  // Look for "投稿" button which only appears when logged in
  const postButton = document.querySelector('a[href="/notes/new"]');
  // Look for profile avatar in header
  const profileAvatar = document.querySelector('[class*="navbarPrimary"] img, [class*="avatar"]');

  const loggedIn = !!(postButton || profileAvatar);

  // Try to get username from profile link if available
  let username = null;
  const profileLink = document.querySelector('a[href^="/"][class*="profile"], a[href^="/kako"]');
  if (profileLink) {
    const match = profileLink.href.match(/note\.com\/([^\/\?]+)/);
    if (match) username = match[1];
  }

  return {
    logged_in: loggedIn,
    username: username
  };
}

// Scrape article list from /notes page
function scrapeArticleList() {
  const articles = [];

  // Find all article rows - they have title, status, date, and more button
  const rows = document.querySelectorAll('[class*="articleList"] > div, .o-articleList__item');

  // Alternative: look for elements with "その他" button
  const moreButtons = document.querySelectorAll('[aria-label="その他"]');

  moreButtons.forEach((btn, index) => {
    const row = btn.closest('div[class*="item"], div[class*="row"], li') || btn.parentElement?.parentElement?.parentElement;
    if (!row) return;

    // Get title - usually in a link or heading
    const titleEl = row.querySelector('a[href*="/n/"], [class*="title"], h3, h4');
    const title = titleEl?.textContent?.trim() || '';

    // Get article key from link
    let key = null;
    const link = row.querySelector('a[href*="/n/"]');
    if (link) {
      const match = link.href.match(/\/n\/([^\/\?]+)/);
      if (match) key = match[1];
    }

    // Get status (下書き or 公開中)
    const statusEl = row.querySelector('[class*="status"], span');
    let status = 'unknown';
    const rowText = row.textContent || '';
    if (rowText.includes('下書き')) {
      status = 'draft';
    } else if (rowText.includes('公開中')) {
      status = 'published';
    }

    // Get date
    const dateEl = row.querySelector('time, [class*="date"]');
    const date = dateEl?.textContent?.trim() || dateEl?.getAttribute('datetime') || '';

    if (title || key) {
      articles.push({
        key,
        title,
        status,
        date
      });
    }
  });

  return {
    articles,
    count: articles.length
  };
}

// Scrape article content from public article page
function scrapeArticlePage() {
  // Title: h1.o-noteContentHeader__title
  const titleEl = document.querySelector('h1.o-noteContentHeader__title');
  const title = titleEl?.textContent?.trim() || '';

  // Body HTML: .note-common-styles__textnote-body
  const bodyEl = document.querySelector('.note-common-styles__textnote-body');
  const html = bodyEl?.innerHTML || '';

  // Tags: a[href*="/hashtag/"]
  const tagEls = document.querySelectorAll('a[href*="/hashtag/"]');
  const tags = Array.from(tagEls).map(a => {
    const text = a.textContent?.trim() || '';
    // Remove # prefix if present
    return text.startsWith('#') ? text.slice(1) : text;
  }).filter(Boolean);

  // Publish date: time[datetime]
  const timeEl = document.querySelector('time[datetime]');
  const publishedAt = timeEl?.getAttribute('datetime') || '';

  // Check if article was found
  if (!title && !html) {
    return {
      error: "Article not found or page did not load",
      success: false
    };
  }

  return {
    success: true,
    title,
    html,
    tags,
    published_at: publishedAt
  };
}

/**
 * Human-like input helper functions
 */
function humanFocus(element) {
  element.focus();
  element.dispatchEvent(new FocusEvent('focus', { bubbles: true }));
}

function humanBlur(element) {
  element.blur();
  element.dispatchEvent(new FocusEvent('blur', { bubbles: true }));
}

function humanClick(element) {
  const rect = element.getBoundingClientRect();
  const x = rect.left + rect.width / 2;
  const y = rect.top + rect.height / 2;

  element.dispatchEvent(new MouseEvent('mouseenter', { bubbles: true, clientX: x, clientY: y }));
  element.dispatchEvent(new MouseEvent('mouseover', { bubbles: true, clientX: x, clientY: y }));
  element.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, clientX: x, clientY: y }));
  element.dispatchEvent(new MouseEvent('mouseup', { bubbles: true, clientX: x, clientY: y }));
  element.dispatchEvent(new MouseEvent('click', { bubbles: true, clientX: x, clientY: y }));
}

function humanInput(element, value) {
  humanFocus(element);

  if (element.tagName === 'TEXTAREA' || element.tagName === 'INPUT') {
    element.value = value;
  } else if (element.isContentEditable) {
    element.innerHTML = value;
  }

  element.dispatchEvent(new Event('input', { bubbles: true, composed: true }));
  element.dispatchEvent(new Event('change', { bubbles: true }));

  humanBlur(element);
}

/**
 * Editor Form Functions - injected into editor.note.com
 */

// Base64 to Blob conversion utility
function base64ToBlob(base64, mimeType) {
  const byteCharacters = atob(base64);
  const byteArrays = [];

  for (let offset = 0; offset < byteCharacters.length; offset += 512) {
    const slice = byteCharacters.slice(offset, offset + 512);
    const byteNumbers = new Array(slice.length);
    for (let i = 0; i < slice.length; i++) {
      byteNumbers[i] = slice.charCodeAt(i);
    }
    const byteArray = new Uint8Array(byteNumbers);
    byteArrays.push(byteArray);
  }

  return new Blob(byteArrays, { type: mimeType });
}

// Wait for condition with timeout
function waitForCondition(condition, timeout = 10000) {
  return new Promise((resolve, reject) => {
    const startTime = Date.now();
    const interval = setInterval(() => {
      if (condition()) {
        clearInterval(interval);
        resolve();
      } else if (Date.now() - startTime > timeout) {
        clearInterval(interval);
        reject(new Error('Timeout waiting for condition'));
      }
    }, 100);
  });
}

// Upload header image (eyecatch)
async function uploadHeaderImage(imageData, filename) {
  try {
    // Convert base64 to Blob
    const blob = base64ToBlob(imageData.data, imageData.mime_type);
    const file = new File([blob], filename, { type: imageData.mime_type });

    // Create DataTransfer
    const dataTransfer = new DataTransfer();
    dataTransfer.items.add(file);

    // Click "画像を追加" button
    const addImageButton = document.querySelector('button[aria-label="画像を追加"]');
    if (!addImageButton) {
      throw new Error('Header image add button not found');
    }
    addImageButton.click();

    // Wait for upload menu to appear
    await new Promise(resolve => setTimeout(resolve, 300));

    // Click "画像をアップロード" button
    const buttons = Array.from(document.querySelectorAll('button'));
    const uploadButton = buttons.find(btn => {
      const div = btn.querySelector('div');
      return div && div.textContent.trim() === '画像をアップロード';
    });
    if (!uploadButton) {
      throw new Error('Upload button not found');
    }
    uploadButton.click();

    // Wait for file input
    await new Promise(resolve => setTimeout(resolve, 300));

    // Find file input
    const fileInput = document.querySelector('input[type="file"]');
    if (!fileInput) {
      throw new Error('File input not found');
    }

    // Set file
    fileInput.files = dataTransfer.files;
    fileInput.dispatchEvent(new Event('change', { bubbles: true }));

    // Wait for crop UI and save button to appear
    await waitForCondition(() => {
      const saveButton = Array.from(document.querySelectorAll('button')).find(btn =>
        btn.textContent.includes('保存')
      );
      return !!saveButton;
    }, 10000);

    // Click save button (accept default crop)
    const saveButton = Array.from(document.querySelectorAll('button')).find(btn =>
      btn.textContent.includes('保存')
    );
    if (!saveButton) {
      throw new Error('Save button not found');
    }
    saveButton.click();

    // Wait for upload to complete (image appears)
    await waitForCondition(() => {
      const img = document.querySelector('img[alt="eyecatch"]');
      return img && img.src && !img.src.startsWith('blob:') && img.src.includes('st-note.com');
    }, 15000);

    // Get uploaded image URL
    const img = document.querySelector('img[alt="eyecatch"]');
    const noteUrl = img.src;

    return {
      success: true,
      url: noteUrl
    };

  } catch (e) {
    return {
      success: false,
      error: e.message
    };
  }
}

// Remove header image
async function removeHeaderImage() {
  try {
    // Find delete button (× button with aria-label="削除")
    const deleteIcon = document.querySelector('[role="img"][aria-label="削除"]');
    if (!deleteIcon) {
      return { success: true, message: 'No header image to remove' };
    }

    // Find the button containing this icon
    const deleteButton = deleteIcon.closest('button');
    if (!deleteButton) {
      throw new Error('Delete button not found');
    }

    deleteButton.click();

    // Wait for image to be removed
    await waitForCondition(() => {
      const img = document.querySelector('img[alt="eyecatch"]');
      return !img;
    }, 5000);

    return {
      success: true,
      message: 'Header image removed'
    };

  } catch (e) {
    return {
      success: false,
      error: e.message
    };
  }
}

// Upload a single content image to the editor
async function uploadImage(imageData, filename, caption) {
  try {
    // Convert base64 to Blob
    const blob = base64ToBlob(imageData.data, imageData.mime_type);
    const file = new File([blob], filename, { type: imageData.mime_type });

    // Create DataTransfer
    const dataTransfer = new DataTransfer();
    dataTransfer.items.add(file);

    // Get editor
    const editor = document.querySelector('.ProseMirror.note-common-styles__textnote-body');
    if (!editor) {
      throw new Error('Editor not found');
    }

    // Focus editor at the end
    editor.focus();
    const range = document.createRange();
    range.selectNodeContents(editor);
    range.collapse(false);
    const selection = window.getSelection();
    selection.removeAllRanges();
    selection.addRange(range);

    // Click + button (menu button)
    const plusButton = document.querySelector('button[aria-label="メニューを開く"]');
    if (!plusButton) {
      throw new Error('Plus button not found - click in editor first');
    }
    plusButton.click();

    // Wait for menu to appear
    await new Promise(resolve => setTimeout(resolve, 300));

    // Click image button
    const imageButtons = Array.from(document.querySelectorAll('button'));
    const imageButton = imageButtons.find(btn => btn.textContent.trim() === '画像');
    if (!imageButton) {
      throw new Error('Image button not found in menu');
    }
    imageButton.click();

    // Wait for file input
    await new Promise(resolve => setTimeout(resolve, 300));

    // Find file input
    const fileInput = document.querySelector('input[type="file"]');
    if (!fileInput) {
      throw new Error('File input not found');
    }

    // Set file
    fileInput.files = dataTransfer.files;
    fileInput.dispatchEvent(new Event('change', { bubbles: true }));

    // Wait for upload to complete (new figure appears)
    const initialFigureCount = document.querySelectorAll('.ProseMirror figure').length;
    await waitForCondition(() => {
      return document.querySelectorAll('.ProseMirror figure').length > initialFigureCount;
    }, 15000);

    // Get the newly added figure
    const figures = document.querySelectorAll('.ProseMirror figure');
    const newFigure = figures[figures.length - 1];
    const img = newFigure.querySelector('img');

    // Wait for image URL to be set (not blob:)
    await waitForCondition(() => {
      return img.src && !img.src.startsWith('blob:') && img.src.includes('st-note.com');
    }, 15000);

    const noteUrl = img.src;

    // Set caption if provided
    if (caption) {
      const figcaption = newFigure.querySelector('figcaption');
      if (figcaption) {
        figcaption.textContent = caption;
        figcaption.dispatchEvent(new Event('input', { bubbles: true }));
      }
    }

    return {
      success: true,
      url: noteUrl,
      caption: caption
    };

  } catch (e) {
    return {
      success: false,
      error: e.message
    };
  }
}

// Fill article title and body (without images)
function fillArticleForm(title, body) {
  try {
    // Fill title
    const titleInput = document.querySelector('textarea[placeholder="記事タイトル"]');
    if (!titleInput) {
      return { success: false, error: "Title input not found. Page may not be editor." };
    }

    // Use human-like input
    titleInput.focus();
    titleInput.value = title;
    titleInput.dispatchEvent(new Event('input', { bubbles: true }));
    titleInput.dispatchEvent(new Event('change', { bubbles: true }));

    // Fill body - ProseMirror editor
    const bodyEditor = document.querySelector('.ProseMirror.note-common-styles__textnote-body');
    if (!bodyEditor) {
      return { success: false, error: "Body editor not found" };
    }

    bodyEditor.focus();

    // ProseMirror recognizes Markdown when pasted
    // Simulate paste event with Markdown content
    const clipboardData = new DataTransfer();
    clipboardData.setData('text/plain', body);

    const pasteEvent = new ClipboardEvent('paste', {
      bubbles: true,
      cancelable: true,
      clipboardData: clipboardData
    });

    bodyEditor.dispatchEvent(pasteEvent);

    // Fallback: if paste didn't work, try direct input
    if (bodyEditor.textContent.trim() === '') {
      bodyEditor.textContent = body;
      bodyEditor.dispatchEvent(new Event('input', { bubbles: true }));
    }

    return { success: true };
  } catch (e) {
    return { success: false, error: e.message };
  }
}

// Fill article with images support
async function fillArticleFormWithImages(title, body, images, headerImage) {
  try {
    // Fill title first
    const titleInput = document.querySelector('textarea[placeholder="記事タイトル"]');
    if (!titleInput) {
      return { success: false, error: "Title input not found. Page may not be editor." };
    }

    titleInput.focus();
    titleInput.value = title;
    titleInput.dispatchEvent(new Event('input', { bubbles: true }));
    titleInput.dispatchEvent(new Event('change', { bubbles: true }));

    // Upload header image if provided
    let headerImageUrl = null;
    if (headerImage) {
      const headerResult = await uploadHeaderImage(headerImage, headerImage.filename);
      if (!headerResult.success) {
        return { success: false, error: `Header image upload failed: ${headerResult.error}` };
      }
      headerImageUrl = headerResult.url;
    }

    // Upload content images and collect URLs
    const uploadedImages = [];

    if (images && images.length > 0) {
      for (const img of images) {
        const result = await uploadImage(img, img.filename, img.caption);
        if (!result.success) {
          return { success: false, error: `Image upload failed: ${result.error}` };
        }
        uploadedImages.push({
          local_path: img.local_path,
          note_url: result.url,
          caption: img.caption
        });
      }
    }

    // Replace image paths in Markdown body
    let modifiedBody = body;
    for (const img of uploadedImages) {
      // Match ![caption](local_path) patterns
      const pattern = new RegExp(`!\\[([^\\]]*)\\]\\(${img.local_path.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}\\)`, 'g');
      modifiedBody = modifiedBody.replace(pattern, `![$1](${img.note_url})`);
    }

    // Fill body with modified Markdown
    const bodyEditor = document.querySelector('.ProseMirror.note-common-styles__textnote-body');
    if (!bodyEditor) {
      return { success: false, error: "Body editor not found" };
    }

    bodyEditor.focus();

    const clipboardData = new DataTransfer();
    clipboardData.setData('text/plain', modifiedBody);

    const pasteEvent = new ClipboardEvent('paste', {
      bubbles: true,
      cancelable: true,
      clipboardData: clipboardData
    });

    bodyEditor.dispatchEvent(pasteEvent);

    // Fallback
    if (bodyEditor.textContent.trim() === '') {
      bodyEditor.textContent = modifiedBody;
      bodyEditor.dispatchEvent(new Event('input', { bubbles: true }));
    }

    return {
      success: true,
      uploaded_images: uploadedImages,
      header_image_url: headerImageUrl
    };
  } catch (e) {
    return { success: false, error: e.message };
  }
}

// Click draft save button
function clickDraftSaveButton() {
  try {
    // Find "下書き保存" button
    const buttons = document.querySelectorAll('button');
    let draftBtn = null;

    for (const btn of buttons) {
      if (btn.textContent?.includes('下書き保存')) {
        draftBtn = btn;
        break;
      }
    }

    if (!draftBtn) {
      return { success: false, error: "Draft save button not found" };
    }

    draftBtn.click();
    return { success: true };
  } catch (e) {
    return { success: false, error: e.message };
  }
}

// Click "公開に進む" button
function clickPublishProceedButton() {
  try {
    const buttons = document.querySelectorAll('button');
    let publishBtn = null;

    for (const btn of buttons) {
      if (btn.textContent?.includes('公開に進む')) {
        publishBtn = btn;
        break;
      }
    }

    if (!publishBtn) {
      return { success: false, error: "Publish proceed button not found" };
    }

    publishBtn.click();
    return { success: true };
  } catch (e) {
    return { success: false, error: e.message };
  }
}

// Click "公開に進む" or "更新" button (for update flow)
function clickPublishOrUpdateButton() {
  try {
    const buttons = document.querySelectorAll('button');
    let targetBtn = null;

    for (const btn of buttons) {
      const text = btn.textContent || '';
      if (text.includes('公開に進む') || text.includes('更新') || text.includes('公開する')) {
        targetBtn = btn;
        break;
      }
    }

    if (!targetBtn) {
      return { success: false, error: "Publish/Update button not found" };
    }

    targetBtn.click();
    return { success: true };
  } catch (e) {
    return { success: false, error: e.message };
  }
}

// Handle publish page at /publish/ URL - fill tags, add to magazines, and click final publish
// Note: "公開に進む" button navigates to /publish/ page, not a dialog!
function handlePublishPage(tags, magazines) {
  try {
    // Verify we're on the publish page
    if (!window.location.href.includes('/publish')) {
      return { success: false, error: "Not on publish page. URL: " + window.location.href };
    }

    // If we have tags to add, fill the hashtag input
    if (tags && tags.length > 0) {
      const tagInput = document.querySelector('input[placeholder="ハッシュタグを追加する"]');
      if (tagInput) {
        // Add tags one by one with Enter key
        tags.forEach((tag) => {
          tagInput.focus();
          tagInput.value = tag;
          tagInput.dispatchEvent(new Event('input', { bubbles: true }));
          // Press Enter to confirm the tag
          tagInput.dispatchEvent(new KeyboardEvent('keydown', {
            key: 'Enter',
            code: 'Enter',
            keyCode: 13,
            which: 13,
            bubbles: true
          }));
          tagInput.dispatchEvent(new KeyboardEvent('keyup', {
            key: 'Enter',
            code: 'Enter',
            keyCode: 13,
            which: 13,
            bubbles: true
          }));
        });
      }
    }

    // If we have magazines to add to, find and click their "追加" buttons
    if (magazines && magazines.length > 0) {
      // Find magazine items in the list
      // Each magazine row contains the magazine name and an "追加" button
      const magazineSection = document.querySelector('section') || document.body;
      const allRows = magazineSection.querySelectorAll('div, li');

      magazines.forEach((magazineName) => {
        // Find the row containing this magazine name
        for (const row of allRows) {
          const text = row.textContent || '';
          if (text.includes(magazineName)) {
            // Find the "追加" button in this row
            const addBtn = row.querySelector('button');
            if (addBtn && addBtn.textContent?.trim() === '追加') {
              addBtn.click();
              break;
            }
          }
        }
      });
    }

    // Find and click the "投稿する" button
    const buttons = document.querySelectorAll('button');
    for (const btn of buttons) {
      const text = btn.textContent?.trim() || '';
      if (text === '投稿する') {
        btn.click();
        return { success: true, message: "Clicked publish button" };
      }
    }

    return { success: false, error: "投稿する button not found on publish page" };
  } catch (e) {
    return { success: false, error: e.message };
  }
}

/**
 * Article List Functions - injected into note.com/notes
 */

// Find article by key and click its more menu
function findArticleAndClickMore(key) {
  try {
    const links = document.querySelectorAll('a[href*="/n/"]');
    let targetRow = null;

    for (const link of links) {
      if (link.href.includes(`/n/${key}`)) {
        targetRow = link.closest('div[class*="item"], div[class*="row"], li, tr');
        if (!targetRow) {
          // Try going up more levels
          targetRow = link.parentElement?.parentElement?.parentElement?.parentElement;
        }
        break;
      }
    }

    if (!targetRow) {
      return { success: false, error: `Article with key "${key}" not found in list` };
    }

    const moreBtn = targetRow.querySelector('[aria-label="その他"]');
    if (!moreBtn) {
      return { success: false, error: "More button not found for article" };
    }

    // Human-like click
    const rect = moreBtn.getBoundingClientRect();
    const x = rect.left + rect.width / 2;
    const y = rect.top + rect.height / 2;

    moreBtn.dispatchEvent(new MouseEvent('click', { bubbles: true, clientX: x, clientY: y }));

    return { success: true };
  } catch (e) {
    return { success: false, error: e.message };
  }
}

// Click "編集" button in the menu popup
function clickEditInMenu() {
  try {
    // Wait a bit for menu to render (should already be done by caller)
    const editBtns = document.querySelectorAll('button.m-basicBalloonList__button, [role="menuitem"], button');

    for (const btn of editBtns) {
      const text = btn.textContent?.trim() || '';
      if (text === '編集' || text.includes('編集')) {
        btn.click();
        return { success: true };
      }
    }

    return { success: false, error: "Edit button not found in menu" };
  } catch (e) {
    return { success: false, error: e.message };
  }
}

// Click "削除" button in the menu popup
function clickDeleteInMenu() {
  try {
    const deleteBtns = document.querySelectorAll('button.m-basicBalloonList__button, [role="menuitem"], button');

    for (const btn of deleteBtns) {
      const text = btn.textContent?.trim() || '';
      if (text === '削除' || text.includes('削除')) {
        btn.click();
        return { success: true };
      }
    }

    return { success: false, error: "Delete button not found in menu" };
  } catch (e) {
    return { success: false, error: e.message };
  }
}

// Confirm delete in the confirmation dialog
function confirmDeleteDialog() {
  try {
    // Look for confirmation dialog
    const modal = document.querySelector('.ReactModal__Content, [role="dialog"], [role="alertdialog"]');

    if (!modal) {
      // Maybe no modal, look for any confirm button
      const confirmBtns = document.querySelectorAll('button');
      for (const btn of confirmBtns) {
        const text = btn.textContent?.trim() || '';
        if (text === '削除' || text === '削除する' || text === 'OK' || text === '確認') {
          btn.click();
          return { success: true };
        }
      }
      return { success: false, error: "Confirmation dialog not found" };
    }

    // Find confirm button in dialog
    const dialogBtns = modal.querySelectorAll('button');
    for (const btn of dialogBtns) {
      const text = btn.textContent?.trim() || '';
      // The delete confirm button is usually red/danger styled
      if (text === '削除' || text === '削除する' || text.includes('削除')) {
        btn.click();
        return { success: true };
      }
    }

    return { success: false, error: "Confirm button not found in dialog" };
  } catch (e) {
    return { success: false, error: e.message };
  }
}

/**
 * WebSocket Communication
 */

function connectWebSocket() {
  if (ws && ws.readyState === WebSocket.OPEN) {
    return; // Already connected
  }

  try {
    ws = new WebSocket(WEBSOCKET_URL);

    ws.onopen = () => {
      console.log("[noet] WebSocket connected to CLI");
      if (wsReconnectTimer) {
        clearInterval(wsReconnectTimer);
        wsReconnectTimer = null;
      }
    };

    ws.onmessage = async (event) => {
      try {
        const request = JSON.parse(event.data);
        console.log("[noet] WebSocket received:", request);
        await handleWebSocketMessage(request);
      } catch (e) {
        console.error("[noet] WebSocket message error:", e);
      }
    };

    ws.onclose = () => {
      console.log("[noet] WebSocket disconnected");
      ws = null;
      // Attempt reconnection every 5 seconds
      if (!wsReconnectTimer) {
        wsReconnectTimer = setInterval(connectWebSocket, 5000);
      }
    };

    ws.onerror = (error) => {
      console.log("[noet] WebSocket error (CLI may not be running)");
      ws = null;
    };

  } catch (e) {
    console.log("[noet] WebSocket connection failed (CLI may not be running)");
  }
}

function sendWebSocketResponse(response) {
  if (ws && ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify(response));
  }
}

async function handleWebSocketMessage(request) {
  const { id, command, params = {} } = request;

  try {
    let result;

    switch (command) {
      case "ping":
        result = await handlePing();
        break;

      case "check_auth":
        result = await handleCheckAuth();
        break;

      case "list_articles":
        result = await handleListArticles(params);
        break;

      case "get_article":
        result = await handleGetArticle(params);
        break;

      case "create_article":
        result = await handleCreateArticle(params);
        break;

      case "update_article":
        result = await handleUpdateArticle(params);
        break;

      case "delete_article":
        result = await handleDeleteArticle(params);
        break;

      case "set_debug_mode":
        debugMode = params.enabled;
        result = { success: true, debug_mode: debugMode };
        break;

      case "get_debug_mode":
        result = { debug_mode: debugMode };
        break;

      default:
        throw new Error(`Unknown command: ${command}`);
    }

    sendWebSocketResponse({
      id,
      status: "success",
      data: result
    });

  } catch (e) {
    sendWebSocketResponse({
      id,
      status: "error",
      error: {
        code: e.code || "UNKNOWN",
        message: e.message
      }
    });
  }
}

// Initialize
console.log("[noet] Extension loaded, version:", VERSION);

// Try to connect via Native Messaging (if host is installed)
connectToNativeHost();

// Also try WebSocket connection (for direct CLI communication)
connectWebSocket();
