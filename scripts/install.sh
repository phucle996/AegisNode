#!/usr/bin/env bash
# ==============================================================================
# AegisNode Automatic Installation Script for Linux Hosts / KVM VMs
# Tự động tải Release Artifact từ GitHub Releases: phucle996/AegisNode
# ==============================================================================

set -euo pipefail

# Màu sắc hiển thị log Terminal
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

REPO="phucle996/AegisNode"
VERSION="v0.1.1"

echo -e "${CYAN}[AegisNode Install] Safe Linux Firewall Agent Installation...${NC}"

# Parse optional --version flag
while [[ $# -gt 0 ]]; do
  case $1 in
    -v|--version)
      VERSION="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done

# 1. Kiểm tra quyền root
if [ "$(id -u)" -ne 0 ]; then
    echo -e "${RED}Error: Installing AegisNode requires root permissions. Please run with sudo.${NC}" >&2
    exit 1
fi

# 2. Kiểm tra binary nftables và systemd
if ! command -v nft &> /dev/null; then
    echo -e "${RED}Error: 'nft' command (nftables) not found. Please install nftables first.${NC}" >&2
    exit 1
fi

if ! command -v systemctl &> /dev/null; then
    echo -e "${RED}Error: 'systemctl' command not found. AegisNode requires systemd.${NC}" >&2
    exit 1
fi

# 3. Thiết lập thư mục hệ thống
echo -e "${CYAN}[1/5] Setting up system directories...${NC}"
mkdir -p /etc/aegisnode
mkdir -p /var/lib/aegisnode
mkdir -p /run/aegisnode

chmod 0755 /etc/aegisnode
chmod 0750 /var/lib/aegisnode
chmod 0750 /run/aegisnode

# 4. Tạo user và group phân quyền aegisnode
echo -e "${CYAN}[2/5] Configuring system user and group 'aegisnode'...${NC}"
if ! getent group aegisnode >/dev/null; then
    groupadd --system aegisnode
fi

if ! getent passwd aegisnode >/dev/null; then
    useradd --system --gid aegisnode --no-create-home --shell /bin/false aegisnode
fi

# 5. Tải Release Binary từ GitHub Releases phucle996/AegisNode
echo -e "${CYAN}[3/5] Downloading AegisNode binary release (${VERSION}) from GitHub...${NC}"
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

ARCH=$(uname -m)
TARBALL="aegisnode-${VERSION}-${ARCH}-unknown-linux-gnu.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${TARBALL}"

echo -e "${CYAN}   URL: ${DOWNLOAD_URL}${NC}"

if curl -sSL -f -o "${TMP_DIR}/${TARBALL}" "$DOWNLOAD_URL"; then
    tar -xzf "${TMP_DIR}/${TARBALL}" -C "$TMP_DIR"
    cp "${TMP_DIR}/aegisnode" /usr/local/bin/aegisnode
    chmod 0755 /usr/local/bin/aegisnode
    echo -e "${GREEN}✓ Downloaded & installed binary to /usr/local/bin/aegisnode${NC}"
else
    echo -e "${YELLOW}Warning: Could not download release ${VERSION} from GitHub. Checking local binary build...${NC}"
    LOCAL_BIN="target/release/aegisnode"
    if [ -f "$LOCAL_BIN" ]; then
        cp "$LOCAL_BIN" /usr/local/bin/aegisnode
        chmod 0755 /usr/local/bin/aegisnode
        echo -e "${GREEN}✓ Installed local binary from $LOCAL_BIN${NC}"
    elif [ -f "/usr/local/bin/aegisnode" ]; then
        echo -e "${GREEN}✓ Using existing binary at /usr/local/bin/aegisnode${NC}"
    else
        echo -e "${RED}Error: Failed to find or download aegisnode binary.${NC}" >&2
        exit 1
    fi
fi

# 6. Tạo default configuration nếu chưa tồn tại
echo -e "${CYAN}[4/5] Checking configuration file...${NC}"
CONFIG_FILE="/etc/aegisnode/agent.yaml"
if [ ! -f "$CONFIG_FILE" ]; then
    cat <<'EOF' > "$CONFIG_FILE"
# AegisNode Agent Configuration File
version: "1.0"
server:
  host: "127.0.0.1"
  port: 8080
  unix_socket: "/run/aegisnode/aegisnode.sock"
storage:
  database_path: "/var/lib/aegisnode/aegisnode.db"
firewall:
  managed_table_ipv4: "aegis_filter"
  managed_table_ipv6: "aegis_filter"
  managed_nat_table: "aegis_nat"
  safe_apply_timeout_seconds: 30
EOF
    chmod 0640 "$CONFIG_FILE"
    echo -e "${GREEN}✓ Created default configuration at $CONFIG_FILE${NC}"
else
    echo -e "${YELLOW}Notice: Existing configuration file preserved at $CONFIG_FILE${NC}"
fi

# 7. Cài đặt và kích hoạt systemd service
echo -e "${CYAN}[5/5] Registering systemd service aegisnode-agent.service...${NC}"
SERVICE_SRC="packaging/systemd/aegisnode-agent.service"
if [ -f "$SERVICE_SRC" ]; then
    cp "$SERVICE_SRC" /etc/systemd/system/aegisnode-agent.service
else
    # Fallback tạo systemd unit file
    cat <<'EOF' > /etc/systemd/system/aegisnode-agent.service
[Unit]
Description=AegisNode Local Firewall Agent
After=network.target network-online.target
Wants=network-online.target

[Service]
Type=exec
ExecStart=/usr/local/bin/aegisnode local --config /etc/aegisnode/agent.yaml
Restart=on-failure
RestartSec=5s
RuntimeDirectory=aegisnode
StateDirectory=aegisnode
ConfigurationDirectory=aegisnode
User=root

[Install]
WantedBy=multi-user.target
EOF
fi

chmod 0644 /etc/systemd/system/aegisnode-agent.service
systemctl daemon-reload
systemctl enable aegisnode-agent.service
echo -e "${GREEN}✓ AegisNode systemd service registered and enabled.${NC}"

echo -e "${GREEN}==============================================================================${NC}"
echo -e "${GREEN}★ AegisNode installed successfully from GitHub Releases! Start with:${NC}"
echo -e "${CYAN}   sudo systemctl start aegisnode-agent${NC}"
echo -e "${GREEN}==============================================================================${NC}"

