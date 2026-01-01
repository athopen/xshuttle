#!/bin/bash
set -euo pipefail

# --- Configuration ---
REPO="athopen/xshuttle"
APP_NAME="xshuttle"
PLIST_LABEL="com.athopen.xshuttle"
CLEANUP_DIR=""

# --- Colors ---
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${BLUE}$1${NC}" >&2; }
success() { echo -e "${GREEN}✓ $1${NC}" >&2; }
error() { echo -e "${RED}error: $1${NC}" >&2; }

# --- Cleanup ---
cleanup() {
    if [[ -n "$CLEANUP_DIR" && -d "$CLEANUP_DIR" ]]; then
        rm -rf "$CLEANUP_DIR"
    fi
}
trap cleanup EXIT

# --- Platform Detection ---
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS-$ARCH" in
        Linux-x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
        Darwin-x86_64) TARGET="x86_64-apple-darwin" ;;
        Darwin-arm64)  TARGET="aarch64-apple-darwin" ;;
        *)
            error "Unsupported platform: $OS-$ARCH"
            exit 1
            ;;
    esac
}

# --- Download ---
download_release() {
    local version
    local url
    local archive

    info "Fetching latest version..."
    version=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)

    if [[ -z "$version" ]]; then
        error "Failed to fetch latest version"
        exit 1
    fi

    info "Downloading xshuttle $version for $TARGET..."

    CLEANUP_DIR=$(mktemp -d)
    archive="$CLEANUP_DIR/$APP_NAME-$TARGET.tar.gz"
    url="https://github.com/$REPO/releases/download/$version/$APP_NAME-$TARGET.tar.gz"

    # Download to file (not pipe) for better error handling
    if ! curl -fsSL "$url" -o "$archive"; then
        error "Failed to download from $url"
        exit 1
    fi

    # Verify download succeeded
    if [[ ! -s "$archive" ]]; then
        error "Downloaded file is empty or missing"
        exit 1
    fi

    # Extract
    if ! tar xzf "$archive" -C "$CLEANUP_DIR"; then
        error "Failed to extract archive"
        exit 1
    fi

    # Verify binary exists
    local extract_dir="$CLEANUP_DIR/$APP_NAME-$TARGET"
    if [[ ! -f "$extract_dir/$APP_NAME" ]]; then
        error "Binary not found in archive (expected $extract_dir/$APP_NAME)"
        exit 1
    fi

    echo "$extract_dir"
}

# --- Installation Functions ---
install_binary() {
    local srcdir="$1"
    local binary="$srcdir/$APP_NAME"

    sudo mkdir -p /usr/local/bin
    sudo cp "$binary" /usr/local/bin/
    sudo chmod +x "/usr/local/bin/$APP_NAME"

    success "Installed binary to /usr/local/bin/$APP_NAME"
}

install_desktop_linux() {
    local srcdir="$1"

    # Desktop entry
    if [[ -f "$srcdir/xshuttle.desktop" ]]; then
        sudo mkdir -p /usr/share/applications
        sudo cp "$srcdir/xshuttle.desktop" /usr/share/applications/
        success "Installed desktop entry"
    fi

    # Icons (hicolor theme)
    if [[ -d "$srcdir/icons" ]]; then
        local sizes="16 32 48 64 128 256 512 1024"
        for size in $sizes; do
            if [[ -f "$srcdir/icons/xshuttle-${size}.png" ]]; then
                sudo mkdir -p "/usr/share/icons/hicolor/${size}x${size}/apps"
                sudo cp "$srcdir/icons/xshuttle-${size}.png" "/usr/share/icons/hicolor/${size}x${size}/apps/xshuttle.png"
            fi
        done
        if [[ -f "$srcdir/icons/xshuttle.svg" ]]; then
            sudo mkdir -p /usr/share/icons/hicolor/scalable/apps
            sudo cp "$srcdir/icons/xshuttle.svg" /usr/share/icons/hicolor/scalable/apps/xshuttle.svg
        fi
        success "Installed icons"
    fi

    # Autostart
    if [[ -f "$srcdir/xshuttle-autostart.desktop" ]]; then
        mkdir -p "$HOME/.config/autostart"
        cp "$srcdir/xshuttle-autostart.desktop" "$HOME/.config/autostart/xshuttle.desktop"
        success "Enabled autostart"
    fi

    sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
    sudo update-desktop-database /usr/share/applications 2>/dev/null || true
}

install_autostart_macos() {
    local srcdir="$1"
    local plist_file="$HOME/Library/LaunchAgents/$PLIST_LABEL.plist"

    if [[ -f "$srcdir/$PLIST_LABEL.plist" ]]; then
        mkdir -p "$HOME/Library/LaunchAgents"
        cp "$srcdir/$PLIST_LABEL.plist" "$plist_file"

        launchctl bootout "gui/$(id -u)" "$plist_file" 2>/dev/null || true
        launchctl bootstrap "gui/$(id -u)" "$plist_file"

        success "Enabled autostart (LaunchAgent)"
    fi
}

# --- Main ---
main() {
    detect_platform

    echo ""
    srcdir=$(download_release)

    install_binary "$srcdir"

    if [[ "$OS" == "Linux" ]]; then
        install_desktop_linux "$srcdir"
    elif [[ "$OS" == "Darwin" ]]; then
        install_autostart_macos "$srcdir"
    fi

    echo ""
    success "xshuttle installed successfully!"
    echo ""
    info "Run 'xshuttle' to start."
}

main "$@"
