#!/bin/bash
set -e

OS="$(uname -s)"
ARCH="$(uname -m)"

build_linux() {
    echo "Building lmx for Linux (amd64)..."
    cargo build --release --target x86_64-unknown-linux-gnu
    cp target/x86_64-unknown-linux-gnu/release/lmx lmx-linux-amd64
    chmod +x lmx-linux-amd64
    echo "Created lmx-linux-amd64"
}

build_macos() {
    echo "Building lmx for macOS (arm64)..."
    cargo build --release --target aarch64-apple-darwin
    cp target/aarch64-apple-darwin/release/lmx lmx-macos-arm64
    chmod +x lmx-macos-arm64
    echo "Created lmx-macos-arm64"
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
