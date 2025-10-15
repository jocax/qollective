#!/usr/bin/env bash

# check_certs.sh - Validate TLS certificate files
# Checks for required certificate files in ../../../tests/certs/

set -e

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get script directory and construct cert path
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CERT_DIR="$SCRIPT_DIR/../../../tests/certs"

# Required certificate files
REQUIRED_CERTS=("ca.pem" "client-cert.pem" "client-key.pem")

echo -e "${YELLOW}Checking TLS certificates...${NC}"

ALL_PRESENT=true

for cert_file in "${REQUIRED_CERTS[@]}"; do
    cert_path="$CERT_DIR/$cert_file"

    if [ ! -f "$cert_path" ]; then
        echo -e "${RED}✗ $cert_file not found at $cert_path${NC}"
        ALL_PRESENT=false
    elif [ ! -r "$cert_path" ]; then
        echo -e "${RED}✗ $cert_file exists but is not readable${NC}"
        ALL_PRESENT=false
    else
        echo -e "${GREEN}✓ $cert_file exists and is readable${NC}"
    fi
done

if [ "$ALL_PRESENT" = true ]; then
    exit 0
else
    echo -e "${RED}  Please ensure all certificate files are present in $CERT_DIR${NC}"
    exit 1
fi
