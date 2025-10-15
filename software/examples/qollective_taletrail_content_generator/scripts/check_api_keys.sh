#!/usr/bin/env bash

# check_api_keys.sh - Validate required API keys are set
# Checks for ANTHROPIC_API_KEY and OPENAI_API_KEY environment variables

set -e

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Required API keys
REQUIRED_KEYS=("ANTHROPIC_API_KEY" "OPENAI_API_KEY")

echo -e "${YELLOW}Checking API keys...${NC}"

ALL_PRESENT=true

for key_name in "${REQUIRED_KEYS[@]}"; do
    # Use indirect variable expansion to check if the key is set and non-empty
    if [ -z "${!key_name}" ]; then
        echo -e "${RED}✗ $key_name not set${NC}"
        ALL_PRESENT=false
    else
        # Show that key is set without revealing value
        echo -e "${GREEN}✓ $key_name is set${NC}"
    fi
done

if [ "$ALL_PRESENT" = true ]; then
    exit 0
else
    echo -e "${RED}  Please set missing API keys as environment variables${NC}"
    exit 1
fi
