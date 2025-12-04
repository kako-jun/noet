// noet Browser Extension - Background Service Worker
//
// Communicates with CLI via Native Messaging
// Executes Note.com operations using DOM scraping (no API)

const VERSION = "0.1.3";
const NATIVE_HOST_NAME = "com.noet.host";

// Debug mode: when true, opens tabs visibly for DOM operations
let debugMode = false;

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
  const { title, body, tags = [] } = params;

  return await executeInTab("https://note.com/notes/new", async (tabId) => {
    await waitForTabLoad(tabId);
    await humanPageLoadWait();
    await randomDelay(2000, 4000); // Editor needs more time to initialize

    const result = await chrome.scripting.executeScript({
      target: { tabId },
      func: fillAndPublishArticle,
      args: [title, body, tags]
    });

    return result[0].result;
  });
}

async function handleUpdateArticle(params) {
  const { key, title, body, tags } = params;

  // First go to /notes, find the article, click edit
  return await executeInTab("https://note.com/notes", async (tabId) => {
    await waitForTabLoad(tabId);
    await humanPageLoadWait();
    await randomDelay(500, 1500); // Wait for article list to render

    const result = await chrome.scripting.executeScript({
      target: { tabId },
      func: findAndEditArticle,
      args: [key, title, body, tags]
    });

    return result[0].result;
  });
}

async function handleDeleteArticle(params) {
  const { key } = params;

  return await executeInTab("https://note.com/notes", async (tabId) => {
    await waitForTabLoad(tabId);
    await humanPageLoadWait();
    await randomDelay(500, 1500); // Wait for article list to render

    const result = await chrome.scripting.executeScript({
      target: { tabId },
      func: findAndDeleteArticle,
      args: [key]
    });

    return result[0].result;
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

// Fill editor and publish article (for create)
// Uses human-like input simulation to avoid detection
function fillAndPublishArticle(title, body, tags) {
  // This runs on editor.note.com/new
  // Selectors from DOM investigation:
  // - Title: textarea[placeholder="記事タイトル"]
  // - Body: .ProseMirror.note-common-styles__textnote-body (contenteditable)
  // - Publish: button:has-text("公開に進む")

  // Helper: simulate human-like focus and input
  function humanFocus(element) {
    element.focus();
    element.dispatchEvent(new FocusEvent('focus', { bubbles: true }));
  }

  function humanBlur(element) {
    element.blur();
    element.dispatchEvent(new FocusEvent('blur', { bubbles: true }));
  }

  function humanInput(element, value) {
    // Simulate keyboard events for more natural input
    humanFocus(element);

    if (element.tagName === 'TEXTAREA' || element.tagName === 'INPUT') {
      element.value = value;
    } else if (element.isContentEditable) {
      element.innerHTML = value;
    }

    // Fire events in natural order
    element.dispatchEvent(new Event('input', { bubbles: true, composed: true }));
    element.dispatchEvent(new Event('change', { bubbles: true }));

    humanBlur(element);
  }

  try {
    // Fill title with human-like interaction
    const titleInput = document.querySelector('textarea[placeholder="記事タイトル"]');
    if (!titleInput) {
      return { success: false, error: "Title input not found" };
    }
    humanInput(titleInput, title);

    // Fill body - ProseMirror editor
    const bodyEditor = document.querySelector('.ProseMirror.note-common-styles__textnote-body');
    if (!bodyEditor) {
      return { success: false, error: "Body editor not found" };
    }
    humanInput(bodyEditor, body);

    // TODO: Handle tags - needs publish dialog investigation
    // TODO: Click publish button and handle publish flow

    return {
      success: true,
      message: "Content filled. Manual publish required (publish flow not yet implemented)"
    };
  } catch (e) {
    return { success: false, error: e.message };
  }
}

// Find article by key and edit it
// Uses human-like click simulation
function findAndEditArticle(key, title, body, tags) {
  // Helper: simulate human-like click with mouse events
  function humanClick(element) {
    const rect = element.getBoundingClientRect();
    const x = rect.left + rect.width / 2;
    const y = rect.top + rect.height / 2;

    // Simulate mouse movement and click sequence
    element.dispatchEvent(new MouseEvent('mouseenter', { bubbles: true, clientX: x, clientY: y }));
    element.dispatchEvent(new MouseEvent('mouseover', { bubbles: true, clientX: x, clientY: y }));
    element.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, clientX: x, clientY: y }));
    element.dispatchEvent(new MouseEvent('mouseup', { bubbles: true, clientX: x, clientY: y }));
    element.dispatchEvent(new MouseEvent('click', { bubbles: true, clientX: x, clientY: y }));
  }

  // Find the article row with matching key
  const links = document.querySelectorAll('a[href*="/n/"]');
  let targetRow = null;

  for (const link of links) {
    if (link.href.includes(`/n/${key}`)) {
      targetRow = link.closest('div[class*="item"], div[class*="row"], li');
      break;
    }
  }

  if (!targetRow) {
    return { success: false, error: `Article with key ${key} not found` };
  }

  // Click the more button with human-like interaction
  const moreBtn = targetRow.querySelector('[aria-label="その他"]');
  if (!moreBtn) {
    return { success: false, error: "More button not found" };
  }

  humanClick(moreBtn);

  // TODO: Wait for menu, click edit, handle editor page
  return {
    success: true,
    message: "Found article. Edit flow not yet fully implemented"
  };
}

// Find article by key and delete it
// Uses human-like click simulation
function findAndDeleteArticle(key) {
  // Helper: simulate human-like click with mouse events
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

  const links = document.querySelectorAll('a[href*="/n/"]');
  let targetRow = null;

  for (const link of links) {
    if (link.href.includes(`/n/${key}`)) {
      targetRow = link.closest('div[class*="item"], div[class*="row"], li');
      break;
    }
  }

  if (!targetRow) {
    return { success: false, error: `Article with key ${key} not found` };
  }

  const moreBtn = targetRow.querySelector('[aria-label="その他"]');
  if (!moreBtn) {
    return { success: false, error: "More button not found" };
  }

  humanClick(moreBtn);

  // TODO: Wait for menu, click delete, confirm
  return {
    success: true,
    message: "Found article. Delete flow not yet fully implemented"
  };
}

// Initialize
console.log("[noet] Extension loaded, version:", VERSION);
connectToNativeHost();
