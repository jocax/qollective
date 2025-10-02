#!/bin/bash
# ABOUTME: Test script for holodeck workspace - runs all tests with proper isolation  
# ABOUTME: Supports unit tests, integration tests, and schema validation tests

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default test configuration
TEST_TYPE="all"
VERBOSE=false
SINGLE_THREADED=true

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --unit)
            TEST_TYPE="unit"
            shift
            ;;
        --integration)
            TEST_TYPE="integration"
            shift
            ;;
        --schema)
            TEST_TYPE="schema"
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --parallel)
            SINGLE_THREADED=false
            shift
            ;;
        --help)
            echo "Usage: $0 [--unit] [--integration] [--schema] [--verbose] [--parallel]"
            echo "  --unit: Run only unit tests"
            echo "  --integration: Run only integration tests"
            echo "  --schema: Run only schema validation tests"
            echo "  --verbose: Show detailed test output"
            echo "  --parallel: Allow parallel test execution (default: single-threaded)"
            exit 0
            ;;
        *)
            echo "Unknown option $1"
            exit 1
            ;;
    esac
done

echo -e "${GREEN}🧪 Running Holodeck Test Suite${NC}"

# Set test threading
if [ "$SINGLE_THREADED" = true ]; then
    TEST_ARGS="-- --test-threads=1"
    echo -e "${BLUE}ℹ️  Running tests single-threaded to prevent race conditions${NC}"
else
    TEST_ARGS=""
    echo -e "${BLUE}ℹ️  Running tests in parallel${NC}"
fi

# Add verbose flag if requested
if [ "$VERBOSE" = true ]; then
    TEST_ARGS="$TEST_ARGS --nocapture"
fi

# Function to run specific test type
run_test_type() {
    local test_name=$1
    local test_command=$2
    
    echo -e "${YELLOW}🔍 Running $test_name tests${NC}"
    
    if eval $test_command; then
        echo -e "${GREEN}✅ $test_name tests passed${NC}"
        return 0
    else
        echo -e "${RED}❌ $test_name tests failed${NC}"
        return 1
    fi
}

# Track overall test results
TOTAL_TESTS=0
PASSED_TESTS=0

# Run unit tests
if [ "$TEST_TYPE" = "all" ] || [ "$TEST_TYPE" = "unit" ]; then
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    if run_test_type "Unit" "cargo test --workspace --lib $TEST_ARGS"; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
    fi
fi

# Run integration tests (if they exist)
if [ "$TEST_TYPE" = "all" ] || [ "$TEST_TYPE" = "integration" ]; then
    if [ -d "tests" ]; then
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
        if run_test_type "Integration" "cargo test --workspace --test '*' $TEST_ARGS"; then
            PASSED_TESTS=$((PASSED_TESTS + 1))
        fi
    else
        echo -e "${BLUE}ℹ️  No integration tests directory found, skipping${NC}"
    fi
fi

# Run schema validation tests (from shared-types)
if [ "$TEST_TYPE" = "all" ] || [ "$TEST_TYPE" = "schema" ]; then
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    if run_test_type "Schema Validation" "cargo test -p shared-types $TEST_ARGS"; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
    fi
fi

# Run doc tests
if [ "$TEST_TYPE" = "all" ]; then
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    if run_test_type "Documentation" "cargo test --workspace --doc $TEST_ARGS"; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
    fi
fi

# Print final results
echo -e "\n${BLUE}📊 Test Results Summary:${NC}"
echo -e "  Tests Run: $TOTAL_TESTS"
echo -e "  Passed: ${GREEN}$PASSED_TESTS${NC}"
echo -e "  Failed: ${RED}$((TOTAL_TESTS - PASSED_TESTS))${NC}"

if [ $PASSED_TESTS -eq $TOTAL_TESTS ]; then
    echo -e "\n${GREEN}🎉 All tests passed successfully!${NC}"
    exit 0
else
    echo -e "\n${RED}💥 Some tests failed${NC}"
    exit 1
fi