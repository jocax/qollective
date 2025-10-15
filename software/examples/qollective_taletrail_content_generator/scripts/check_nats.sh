#!/usr/bin/env bash

# check_nats.sh - Validate NATS server availability
# Checks if NATS server is accessible on localhost:5222

set +e  # Don't exit on error for connection checks

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

NATS_HOST="localhost"
NATS_PORT="5222"
TIMEOUT=2

# Function to check NATS connectivity
check_nats_connection() {
    local host=$1
    local port=$2

    # Try multiple methods to check connectivity

    # Method 1: Try nc (netcat)
    if command -v nc &> /dev/null; then
        if timeout $TIMEOUT nc -zv "$host" "$port" &> /dev/null; then
            return 0
        fi
    fi

    # Method 2: Try with /dev/tcp (bash built-in)
    if timeout $TIMEOUT bash -c "exec 3<>/dev/tcp/$host/$port" 2>/dev/null; then
        exec 3>&-  # Close the file descriptor
        return 0
    fi

    # Method 3: Try curl
    if command -v curl &> /dev/null; then
        if timeout $TIMEOUT curl -s "telnet://$host:$port" &> /dev/null; then
            return 0
        fi
    fi

    return 1
}

echo -e "${YELLOW}Checking NATS server...${NC}"

# Check localhost first
if check_nats_connection "$NATS_HOST" "$NATS_PORT"; then
    echo -e "${GREEN}✓ NATS server accessible at $NATS_HOST:$NATS_PORT${NC}"
    exit 0
fi

# If localhost fails, try 127.0.0.1
if check_nats_connection "127.0.0.1" "$NATS_PORT"; then
    echo -e "${GREEN}✓ NATS server accessible at 127.0.0.1:$NATS_PORT${NC}"
    exit 0
fi

# Both failed
echo -e "${RED}✗ NATS server not accessible at $NATS_HOST:$NATS_PORT${NC}"
echo -e "${RED}  Please ensure NATS server is running with TLS on port $NATS_PORT${NC}"
exit 1
