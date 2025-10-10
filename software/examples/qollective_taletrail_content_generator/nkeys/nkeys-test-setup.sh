#!/usr/bin/env bash

#######################################################################
# NKey Setup Verification Script
#
# This script verifies that the NKey authentication system is properly
# configured and all components are in place.
#######################################################################

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  NKey Setup Verification${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

ERRORS=0
WARNINGS=0

# Test 1: Check if nkeys-generate.sh exists and is executable
echo -e "${BLUE}[1/8]${NC} Checking nkeys-generate.sh..."
if [ -x "$SCRIPT_DIR/nkeys-generate.sh" ]; then
    echo -e "  ${GREEN}✓${NC} Script exists and is executable"
else
    echo -e "  ${RED}✗${NC} Script not found or not executable"
    ((ERRORS++))
fi

# Test 2: Check if README.md exists
echo -e "${BLUE}[2/8]${NC} Checking README.md..."
if [ -f "$SCRIPT_DIR/README.md" ]; then
    echo -e "  ${GREEN}✓${NC} Documentation exists"
else
    echo -e "  ${RED}✗${NC} README.md not found"
    ((ERRORS++))
fi

# Test 3: Check if .gitignore properly excludes generated files
echo -e "${BLUE}[3/8]${NC} Checking .gitignore configuration..."
GITIGNORE="$PROJECT_ROOT/.gitignore"
if [ -f "$GITIGNORE" ]; then
    if grep -q "nkeys/\*\.nk" "$GITIGNORE" && \
       grep -q "nkeys/\*\.pub" "$GITIGNORE" && \
       grep -q "nkeys/users\.conf" "$GITIGNORE"; then
        echo -e "  ${GREEN}✓${NC} .gitignore properly configured"
    else
        echo -e "  ${RED}✗${NC} .gitignore missing required entries"
        ((ERRORS++))
    fi
else
    echo -e "  ${RED}✗${NC} .gitignore not found"
    ((ERRORS++))
fi

# Test 4: Check if nats-server.conf uses include directive
echo -e "${BLUE}[4/8]${NC} Checking nats-server.conf..."
NATS_CONF="$PROJECT_ROOT/nats-server.conf"
if [ -f "$NATS_CONF" ]; then
    if grep -q 'include "nkeys/users.conf"' "$NATS_CONF"; then
        echo -e "  ${GREEN}✓${NC} NATS config uses include directive"
    else
        echo -e "  ${RED}✗${NC} NATS config missing include directive"
        ((ERRORS++))
    fi
else
    echo -e "  ${RED}✗${NC} nats-server.conf not found"
    ((ERRORS++))
fi

# Test 5: Check if NKey files exist
echo -e "${BLUE}[5/8]${NC} Checking for generated NKey files..."
SERVICES=("story-generator" "quality-control" "constraint-enforcer" "prompt-helper" "orchestrator" "gateway" "nats-cli")
MISSING_KEYS=0
for service in "${SERVICES[@]}"; do
    if [ ! -f "$SCRIPT_DIR/${service}.nk" ]; then
        echo -e "  ${YELLOW}⚠${NC} Missing private key: ${service}.nk"
        ((MISSING_KEYS++))
    fi
    if [ ! -f "$SCRIPT_DIR/${service}.pub" ]; then
        echo -e "  ${YELLOW}⚠${NC} Missing public key: ${service}.pub"
        ((MISSING_KEYS++))
    fi
done

if [ $MISSING_KEYS -eq 0 ]; then
    echo -e "  ${GREEN}✓${NC} All NKey files present (7 services x 2 files = 14 files)"
else
    echo -e "  ${YELLOW}⚠${NC} $MISSING_KEYS key files missing (run ./nkeys-generate.sh)"
    ((WARNINGS++))
fi

# Test 6: Check if users.conf exists
echo -e "${BLUE}[6/8]${NC} Checking users.conf..."
if [ -f "$SCRIPT_DIR/users.conf" ]; then
    echo -e "  ${GREEN}✓${NC} users.conf exists"

    # Verify it contains the required sections
    if grep -q "nkey:" "$SCRIPT_DIR/users.conf" && \
       grep -q "permissions:" "$SCRIPT_DIR/users.conf"; then
        echo -e "  ${GREEN}✓${NC} users.conf has authorization structure"
    else
        echo -e "  ${YELLOW}⚠${NC} users.conf may be invalid"
        ((WARNINGS++))
    fi
else
    echo -e "  ${YELLOW}⚠${NC} users.conf not found (run ./nkeys-generate.sh)"
    ((WARNINGS++))
fi

# Test 7: Check file permissions on private keys
echo -e "${BLUE}[7/8]${NC} Checking private key permissions..."
BAD_PERMS=0
for service in "${SERVICES[@]}"; do
    if [ -f "$SCRIPT_DIR/${service}.nk" ]; then
        PERMS=$(stat -f "%Lp" "$SCRIPT_DIR/${service}.nk" 2>/dev/null || stat -c "%a" "$SCRIPT_DIR/${service}.nk" 2>/dev/null || echo "unknown")
        if [ "$PERMS" != "600" ]; then
            echo -e "  ${YELLOW}⚠${NC} ${service}.nk has permissions $PERMS (should be 600)"
            ((BAD_PERMS++))
        fi
    fi
done

if [ $BAD_PERMS -eq 0 ]; then
    echo -e "  ${GREEN}✓${NC} All private keys have correct permissions (600)"
else
    echo -e "  ${YELLOW}⚠${NC} $BAD_PERMS keys have incorrect permissions"
    ((WARNINGS++))
fi

# Test 8: Check if nats command is available
echo -e "${BLUE}[8/8]${NC} Checking nats command availability..."
if command -v nats &> /dev/null; then
    NATS_VERSION=$(nats --version 2>&1 | head -1 || echo "unknown")
    echo -e "  ${GREEN}✓${NC} nats command available ($NATS_VERSION)"
else
    echo -e "  ${YELLOW}⚠${NC} nats command not found (needed to regenerate keys)"
    echo -e "     Install: brew install nats-io/nats-tools/nats"
    ((WARNINGS++))
fi

# Summary
echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Verification Summary${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
    echo -e "${GREEN}✓ All checks passed!${NC}"
    echo ""
    echo "Your NKey authentication system is properly configured."
    exit 0
elif [ $ERRORS -eq 0 ]; then
    echo -e "${YELLOW}⚠ $WARNINGS warning(s) found${NC}"
    echo ""
    echo "The setup is mostly correct, but you may want to address the warnings above."
    echo ""
    echo "To generate or regenerate NKey files, run:"
    echo "  cd nkeys && ./nkeys-generate.sh"
    exit 0
else
    echo -e "${RED}✗ $ERRORS error(s) found${NC}"
    if [ $WARNINGS -gt 0 ]; then
        echo -e "${YELLOW}⚠ $WARNINGS warning(s) found${NC}"
    fi
    echo ""
    echo "Please fix the errors above before proceeding."
    exit 1
fi
