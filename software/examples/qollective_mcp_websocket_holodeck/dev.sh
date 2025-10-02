#!/bin/bash
# ABOUTME: Development script for holodeck workspace - comprehensive dev workflow
# ABOUTME: Combines build, test, and validation in a single convenient command

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Default configuration
QUICK=false
WATCH=false
FORMAT=true

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --quick)
            QUICK=true
            shift
            ;;
        --watch)
            WATCH=true
            shift
            ;;
        --no-format)
            FORMAT=false
            shift
            ;;
        --help)
            echo "Usage: $0 [--quick] [--watch] [--no-format]"
            echo "  --quick: Skip full validation (clippy, tests)"
            echo "  --watch: Watch for file changes and rebuild"
            echo "  --no-format: Skip automatic code formatting"
            exit 0
            ;;
        *)
            echo "Unknown option $1"
            exit 1
            ;;
    esac
done

# Function to run development cycle
run_dev_cycle() {
    echo -e "${PURPLE}🚀 Running Holodeck Development Cycle${NC}"
    
    # Format code
    if [ "$FORMAT" = true ]; then
        echo -e "${YELLOW}📝 Formatting code${NC}"
        cargo fmt --all
        echo -e "${GREEN}✅ Code formatted${NC}"
    fi
    
    # Quick build check
    echo -e "${YELLOW}🔨 Quick build check${NC}"
    if ! cargo check --workspace; then
        echo -e "${RED}❌ Build check failed${NC}"
        return 1
    fi
    echo -e "${GREEN}✅ Build check passed${NC}"
    
    if [ "$QUICK" = false ]; then
        # Full validation build
        echo -e "${YELLOW}🏗️  Running full build with validation${NC}"
        if ! ./build.sh --validate; then
            echo -e "${RED}❌ Full build failed${NC}"
            return 1
        fi
        
        # Run tests
        echo -e "${YELLOW}🧪 Running test suite${NC}"
        if ! ./test.sh; then
            echo -e "${RED}❌ Tests failed${NC}"
            return 1
        fi
        
        echo -e "${GREEN}🎉 Full development cycle completed successfully${NC}"
    else
        echo -e "${GREEN}✅ Quick development cycle completed${NC}"
    fi
    
    return 0
}

# Watch mode
if [ "$WATCH" = true ]; then
    echo -e "${BLUE}👁️  Starting watch mode (Ctrl+C to exit)${NC}"
    echo -e "${BLUE}ℹ️  Watching for changes in src/, Cargo.toml, and schema files${NC}"
    
    # Install cargo-watch if not available
    if ! command -v cargo-watch &> /dev/null; then
        echo -e "${YELLOW}📦 Installing cargo-watch${NC}"
        cargo install cargo-watch
    fi
    
    # Run initial development cycle
    run_dev_cycle
    
    # Watch for changes
    if [ "$QUICK" = true ]; then
        cargo watch -x "check --workspace" -s "cargo fmt --all"
    else
        cargo watch -s "./dev.sh"
    fi
else
    # Run once
    run_dev_cycle
fi