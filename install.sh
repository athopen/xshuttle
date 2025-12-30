#!/bin/bash
set -euo pipefail

main() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS-$ARCH" in
        Linux-x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
        Darwin-x86_64) TARGET="x86_64-apple-darwin" ;;
        Darwin-arm64)  TARGET="aarch64-apple-darwin" ;;
        *) echo "Unsupported platform: $OS-$ARCH"; exit 1 ;;
    esac

    VERSION=$(curl -fsSL https://api.github.com/repos/athopen/xshuttle/releases/latest | grep '"tag_name"' | cut -d'"' -f4)

    echo "Installing xshuttle $VERSION for $TARGET..."
    curl -fsSL "https://github.com/athopen/xshuttle/releases/download/$VERSION/xshuttle-$TARGET.tar.gz" \
        | sudo tar xz -C /usr/local/bin

    echo "xshuttle installed successfully!"
}

main "$@"
