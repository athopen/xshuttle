#!/bin/bash
set -euo pipefail

# --- Configuration ---
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
}

# --- Uninstallation ---
uninstall_linux() {
    # Remove binary
    if [[ -f "/usr/local/bin/$APP_NAME" ]]; then
        sudo rm -f "/usr/local/bin/$APP_NAME"
        success "Removed /usr/local/bin/$APP_NAME"
    fi

    # Remove desktop entry
    if [[ -f "/usr/share/applications/xshuttle.desktop" ]]; then
        sudo rm -f /usr/share/applications/xshuttle.desktop
        success "Removed desktop entry"
    fi

    # Remove icons (hicolor theme)
    local sizes="16 32 48 64 128 256 512 1024"
    for size in $sizes; do
        sudo rm -f "/usr/share/icons/hicolor/${size}x${size}/apps/xshuttle.png" 2>/dev/null || true
    done
    sudo rm -f /usr/share/icons/hicolor/scalable/apps/xshuttle.svg 2>/dev/null || true
    success "Removed icons"

    # Remove autostart
    if [[ -f "$HOME/.config/autostart/xshuttle.desktop" ]]; then
        rm -f "$HOME/.config/autostart/xshuttle.desktop"
        success "Removed autostart entry"
    fi

    sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
    sudo update-desktop-database /usr/share/applications 2>/dev/null || true
}

uninstall_macos() {
    # Remove binary
    if [[ -f "/usr/local/bin/$APP_NAME" ]]; then
        sudo rm -f "/usr/local/bin/$APP_NAME"
        success "Removed /usr/local/bin/$APP_NAME"
    fi

    # Unload and remove LaunchAgent
    local plist_file="$HOME/Library/LaunchAgents/$PLIST_LABEL.plist"
    if [[ -f "$plist_file" ]]; then
        launchctl bootout "gui/$(id -u)" "$plist_file" 2>/dev/null || true
        rm -f "$plist_file"
        success "Removed LaunchAgent"
    fi
}

# --- Main ---
main() {
    detect_platform

    info "Uninstalling xshuttle..."
    echo ""

    if [[ "$OS" == "Linux" ]]; then
        uninstall_linux
    elif [[ "$OS" == "Darwin" ]]; then
        uninstall_macos
    else
        error "Unsupported platform: $OS"
        exit 1
    fi

    echo ""
    success "xshuttle has been uninstalled"
}

main "$@"
