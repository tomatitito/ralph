#!/bin/sh
set -e

REPO="tomatitito/ralph"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

get_platform_suffix() {
    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    arch=$(uname -m)

    case "$os" in
        linux)
            case "$arch" in
                x86_64|amd64)
                    echo "linux-x86_64"
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
                    echo "macos-arm64"
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
}

install_binary() {
    binary_name="$1"
    platform_suffix="$2"
    artifact="${binary_name}-${platform_suffix}"

    echo "Fetching latest release for $binary_name..."

    latest_url=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" |
        grep "browser_download_url.*${artifact}.tar.gz" |
        cut -d '"' -f 4)

    if [ -z "$latest_url" ]; then
        echo "Error: Could not find release artifact for $artifact" >&2
        return 1
    fi

    echo "Downloading $artifact..."

    tmpdir=$(mktemp -d)
    trap 'rm -rf "$tmpdir"' EXIT

    curl -fsSL "$latest_url" | tar -xz -C "$tmpdir"

    mkdir -p "$INSTALL_DIR"
    mv "$tmpdir/$binary_name" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/$binary_name"

    echo "Installed $binary_name to $INSTALL_DIR/$binary_name"
}

main() {
    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    arch=$(uname -m)
    echo "Detected: $os/$arch"

    platform_suffix=$(get_platform_suffix)

    install_binary "ralph-loop" "$platform_suffix"
    install_binary "ralph-viewer" "$platform_suffix"

    if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
        echo ""
        echo "Add $INSTALL_DIR to your PATH:"
        echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
    fi
}

main
