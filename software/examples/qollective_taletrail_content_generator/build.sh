#!/usr/bin/env bash

# TaleTrail Content Generator - Build Script
# Builds all workspace members in dependency order

set -e

# Color definitions
BLUE='\033[0;34m'
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Build order based on dependencies
CRATES=(
    "shared-types"
    "orchestrator"
    "story-generator"
    "quality-control"
    "constraint-enforcer"
    "prompt-helper"
    "gateway"
)

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}TaleTrail Content Generator - Build${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Track build status
BUILD_FAILED=0
SUCCESSFUL_BUILDS=()
FAILED_BUILDS=()

# Build each crate in order
for crate in "${CRATES[@]}"; do
    echo -e "${BLUE}Building ${crate}...${NC}"

    if cargo build --package "$crate" --release 2>&1; then
        echo -e "${GREEN}✓ ${crate} built successfully${NC}"
        SUCCESSFUL_BUILDS+=("$crate")
    else
        echo -e "${RED}✗ ${crate} build failed${NC}"
        FAILED_BUILDS+=("$crate")
        BUILD_FAILED=1
    fi
    echo ""
done

# Display build summary
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Build Summary${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

if [ ${#SUCCESSFUL_BUILDS[@]} -gt 0 ]; then
    echo -e "${GREEN}Successful builds (${#SUCCESSFUL_BUILDS[@]}):${NC}"
    for crate in "${SUCCESSFUL_BUILDS[@]}"; do
        echo -e "${GREEN}  ✓ ${crate}${NC}"
    done
    echo ""
fi

if [ ${#FAILED_BUILDS[@]} -gt 0 ]; then
    echo -e "${RED}Failed builds (${#FAILED_BUILDS[@]}):${NC}"
    for crate in "${FAILED_BUILDS[@]}"; do
        echo -e "${RED}  ✗ ${crate}${NC}"
    done
    echo ""
fi

if [ $BUILD_FAILED -eq 0 ]; then
    echo -e "${GREEN}All builds completed successfully!${NC}"
    exit 0
else
    echo -e "${RED}Build failed. Please fix errors and try again.${NC}"
    exit 1
fi
