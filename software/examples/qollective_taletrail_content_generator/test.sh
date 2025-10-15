#!/usr/bin/env bash

# TaleTrail Content Generator - Test Script
# Runs unit tests by default, provides guidance for infrastructure tests

set -e

# Color definitions
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Print usage information
show_usage() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}TaleTrail Content Generator - Test Suite${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
    echo "Usage: ./test.sh [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -h, --help              Show this help message"
    echo "  -a, --all               Run all tests (unit + integration)"
    echo "  -i, --integration       Run integration tests only"
    echo "  -c, --ci                Run CI tests (excludes slow tests)"
    echo "  -p, --profile PROFILE   Run specific nextest profile"
    echo ""
    echo "Examples:"
    echo "  ./test.sh                    # Run unit tests (default)"
    echo "  ./test.sh --all              # Run all tests including integration"
    echo "  ./test.sh --integration      # Run integration tests only"
    echo "  ./test.sh --ci               # Run CI-optimized tests"
    echo ""
    echo -e "${YELLOW}Note: Integration tests require infrastructure (NATS, API keys)${NC}"
    echo -e "${YELLOW}Run ./test_integration.sh to check and run integration tests${NC}"
    echo ""
}

# Parse command line arguments
PROFILE="default"
RUN_INTEGRATION=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_usage
            exit 0
            ;;
        -a|--all)
            PROFILE="all"
            RUN_INTEGRATION=true
            shift
            ;;
        -i|--integration)
            PROFILE="infra"
            RUN_INTEGRATION=true
            shift
            ;;
        -c|--ci)
            PROFILE="ci"
            shift
            ;;
        -p|--profile)
            PROFILE="$2"
            shift 2
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo ""
            show_usage
            exit 1
            ;;
    esac
done

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Running Tests (Profile: ${PROFILE})${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# If running integration tests, check if infrastructure is available
if [ "$RUN_INTEGRATION" = true ]; then
    if [ -z "$ENABLE_INFRA_TESTS" ]; then
        echo -e "${YELLOW}⚠ Infrastructure tests require ENABLE_INFRA_TESTS=1${NC}"
        echo -e "${YELLOW}  Run ./test_integration.sh to validate and run integration tests${NC}"
        echo ""
        echo -e "Proceeding anyway - infrastructure tests will be skipped gracefully"
        echo ""
    fi
fi

# Run tests
echo -e "${GREEN}▶ Running cargo nextest with profile: ${PROFILE}${NC}"
echo ""

if command -v cargo-nextest &> /dev/null; then
    cargo nextest run --workspace --profile "$PROFILE"
    TEST_EXIT_CODE=$?
else
    echo -e "${YELLOW}⚠ cargo-nextest not found, falling back to cargo test${NC}"
    echo -e "${YELLOW}  Install with: cargo install cargo-nextest${NC}"
    echo ""
    cargo test --workspace
    TEST_EXIT_CODE=$?
fi

echo ""
echo -e "${BLUE}========================================${NC}"

if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✓ Tests completed successfully${NC}"
else
    echo -e "${RED}✗ Tests failed with exit code: ${TEST_EXIT_CODE}${NC}"
fi

echo -e "${BLUE}========================================${NC}"

# Show helpful tips on success
if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo ""
    echo -e "${BLUE}Tip:${NC} Run different test profiles:"
    echo "  • Unit tests only:       ./test.sh"
    echo "  • Integration tests:     ./test_integration.sh"
    echo "  • CI tests:              ./test.sh --ci"
    echo "  • All tests:             ./test.sh --all"
    echo ""
fi

exit $TEST_EXIT_CODE
