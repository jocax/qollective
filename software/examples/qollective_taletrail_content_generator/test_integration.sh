#!/usr/bin/env bash
# test_integration.sh - Main integration test orchestrator
# Validates infrastructure and runs integration tests if all checks pass

set -e

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Print header
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Infrastructure Validation${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Track overall status
VALIDATION_FAILED=false

# Run NATS check
if ! "$SCRIPT_DIR/scripts/check_nats.sh"; then
    VALIDATION_FAILED=true
fi
echo ""

# Run API keys check
if ! "$SCRIPT_DIR/scripts/check_api_keys.sh"; then
    VALIDATION_FAILED=true
fi
echo ""

# Run certificate check
if ! "$SCRIPT_DIR/scripts/check_certs.sh"; then
    VALIDATION_FAILED=true
fi
echo ""

# Check if any validation failed
if [ "$VALIDATION_FAILED" = true ]; then
    echo -e "${RED}========================================${NC}"
    echo -e "${RED}Infrastructure Validation Failed${NC}"
    echo -e "${RED}========================================${NC}"
    echo ""
    echo -e "${YELLOW}Please resolve the above issues before running integration tests${NC}"
    exit 1
fi

# All checks passed
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}All Infrastructure Checks Passed${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Export environment variable to enable infrastructure tests
export ENABLE_INFRA_TESTS=1

# Check if cargo-nextest is available
if ! command -v cargo-nextest &> /dev/null; then
    echo -e "${YELLOW}cargo-nextest not found, using 'cargo test' instead${NC}"
    echo -e "${BLUE}Running integration tests...${NC}"
    echo ""
    cargo test
else
    echo -e "${BLUE}Running integration tests with nextest...${NC}"
    echo ""
    # Try with infra profile first, fallback to default profile
    if cargo nextest run --profile infra 2>&1 | grep -q "profile.*not found"; then
        echo -e "${YELLOW}Note: 'infra' profile not found, using default profile${NC}"
        cargo nextest run
    else
        cargo nextest run --profile infra
    fi
fi

# Check test result
if [ $? -eq 0 ]; then
    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}Integration Tests Passed${NC}"
    echo -e "${GREEN}========================================${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}========================================${NC}"
    echo -e "${RED}Integration Tests Failed${NC}"
    echo -e "${RED}========================================${NC}"
    exit 1
fi
