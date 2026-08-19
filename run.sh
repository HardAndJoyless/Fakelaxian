#!/bin/bash

set -e

# Run from this script's own directory so relative paths (Cargo.toml, target/)
# work regardless of where the script is invoked from.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Fakelaxian Terminal Clone - Setup${NC}"
echo "================================"

# Check terminal size (fall back gracefully when tput is unavailable)
COLS="$(tput cols 2>/dev/null || echo 80)"
LINES="$(tput lines 2>/dev/null || echo 24)"

if [ "$COLS" -lt 40 ] || [ "$LINES" -lt 15 ]; then
    echo -e "${YELLOW}Warning: Terminal should be at least 40x15 for best experience${NC}"
    echo "Current size: ${COLS}x${LINES}"
    echo
fi

# Check for cargo
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Rust cargo not found${NC}"
    echo
    echo "To install Rust, run:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo "  source \$HOME/.cargo/env"
    echo
    echo "Or install via your package manager:"
    echo "  Ubuntu/Debian: sudo apt install rustc cargo"
    echo "  Fedora: sudo dnf install rust cargo"
    echo "  Arch: sudo pacman -S rust"
    exit 1
fi

# Check Rust version meets the minimum requirement
RUST_VERSION="$(rustc --version | grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+' | head -1 || true)"
echo "Found Rust version: ${RUST_VERSION:-unknown}"

MIN_MAJOR=1
MIN_MINOR=70
if [ -n "$RUST_VERSION" ]; then
    MAJOR="$(echo "$RUST_VERSION" | cut -d. -f1)"
    MINOR="$(echo "$RUST_VERSION" | cut -d. -f2)"
    if [ "$MAJOR" -lt "$MIN_MAJOR" ] || { [ "$MAJOR" -eq "$MIN_MAJOR" ] && [ "$MINOR" -lt "$MIN_MINOR" ]; }; then
        echo -e "${YELLOW}Warning: Rust 1.70 or higher is recommended (found ${RUST_VERSION})${NC}"
    fi
fi

# Build in release mode
echo
echo -e "${GREEN}Building Fakelaxian...${NC}"
cargo build --release

echo -e "${GREEN}Build successful!${NC}"
echo
echo "================================"
echo "Controls:"
echo "  Arrows / A-D: Move"
echo "  S / Down: Brake"
echo "  Space: Shoot"
echo "  C: Cycle theme"
echo "  P: Pause"
echo "  R: Restart"
echo "  Q / Ctrl-C: Quit"
echo "================================"
echo

./target/release/fakelaxian
