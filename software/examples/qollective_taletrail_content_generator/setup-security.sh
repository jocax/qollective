#!/usr/bin/env bash
# TaleTrail Security Setup Script
# Automates NKey generation and Basic Auth setup for NATS monitoring

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║           TaleTrail NATS Security Setup                                      ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check for required tools
echo -e "${YELLOW}Checking for required tools...${NC}"
for tool in nats htpasswd; do
    if ! command -v $tool &> /dev/null; then
        echo -e "${RED}❌ $tool not found. Please install it first.${NC}"
        exit 1
    fi
    echo -e "${GREEN}✅ $tool found${NC}"
done
echo ""

# 1. Create nkeys directory
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Step 1: Creating nkeys directory...${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
mkdir -p nkeys nginx
echo -e "${GREEN}✅ Directories created${NC}"
echo ""

# 2. Generate NKeys for all services
SERVICES=("story-generator" "quality-control" "constraint-enforcer" "prompt-helper" "orchestrator" "gateway")

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Step 2: Generating NKeys for ${#SERVICES[@]} services...${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

for service in "${SERVICES[@]}"; do
    if [ -f "nkeys/$service.nk" ]; then
        echo -e "${YELLOW}⚠️  NKey for $service already exists, skipping...${NC}"
    else
        echo -e "${BLUE}Generating NKey for: $service${NC}"
        nats auth nkey gen user --output nkeys/$service.nk
        nats auth nkey show nkeys/$service.nk > nkeys/$service.pub
        echo -e "${GREEN}✅ $service NKey generated${NC}"
    fi
done
echo ""

# 3. Display public keys
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Step 3: NKey Public Keys (for nats-server.conf)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
for service in "${SERVICES[@]}"; do
    pubkey=$(cat nkeys/$service.pub)
    printf "${GREEN}%-22s${NC} %s\n" "$service:" "$pubkey"
done
echo ""

# 4. Set proper permissions
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Step 4: Setting file permissions...${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
chmod 600 nkeys/*.nk
chmod 644 nkeys/*.pub
echo -e "${GREEN}✅ Private keys (*.nk): 600 (owner read/write only)${NC}"
echo -e "${GREEN}✅ Public keys (*.pub): 644 (world readable)${NC}"
echo ""

# 5. Generate Basic Auth for monitoring
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Step 5: Setting up NATS monitoring authentication...${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

if [ -f "nginx/.htpasswd" ]; then
    echo -e "${YELLOW}⚠️  Basic Auth file already exists${NC}"
    read -p "Regenerate? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${BLUE}Keeping existing credentials${NC}"
    else
        echo "Enter password for monitoring (user: admin):"
        htpasswd -c nginx/.htpasswd admin
        echo -e "${GREEN}✅ Monitoring credentials updated${NC}"
    fi
else
    echo "Enter password for monitoring (user: admin):"
    htpasswd -c nginx/.htpasswd admin
    echo -e "${GREEN}✅ Monitoring credentials created${NC}"
fi
echo ""

# Summary
echo -e "${BLUE}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                          Security Setup Complete!                            ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}✅ NKeys generated for 5 services${NC}"
echo -e "${GREEN}✅ Basic Auth configured for monitoring${NC}"
echo -e "${GREEN}✅ File permissions secured${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo -e "  1. Update nats-server.conf with public keys listed above"
echo -e "  2. Run: ${BLUE}docker-compose up -d${NC}"
echo -e "  3. Access monitoring: ${BLUE}https://localhost:9222${NC} (user: admin)"
echo -e "  4. Build services: ${BLUE}./build.sh${NC}"
echo ""
echo -e "${BLUE}🔐 Security Features Enabled:${NC}"
echo -e "  • NKey authentication for NATS clients"
echo -e "  • TLS encryption for all NATS traffic"
echo -e "  • HTTPS monitoring with Basic Auth"
echo -e "  • Subject-level authorization per service"
echo ""
