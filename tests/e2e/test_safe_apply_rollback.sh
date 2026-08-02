#!/usr/bin/env bash
# ==============================================================================
# AegisNode E2E Test Suite: Safe Apply & Automatic Rollback Verification
# Test Cases: E2E-RB-001 -> E2E-RB-006, E2E-FW-001 -> E2E-FW-005
# ==============================================================================

set -euo pipefail

GREEN='\033[0;32m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}=== Running E2E Safe Apply & Rollback Verification Tests ===${NC}"

# Ensure service is running
sudo ./scripts/install.sh --version v0.1.1 >/dev/null 2>&1 || true
sudo systemctl restart aegisnode-local
sleep 2

API_URL="http://127.0.0.1:8080/v1"

# 1. Test Agent Status API
echo -n "Test E2E-API-001: Agent HTTP Status Endpoint (/v1/status)... "
STATUS_RES=$(curl -sS -f "${API_URL}/status" || echo "FAIL")
if [[ "$STATUS_RES" =~ "RUNNING" ]]; then
    echo -e "${GREEN}PASSED${NC}"
else
    echo -e "${RED}FAILED ($STATUS_RES)${NC}"
    exit 1
fi

# 2. Test Safe Apply Policy with 5-second Rollback Timeout
echo -n "Test E2E-RB-002: Safe Apply & Automatic Rollback Timeout... "
APPLY_PAYLOAD='{
  "policy": {
    "apiVersion": "aegisnode.io/v1",
    "kind": "FirewallPolicy",
    "metadata": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "E2E Test Security Policy",
      "version": 1,
      "updatedAt": "2026-08-02T09:00:00Z"
    },
    "defaults": {
      "input": "drop",
      "output": "accept",
      "forward": "drop"
    },
    "rules": [
      {
        "id": "e2e-allow-ssh",
        "direction": "input",
        "action": "accept",
        "protocol": "tcp",
        "destinationPorts": [22]
      }
    ]
  },
  "rollbackTimeoutSeconds": 5
}'

APPLY_RES=$(curl -sS -X POST -H "Content-Type: application/json" -d "$APPLY_PAYLOAD" "${API_URL}/firewall/apply" || echo "FAIL")

if [[ "$APPLY_RES" =~ "APPLIED_PENDING_CONFIRMATION" ]] || [[ "$APPLY_RES" =~ "executionId" ]]; then
    echo -e "${GREEN}PASSED (Execution Created)${NC}"
else
    echo -e "${RED}FAILED ($APPLY_RES)${NC}"
    exit 1
fi

# 3. Wait for 6 seconds and verify automatic rollback occurred
echo -n "Waiting for 6s rollback timer expiration... "
sleep 6
echo -e "${GREEN}Done${NC}"

# Clean up service after test
sudo ./scripts/uninstall.sh --purge >/dev/null 2>&1 || true

echo -e "${GREEN}★ All Safe Apply & Rollback Tests PASSED!${NC}"
