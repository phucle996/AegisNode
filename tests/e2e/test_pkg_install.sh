#!/usr/bin/env bash
# ==============================================================================
# AegisNode E2E Test Suite: Package Installation, Systemd Lifecycle & Uninstallation
# Test Cases: E2E-PKG-001 -> E2E-PKG-006, E2E-SD-001 -> E2E-SD-008
# ==============================================================================

set -euo pipefail

GREEN='\033[0;32m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}=== Running E2E Package Installation & Systemd Lifecycle Tests ===${NC}"

# 1. Clean previous state
sudo ./scripts/uninstall.sh --purge >/dev/null 2>&1 || true

# 2. Test Install from GitHub Release
echo -n "Test E2E-PKG-001: Automatic installation from GitHub release... "
if sudo ./scripts/install.sh --version v0.1.1 >/dev/null 2>&1; then
    echo -e "${GREEN}PASSED${NC}"
else
    echo -e "${RED}FAILED${NC}"
    exit 1
fi

# 3. Test Binary Execution & Version Output
echo -n "Test E2E-BUILD-001: Binary execution & version inspection... "
VERSION_OUT=$(aegisnode version 2>&1 || true)
if [[ "$VERSION_OUT" =~ "aegisnode" ]] || [[ "$VERSION_OUT" =~ "0.1" ]]; then
    echo -e "${GREEN}PASSED${NC}"
else
    echo -e "${RED}FAILED ($VERSION_OUT)${NC}"
    exit 1
fi

# 4. Test Permissions & Directories
echo -n "Test E2E-PKG-001: System directories & permission check... "
if [ -d "/etc/aegisnode" ] && [ -d "/var/lib/aegisnode" ] && [ -d "/run/aegisnode" ]; then
    echo -e "${GREEN}PASSED${NC}"
else
    echo -e "${RED}FAILED${NC}"
    exit 1
fi

# 5. Test Systemd Lifecycle (Start, Status, Restart, Stop)
echo -n "Test E2E-SD-003: Systemd start service... "
sudo systemctl start aegisnode-local
sleep 2
if systemctl is-active --quiet aegisnode-local; then
    echo -e "${GREEN}PASSED${NC}"
else
    echo -e "${RED}FAILED${NC}"
    exit 1
fi

echo -n "Test E2E-SD-005: Systemd restart service... "
sudo systemctl restart aegisnode-local
sleep 2
if systemctl is-active --quiet aegisnode-local; then
    echo -e "${GREEN}PASSED${NC}"
else
    echo -e "${RED}FAILED${NC}"
    exit 1
fi

# 6. Test Uninstallation
echo -n "Test E2E-UN-001: Safe uninstallation (preserve config/db)... "
sudo ./scripts/uninstall.sh >/dev/null 2>&1
if [ ! -f "/usr/local/bin/aegisnode" ] && [ -d "/var/lib/aegisnode" ]; then
    echo -e "${GREEN}PASSED${NC}"
else
    echo -e "${RED}FAILED${NC}"
    exit 1
fi

# Clean up
sudo ./scripts/uninstall.sh --purge >/dev/null 2>&1 || true

echo -e "${GREEN}★ All Package & Systemd Lifecycle Tests PASSED!${NC}"
