#!/bin/sh
set -e

REPO="tomatitito/ralph"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

main() {
    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    arch=$(uname -m)

    case "$os" in
        linux)
            case "$arch" in
                x86_64|amd64)
                    artifact="ralph-loop-linux-x86_64"
                    ;;
                *)
                    echo "Error: Unsupported Linux architecture: $arch" >&2
                    exit 1
                    ;;
            esac
            ;;
        darwin)
            case "$arch" in
                arm64|aarch64)
                    artifact="ralph-loop-macos-arm64"
                    ;;
                *)
                    echo "Error: Unsupported macOS architecture: $arch (only Apple Silicon supported)" >&2
                    exit 1
                    ;;
            esac
            ;;
        *)
            echo "Error: Unsupported OS: $os" >&2
            exit 1
            ;;
    esac

    echo "Detected: $os/$arch"
    echo "Fetching latest release..."

    latest_url=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" |
        grep "browser_download_url.*${artifact}.tar.gz" |
        cut -d '"' -f 4)

    if [ -z "$latest_url" ]; then
        echo "Error: Could not find release artifact for $artifact" >&2
        exit 1
    fi

    echo "Downloading $artifact..."

    tmpdir=$(mktemp -d)
    trap 'rm -rf "$tmpdir"' EXIT

    curl -fsSL "$latest_url" | tar -xz -C "$tmpdir"

    mkdir -p "$INSTALL_DIR"
    mv "$tmpdir/ralph-loop" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/ralph-loop"

    echo "Installed ralph-loop to $INSTALL_DIR/ralph-loop"

    if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
        echo ""
        echo "Add $INSTALL_DIR to your PATH:"
        echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
    fi
}

main
