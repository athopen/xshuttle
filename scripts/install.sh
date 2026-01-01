#!/bin/bash
set -euo pipefail

# --- Configuration ---
REPO="athopen/xshuttle"
APP_NAME="xshuttle"
PLIST_LABEL="com.athopen.xshuttle"

# --- Colors ---
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${BLUE}$1${NC}"; }
success() { echo -e "${GREEN}✓ $1${NC}"; }
error() { echo -e "${RED}✗ $1${NC}" >&2; }

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
    local tmpdir

    info "Fetching latest version..."
    version=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)

    if [[ -z "$version" ]]; then
        error "Failed to fetch latest version"
        exit 1
    fi

    info "Installing xshuttle $version for $TARGET..."

    url="https://github.com/$REPO/releases/download/$version/$APP_NAME-$TARGET.tar.gz"
    tmpdir=$(mktemp -d)
    trap 'rm -rf "$tmpdir"' EXIT

    curl -fsSL "$url" | tar xz -C "$tmpdir"

    echo "$tmpdir"
}

# --- Installation Functions ---
install_binary() {
    local tmpdir="$1"

    sudo mkdir -p /usr/local/bin
    sudo cp "$tmpdir/$APP_NAME" /usr/local/bin/
    sudo chmod +x "/usr/local/bin/$APP_NAME"

    success "Installed binary to /usr/local/bin/$APP_NAME"
}

install_desktop_linux() {
    local tmpdir="$1"

    # Desktop entry
    if [[ -f "$tmpdir/xshuttle.desktop" ]]; then
        sudo mkdir -p /usr/share/applications
        sudo cp "$tmpdir/xshuttle.desktop" /usr/share/applications/
        success "Installed desktop entry"
    fi

    # Icons (hicolor theme)
    if [[ -d "$tmpdir/icons" ]]; then
        local sizes="16 32 48 64 128 256 512 1024"
        for size in $sizes; do
            if [[ -f "$tmpdir/icons/xshuttle-${size}.png" ]]; then
                sudo mkdir -p "/usr/share/icons/hicolor/${size}x${size}/apps"
                sudo cp "$tmpdir/icons/xshuttle-${size}.png" "/usr/share/icons/hicolor/${size}x${size}/apps/xshuttle.png"
            fi
        done
        if [[ -f "$tmpdir/icons/xshuttle.svg" ]]; then
            sudo mkdir -p /usr/share/icons/hicolor/scalable/apps
            sudo cp "$tmpdir/icons/xshuttle.svg" /usr/share/icons/hicolor/scalable/apps/xshuttle.svg
        fi
        success "Installed icons"
    fi

    # Autostart
    if [[ -f "$tmpdir/xshuttle-autostart.desktop" ]]; then
        mkdir -p "$HOME/.config/autostart"
        cp "$tmpdir/xshuttle-autostart.desktop" "$HOME/.config/autostart/xshuttle.desktop"
        success "Enabled autostart"
    fi

    sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
    sudo update-desktop-database /usr/share/applications 2>/dev/null || true
}

install_autostart_macos() {
    local tmpdir="$1"
    local plist_file="$HOME/Library/LaunchAgents/$PLIST_LABEL.plist"

    if [[ -f "$tmpdir/$PLIST_LABEL.plist" ]]; then
        mkdir -p "$HOME/Library/LaunchAgents"
        cp "$tmpdir/$PLIST_LABEL.plist" "$plist_file"

        launchctl bootout "gui/$(id -u)" "$plist_file" 2>/dev/null || true
        launchctl bootstrap "gui/$(id -u)" "$plist_file"

        success "Enabled autostart (LaunchAgent)"
    fi
}

# --- Main ---
main() {
    detect_platform

    echo ""
    tmpdir=$(download_release)

    install_binary "$tmpdir"

    if [[ "$OS" == "Linux" ]]; then
        install_desktop_linux "$tmpdir"
    elif [[ "$OS" == "Darwin" ]]; then
        install_autostart_macos "$tmpdir"
    fi

    echo ""
    success "xshuttle installed successfully!"
    echo ""
    info "Run 'xshuttle' to start."
}

main "$@"
