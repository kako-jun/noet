// noet Browser Extension - Content Script
//
// Injected into note.com pages for DOM operations
// Receives messages from background.js

const VERSION = "0.1.2";

/**
 * Message handler from background script
 */
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  console.log("[noet content] Received message:", message.action);

  switch (message.action) {
    case "fill_editor":
      fillEditor(message.title, message.body, message.status)
        .then(sendResponse)
        .catch(e => sendResponse({ success: false, error: e.message }));
      return true; // Keep channel open for async response

    case "get_article_html":
      getArticleHtml()
        .then(sendResponse)
        .catch(e => sendResponse({ success: false, error: e.message }));
      return true;

    case "click_publish":
      clickPublishButton(message.status)
        .then(sendResponse)
        .catch(e => sendResponse({ success: false, error: e.message }));
      return true;

    default:
      sendResponse({ success: false, error: "Unknown action" });
  }
});

/**
 * Fill the note editor with title and body
 * TODO: Update selectors after DOM investigation
 */
async function fillEditor(title, body, status) {
  // Wait for editor to be ready
  await waitForElement(".editor-title, [data-testid='title-input']");

  // Selectors to be determined by Playwright investigation
  const titleInput = document.querySelector(".editor-title, [data-testid='title-input']");
  const bodyInput = document.querySelector(".editor-body, [data-testid='body-input']");

  if (!titleInput || !bodyInput) {
    throw new Error("Editor elements not found");
  }

  // Set title
  titleInput.focus();
  titleInput.value = title;
  titleInput.dispatchEvent(new Event("input", { bubbles: true }));

  // Set body
  bodyInput.focus();
  bodyInput.innerHTML = body;
  bodyInput.dispatchEvent(new Event("input", { bubbles: true }));

  return { success: true };
}

/**
 * Get article HTML from the current page
 */
async function getArticleHtml() {
  // Try to get from __NEXT_DATA__
  const nextData = document.querySelector("#__NEXT_DATA__");
  if (nextData) {
    try {
      const data = JSON.parse(nextData.textContent);
      const note = data.props?.pageProps?.note;
      if (note) {
        return {
          success: true,
          html: note.body,
          title: note.name,
          status: note.status
        };
      }
    } catch (e) {
      console.error("[noet content] Failed to parse __NEXT_DATA__:", e);
    }
  }

  // Fallback: get from DOM
  const articleBody = document.querySelector(".note-common-styles__textnote-body");
  if (articleBody) {
    return {
      success: true,
      html: articleBody.innerHTML,
      title: document.querySelector("h1")?.textContent || "",
      status: "unknown"
    };
  }

  throw new Error("Article content not found");
}

/**
 * Click publish or draft save button
 */
async function clickPublishButton(status) {
  // Selectors to be determined
  const selector = status === "published"
    ? ".publish-button, [data-testid='publish-button']"
    : ".draft-save-button, [data-testid='draft-save-button']";

  const button = document.querySelector(selector);
  if (!button) {
    throw new Error("Button not found: " + selector);
  }

  button.click();

  // Wait for navigation or success indicator
  await new Promise(resolve => setTimeout(resolve, 2000));

  return { success: true };
}

/**
 * Wait for an element to appear in the DOM
 */
function waitForElement(selector, timeout = 10000) {
  return new Promise((resolve, reject) => {
    const element = document.querySelector(selector);
    if (element) {
      resolve(element);
      return;
    }

    const observer = new MutationObserver((mutations, obs) => {
      const element = document.querySelector(selector);
      if (element) {
        obs.disconnect();
        resolve(element);
      }
    });

    observer.observe(document.body, {
      childList: true,
      subtree: true
    });

    setTimeout(() => {
      observer.disconnect();
      reject(new Error("Timeout waiting for element: " + selector));
    }, timeout);
  });
}

console.log("[noet content] Content script loaded, version:", VERSION);
