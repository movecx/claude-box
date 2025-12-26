#!/bin/sh
set -e

REPO="movecx/claude-box"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Detect OS and architecture
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux)  OS="unknown-linux-musl" ;;
        Darwin) OS="apple-darwin" ;;
        *)      echo "Unsupported OS: $OS"; exit 1 ;;
    esac

    case "$ARCH" in
        x86_64)  ARCH="x86_64" ;;
        aarch64) ARCH="aarch64" ;;
        arm64)   ARCH="aarch64" ;;
        *)       echo "Unsupported architecture: $ARCH"; exit 1 ;;
    esac

    PLATFORM="${ARCH}-${OS}"
}

# Get latest release version
get_latest_version() {
    curl -sSf "https://api.github.com/repos/${REPO}/releases/latest" | \
        grep '"tag_name":' | \
        sed -E 's/.*"([^"]+)".*/\1/'
}

main() {
    detect_platform
    echo "Detected platform: $PLATFORM"

    VERSION="${VERSION:-$(get_latest_version)}"
    if [ -z "$VERSION" ]; then
        echo "Failed to get latest version"
        exit 1
    fi
    echo "Installing claude-box $VERSION"

    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/claude-box-${PLATFORM}.tar.gz"

    # Create install directory
    mkdir -p "$INSTALL_DIR"

    # Download and extract
    echo "Downloading from $DOWNLOAD_URL"
    curl -sSfL "$DOWNLOAD_URL" | tar xz -C "$INSTALL_DIR"
    chmod +x "$INSTALL_DIR/claude-box"

    echo "Installed claude-box to $INSTALL_DIR/claude-box"

    # Check if install dir is in PATH
    case ":$PATH:" in
        *":$INSTALL_DIR:"*) ;;
        *)
            echo ""
            echo "Add $INSTALL_DIR to your PATH:"
            echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
            echo ""
            echo "Add this to your ~/.bashrc or ~/.zshrc to make it permanent."
            ;;
    esac
}

main
