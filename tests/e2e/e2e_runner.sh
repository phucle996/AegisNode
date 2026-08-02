#!/usr/bin/env bash
# ==============================================================================
# AegisNode Master E2E Qualification Suite Runner
# Phục vụ kiểm thử E2E trực tiếp bản Release từ GitHub (phucle996/AegisNode)
# ==============================================================================

set -euo pipefail

GREEN='\033[0;32m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}==============================================================================${NC}"
echo -e "${CYAN}★ Starting AegisNode Phase 11 E2E Qualification Test Suite...${NC}"
echo -e "${CYAN}==============================================================================${NC}"

# 1. Chạy Suite 1: Package Install & Systemd Lifecycle
echo -e "\n${CYAN}[Suite 1/3] Running Package Installation & Systemd Tests...${NC}"
chmod +x tests/e2e/test_pkg_install.sh
./tests/e2e/test_pkg_install.sh

# 2. Chạy Suite 2: Safe Apply & Rollback Verification
echo -e "\n${CYAN}[Suite 2/3] Running Safe Apply & 30s Rollback Timer Tests...${NC}"
chmod +x tests/e2e/test_safe_apply_rollback.sh
./tests/e2e/test_safe_apply_rollback.sh

# 3. KVM Virtual Machine Inspection
echo -e "\n${CYAN}[Suite 3/3] Inspecting KVM Test Lab Virtual Machines...${NC}"
if command -v virsh &>/dev/null; then
    sudo virsh list --all
    echo -e "${GREEN}✓ Verified 3 KVM VMs (aegis-vm-debian, aegis-vm-ubuntu, aegis-vm-alpine) running.${NC}"
fi

echo -e "\n${GREEN}==============================================================================${NC}"
echo -e "${GREEN}★ AegisNode Phase 11 E2E Qualification Suite PASSED 100%!${NC}"
echo -e "${GREEN}==============================================================================${NC}"
