#!/bin/bash
set -e

# Dashy Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/mshaaban0/dashy/main/install.sh | bash

REPO="mshaaban0/dashy"
BINARY_NAME="dashy"
INSTALL_DIR="/usr/local/bin"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() {
    echo -e "${GREEN}==>${NC} $1"
}

warn() {
    echo -e "${YELLOW}Warning:${NC} $1"
}

error() {
    echo -e "${RED}Error:${NC} $1"
    exit 1
}

# Detect OS and architecture
detect_platform() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)

    case "$OS" in
        darwin)
            OS="apple-darwin"
            ;;
        linux)
            OS="unknown-linux-gnu"
            ;;
        *)
            error "Unsupported operating system: $OS"
            ;;
    esac

    case "$ARCH" in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        arm64|aarch64)
            ARCH="aarch64"
            ;;
        *)
            error "Unsupported architecture: $ARCH"
            ;;
    esac

    PLATFORM="${ARCH}-${OS}"
    info "Detected platform: $PLATFORM"
}

# Get latest release version
get_latest_version() {
    info "Fetching latest version..."
    VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"v?([^"]+)".*/\1/')

    if [ -z "$VERSION" ]; then
        error "Could not determine latest version. Please check your internet connection."
    fi

    info "Latest version: v$VERSION"
}

# Download and install
install() {
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/v${VERSION}/${BINARY_NAME}-${PLATFORM}.tar.gz"
    TEMP_DIR=$(mktemp -d)

    info "Downloading from: $DOWNLOAD_URL"

    if ! curl -fsSL "$DOWNLOAD_URL" -o "${TEMP_DIR}/${BINARY_NAME}.tar.gz"; then
        error "Failed to download. The release for your platform may not exist yet."
    fi

    info "Extracting..."
    tar -xzf "${TEMP_DIR}/${BINARY_NAME}.tar.gz" -C "$TEMP_DIR"

    info "Installing to ${INSTALL_DIR}/${BINARY_NAME}..."
    if [ -w "$INSTALL_DIR" ]; then
        mv "${TEMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    else
        sudo mv "${TEMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    fi

    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    # Cleanup
    rm -rf "$TEMP_DIR"

    info "Successfully installed ${BINARY_NAME} to ${INSTALL_DIR}/${BINARY_NAME}"
    echo ""
    echo -e "Run ${GREEN}dashy${NC} to start the system monitor."
}

main() {
    echo ""
    echo "  ____            _           "
    echo " |  _ \  __ _ ___| |__  _   _ "
    echo " | | | |/ _\` / __| '_ \| | | |"
    echo " | |_| | (_| \__ \ | | | |_| |"
    echo " |____/ \__,_|___/_| |_|\__, |"
    echo "                        |___/ "
    echo ""
    echo "Terminal System Monitor"
    echo ""

    detect_platform
    get_latest_version
    install
}

main
