#!/usr/bin/env bash
# ==============================================================================
# AegisNode Safe Uninstallation Script
# Gỡ bỏ an toàn AegisNode Agent, duy trì toàn bộ ruleset không thuộc AegisNode
# ==============================================================================

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

PURGE=false

while [[ $# -gt 0 ]]; do
  case $1 in
    --purge)
      PURGE=true
      shift
      ;;
    *)
      shift
      ;;
  esac
done

echo -e "${CYAN}[AegisNode Uninstall] Starting safe uninstallation...${NC}"

if [ "$(id -u)" -ne 0 ]; then
    echo -e "${RED}Error: Uninstalling AegisNode requires root permissions.${NC}" >&2
    exit 1
fi

# 1. Stop and disable systemd service
for SVC in aegisnode-agent.service aegisnode-local.service; do
    if systemctl is-active --quiet "$SVC" 2>/dev/null; then
        echo -e "${CYAN}[1/4] Stopping $SVC...${NC}"
        systemctl stop "$SVC"
    fi

    if systemctl is-enabled --quiet "$SVC" 2>/dev/null; then
        echo -e "${CYAN}[2/4] Disabling $SVC...${NC}"
        systemctl disable "$SVC"
    fi

    if [ -f "/etc/systemd/system/$SVC" ]; then
        rm -f "/etc/systemd/system/$SVC"
        systemctl daemon-reload
    fi
done


# 2. Xóa binary
echo -e "${CYAN}[3/4] Removing binary /usr/local/bin/aegisnode...${NC}"
rm -f /usr/local/bin/aegisnode

# 3. Dọn dẹp nftables table thuộc AegisNode (BẢO TỒN 100% TABLES CỦA DOCKER/FIREWALLD/HỆ THỐNG)
echo -e "${CYAN}[4/4] Cleaning up AegisNode nftables tables...${NC}"
if command -v nft &>/dev/null; then
    nft delete table inet aegis_filter 2>/dev/null || true
    nft delete table ip aegis_nat 2>/dev/null || true
    echo -e "${GREEN}✓ Removed AegisNode nftables tables (inet aegis_filter, ip aegis_nat).${NC}"
fi

# 4. Xử lý purge data/config nếu yêu cầu
if [ "$PURGE" = true ]; then
    echo -e "${YELLOW}Purging data and configuration files (--purge specified)...${NC}"
    rm -rf /etc/aegisnode
    rm -rf /var/lib/aegisnode
    rm -rf /run/aegisnode
    echo -e "${GREEN}✓ Purged /etc/aegisnode and /var/lib/aegisnode.${NC}"
else
    echo -e "${YELLOW}Notice: Database (/var/lib/aegisnode) and Configuration (/etc/aegisnode) were preserved.${NC}"
    echo -e "${YELLOW}        Use 'sudo ./scripts/uninstall.sh --purge' to permanently delete them.${NC}"
fi

echo -e "${GREEN}==============================================================================${NC}"
echo -e "${GREEN}★ AegisNode uninstalled cleanly!${NC}"
echo -e "${GREEN}==============================================================================${NC}"
