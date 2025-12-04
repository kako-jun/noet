// noet Browser Extension - Background Service Worker
//
// Communicates with CLI via Native Messaging
// Executes Note.com operations using browser session

const VERSION = "0.1.2";
const NATIVE_HOST_NAME = "com.noet.host";

// Debug mode: when true, opens tabs visibly for DOM operations
let debugMode = false;

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
 * Command handlers
 */

async function handlePing() {
  return {
    version: VERSION,
    extension_id: chrome.runtime.id
  };
}

async function handleCheckAuth() {
  try {
    const response = await fetch("https://note.com/api/v1/users/me", {
      credentials: "include"
    });

    if (response.ok) {
      const data = await response.json();
      return {
        logged_in: true,
        username: data.data?.urlname || null
      };
    }
  } catch (e) {
    // Ignore fetch errors
  }

  return {
    logged_in: false,
    username: null
  };
}

async function handleListArticles(params) {
  const { username, page = 1 } = params;

  const url = `https://note.com/api/v2/creators/${username}/contents?kind=note&page=${page}`;
  const response = await fetch(url, { credentials: "include" });

  if (!response.ok) {
    const error = new Error("Failed to fetch articles");
    error.code = "NOT_FOUND";
    throw error;
  }

  const data = await response.json();
  const articles = (data.data?.contents || []).map(note => ({
    key: note.key,
    title: note.name,
    updated_at: note.updated_at
  }));

  return {
    articles,
    has_next: data.data?.isLastPage === false
  };
}

async function handleGetArticle(params) {
  const { key } = params;

  const response = await fetch(`https://note.com/api/v3/notes/${key}`, {
    credentials: "include"
  });

  if (!response.ok) {
    const error = new Error("Article not found");
    error.code = "NOT_FOUND";
    throw error;
  }

  const data = await response.json();
  const note = data.data;

  // Extract hashtags
  const tags = (note.hashtag_notes || []).map(h => h.hashtag?.name).filter(Boolean);

  return {
    html: note.body || "",
    title: note.name,
    tags,
    created_at: note.created_at,
    updated_at: note.updated_at
  };
}

async function handleCreateArticle(params) {
  const { title, body, tags = [] } = params;

  // DOM operation required - open editor page
  return await executeInTab("https://note.com/notes/new", async (tabId) => {
    // Wait for page to load
    await waitForTabLoad(tabId);

    // Execute editor operations
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

  // DOM operation required - open editor page
  const editUrl = `https://note.com/notes/${key}/edit`;
  return await executeInTab(editUrl, async (tabId) => {
    await waitForTabLoad(tabId);

    const result = await chrome.scripting.executeScript({
      target: { tabId },
      func: updateAndPublishArticle,
      args: [title, body, tags]
    });

    return result[0].result;
  });
}

async function handleDeleteArticle(params) {
  const { key } = params;

  // DOM operation required
  const deleteUrl = `https://note.com/notes/${key}/edit`;
  return await executeInTab(deleteUrl, async (tabId) => {
    await waitForTabLoad(tabId);

    const result = await chrome.scripting.executeScript({
      target: { tabId },
      func: deleteArticleFromPage,
      args: []
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
        // Additional wait for JS to initialize
        setTimeout(resolve, 1000);
      }
    });
  });
}

/**
 * Functions to be injected into page
 * TODO: Update selectors after Playwright DOM investigation
 */

function fillAndPublishArticle(title, body, tags) {
  // Placeholder - selectors need to be determined
  return {
    success: false,
    error: "DOM selectors not yet implemented - needs Playwright investigation"
  };
}

function updateAndPublishArticle(title, body, tags) {
  return {
    success: false,
    error: "DOM selectors not yet implemented - needs Playwright investigation"
  };
}

function deleteArticleFromPage() {
  return {
    success: false,
    error: "DOM selectors not yet implemented - needs Playwright investigation"
  };
}

// Initialize
console.log("[noet] Extension loaded, version:", VERSION);
connectToNativeHost();
