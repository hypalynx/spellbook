#!/bin/bash
set -e

OS="$(uname -s)"
ARCH="$(uname -m)"

build_linux() {
    echo "Building spellbook for Linux (amd64)..."
    cargo build --release --target x86_64-unknown-linux-gnu
    cp target/x86_64-unknown-linux-gnu/release/spellbook spellbook-linux-amd64
    chmod +x spellbook-linux-amd64
    echo "Created spellbook-linux-amd64"
}

build_macos() {
    echo "Building spellbook for macOS (arm64)..."
    cargo build --release --target aarch64-apple-darwin
    cp target/aarch64-apple-darwin/release/spellbook spellbook-macos-arm64
    chmod +x spellbook-macos-arm64
    echo "Created spellbook-macos-arm64"
}

case "$OS" in
    Linux)
        build_linux
        ;;
    Darwin)
        build_macos
        ;;
    *)
        echo "Unknown OS: $OS"
        exit 1
        ;;
esac

echo "Done! Upload the binary to your GitHub release."
