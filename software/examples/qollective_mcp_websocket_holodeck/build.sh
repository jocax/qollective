#!/bin/bash
# ABOUTME: Build script for holodeck workspace - compiles all components
# ABOUTME: Supports both debug and release builds with comprehensive validation

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default build profile
BUILD_TYPE="debug"
VALIDATE=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            BUILD_TYPE="release"
            shift
            ;;
        --validate)
            VALIDATE=true
            shift
            ;;
        --help)
            echo "Usage: $0 [--release] [--validate]"
            echo "  --release: Build in release mode"
            echo "  --validate: Run validation checks (clippy, fmt)"
            exit 0
            ;;
        *)
            echo "Unknown option $1"
            exit 1
            ;;
    esac
done

echo -e "${GREEN}🚀 Building Holodeck Workspace (${BUILD_TYPE})${NC}"

# Clean previous builds
echo -e "${YELLOW}🧹 Cleaning workspace${NC}"
cargo clean

# Check formatting
if [ "$VALIDATE" = true ]; then
    echo -e "${YELLOW}📝 Checking code formatting${NC}"
    if ! cargo fmt --all -- --check; then
        echo -e "${RED}❌ Code formatting check failed${NC}"
        echo -e "${YELLOW}💡 Run 'cargo fmt --all' to fix formatting${NC}"
        exit 1
    fi
    echo -e "${GREEN}✅ Code formatting passed${NC}"
fi

# Run clippy
if [ "$VALIDATE" = true ]; then
    echo -e "${YELLOW}🔍 Running clippy${NC}"
    if ! cargo clippy --workspace --all-targets -- -D warnings; then
        echo -e "${RED}❌ Clippy found issues${NC}"
        exit 1
    fi
    echo -e "${GREEN}✅ Clippy passed${NC}"
fi

# Build workspace
echo -e "${YELLOW}🔨 Building workspace${NC}"
if [ "$BUILD_TYPE" = "release" ]; then
    cargo build --workspace --release
else
    cargo build --workspace
fi

echo -e "${GREEN}✅ Build completed successfully${NC}"

# Report build artifacts
echo -e "${YELLOW}📦 Build artifacts:${NC}"
if [ "$BUILD_TYPE" = "release" ]; then
    BINARY_PATH="target/release"
else
    BINARY_PATH="target/debug"
fi

# List built binaries (main.rs files that can be run)
find "$BINARY_PATH" -maxdepth 1 -type f -executable -name "holodeck-*" | sort | while read binary; do
    echo "  📄 $(basename "$binary")"
done

echo -e "${GREEN}🎉 Holodeck build process complete${NC}"