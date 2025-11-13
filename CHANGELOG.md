# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2025-01-14

### Added
- **Rate Limiting**: Fixed 500ms delay between all API requests to prevent IP bans
  - Automatic rate limiting integrated into NoteClient
  - 2 requests per second maximum
  - Thread-safe implementation with Mutex
  - 4 new tests for rate limiter functionality
- **Export Overwrite Warning**: Confirmation prompt when exporting to directory with existing files
  - Single confirmation at start, not per-file (practical for bulk export)
  - Prevents accidental data loss during re-export

### Changed
- **Complete Japanese Localization**: All UI messages, error messages, and prompts now in Japanese
  - error.rs: All error enum variants translated
  - All command modules: article, workspace, template, export, tag, engagement, magazine, user
  - Consistent with interactive mode language
- **CSRF Token Naming**: Renamed to XSRF to match Note.com API specification
- **rustls Migration**: Switched from OpenSSL to rustls for TLS (pure Rust implementation)

### Documentation
- Updated CLAUDE.md with rate limiting implementation details
- Updated README.md with new features (rate limiting, Japanese UI)
- Added comprehensive inline documentation

### Tests
- All 58 tests passing (54 existing + 4 new rate_limiter tests)

## [0.1.0] - 2025-01-13

### Added
- Initial release
- **Article Management**: Create, publish, update, delete, list articles
- **Interactive Mode**: Menu-driven UI when running `noet` without arguments
- **TUI Diff Display**: Side-by-side diff view before publishing
- **Workspace Feature**: Project management with `.noet/` directory
- **Template System**: Create and manage article templates
- **Export Functionality**: Download articles from Note.com as Markdown
- **Editor Integration**: Auto-launch configured editor for new articles
- **Tag Management**: List, search hashtags
- **Magazine Management**: Add/remove articles from magazines
- **Engagement**: Like/unlike articles, view comments
- **User Information**: Fetch user profiles and statistics
- **Secure Authentication**: Credentials stored in system keyring
  - macOS: Keychain
  - Linux: Secret Service (libsecret)
  - Windows: Credential Manager
- **Hugo-like Interface**: Frontmatter-based Markdown articles
- **Configuration Management**: Global and workspace-level config with merging
- **Keyboard Shortcuts**: Single-character shortcuts in interactive mode

### Features
- Cross-platform support (Linux, macOS, Windows)
- Proxy support (HTTP_PROXY, HTTPS_PROXY environment variables)
- 30-second timeout for API requests
- Custom error handling with thiserror
- Git-like workflow with workspaces

[0.1.1]: https://github.com/kako-jun/noet/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/kako-jun/noet/releases/tag/v0.1.0
