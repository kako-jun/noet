#!/bin/bash
# Install noet Native Messaging Host manifest
#
# Usage:
#   ./install.sh <extension_id> [noet_path]
#
# Arguments:
#   extension_id  - Chrome extension ID (required)
#   noet_path     - Path to noet binary (default: /usr/local/bin/noet)

set -e

EXTENSION_ID="${1:-}"
NOET_PATH="${2:-/usr/local/bin/noet}"

if [ -z "$EXTENSION_ID" ]; then
    echo "Usage: $0 <extension_id> [noet_path]"
    echo ""
    echo "Get your extension ID from chrome://extensions after loading the extension."
    exit 1
fi

# Determine OS and manifest location
case "$(uname -s)" in
    Linux*)
        MANIFEST_DIR="$HOME/.config/google-chrome/NativeMessagingHosts"
        ;;
    Darwin*)
        MANIFEST_DIR="$HOME/Library/Application Support/Google/Chrome/NativeMessagingHosts"
        ;;
    *)
        echo "Unsupported OS: $(uname -s)"
        exit 1
        ;;
esac

# Create directory if it doesn't exist
mkdir -p "$MANIFEST_DIR"

# Generate manifest
MANIFEST_FILE="$MANIFEST_DIR/com.noet.host.json"

cat > "$MANIFEST_FILE" << EOF
{
  "name": "com.noet.host",
  "description": "noet CLI Native Messaging Host",
  "path": "$NOET_PATH",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://$EXTENSION_ID/"
  ]
}
EOF

echo "Native Messaging Host manifest installed to:"
echo "  $MANIFEST_FILE"
echo ""
echo "Contents:"
cat "$MANIFEST_FILE"
echo ""
echo "Make sure noet is installed at: $NOET_PATH"
echo "Run 'noet --native-messaging' to test the host."
