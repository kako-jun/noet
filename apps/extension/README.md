# noet Browser Extension

noet CLI companion extension for Note.com.

## Overview

This extension acts as a bridge between the noet CLI and Note.com. It uses the browser's logged-in session to perform operations that require authentication.

## Architecture

```
CLI (Rust) <---> Native Messaging Host <---> Extension <---> Note.com
```

The extension:
1. Receives commands from CLI via Native Messaging
2. Executes operations using the browser's session
3. Returns results (raw HTML) back to CLI

## Installation (Development)

1. Open `chrome://extensions`
2. Enable "Developer mode"
3. Click "Load unpacked"
4. Select this directory (`packages/extension`)

## Files

- `manifest.json` - Extension manifest (Manifest V3)
- `src/background.js` - Service Worker for command handling
- `src/content.js` - Content script for DOM operations
- `icons/` - Extension icons

## Commands

| Command | Description |
|---------|-------------|
| `ping` | Check extension availability and version |
| `check_auth` | Check Note.com login status |
| `list_articles` | Get user's articles (including drafts) |
| `get_article` | Get article content as HTML |
| `create_article` | Create new article via DOM |
| `update_article` | Update existing article via DOM |
| `delete_article` | Delete article |
| `set_debug_mode` | Toggle visible tab operations |
| `get_debug_mode` | Get current debug mode status |

## Debug Mode

When debug mode is ON (`noet debug on`), DOM operations will open visible tabs so you can see what the extension is doing. Useful for debugging.

When OFF (default), operations happen in background tabs that are immediately closed.

## Development

No build step required - plain JavaScript.

To test:
1. Load the extension in Chrome
2. Log in to note.com
3. Use noet CLI commands
