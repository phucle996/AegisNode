#!/usr/bin/env bash
# ==============================================================================
# AegisNode Automatic Agent Installation Script for Linux Hosts / KVM VMs
# Tự động cài đặt Agent, sinh CSR và nhận Client Certificate đã được ký từ Controller
# ==============================================================================

set -euo pipefail # Dừng script ngay nếu gặp lỗi hoặc biến chưa định nghĩa

# Định nghĩa màu sắc hiển thị trên màn hình Terminal
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # Clear color

REPO="phucle996/AegisNode"
VERSION="v1.1.3"
CONTROLLER_URL=""
ENROLLMENT_TOKEN=""

echo -e "${CYAN}[AegisNode Install] Safe Linux Firewall Agent Installation...${NC}"

# Parse các tham số dòng lệnh CLI (--version, --controller-url, --token)
while [[ $# -gt 0 ]]; do
  case $1 in
    -v|--version)
      VERSION="$2"
      shift 2
      ;;
    -c|--controller-url)
      CONTROLLER_URL="$2"
      shift 2
      ;;
    -t|--token)
      ENROLLMENT_TOKEN="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done

# 1. Kiểm tra quyền root tối cao
if [ "$(id -u)" -ne 0 ]; then
    echo -e "${RED}Error: Installing AegisNode requires root permissions. Please run with sudo.${NC}" >&2
    exit 1
fi

# 2. Kiểm tra binary nftables và tự động cài đặt nếu thiếu trên Linux Target
if ! command -v nft &> /dev/null; then
    echo -e "${YELLOW}Warning: 'nft' command not found. Attempting to install nftables automatically...${NC}"
    if command -v apt-get &> /dev/null; then
        # Cài đặt nftables cho hệ điều hành Debian / Ubuntu
        apt-get update -qq && apt-get install -y -qq nftables
    elif command -v apk &> /dev/null; then
        # Cài đặt nftables cho Alpine Linux
        apk add --no-cache nftables
    elif command -v dnf &> /dev/null; then
        # Cài đặt nftables cho RHEL / Fedora / CentOS
        dnf install -y -q nftables
    elif command -v yum &> /dev/null; then
        yum install -y -q nftables
    else
        echo -e "${RED}Error: 'nft' command not found and package manager is unsupported. Please install nftables manually.${NC}" >&2
        exit 1
    fi
fi

if ! command -v systemctl &> /dev/null; then
    echo -e "${RED}Error: 'systemctl' command not found. AegisNode requires systemd.${NC}" >&2
    exit 1
fi

# 3. Thiết lập cấu trúc thư mục hệ thống và thư mục PKI chứa chứng chỉ
echo -e "${CYAN}[1/6] Setting up system and PKI directories...${NC}"
mkdir -p /etc/aegisnode/pki
mkdir -p /var/lib/aegisnode
mkdir -p /run/aegisnode

# Phân quyền bảo mật tuyệt đối cho thư mục PKI (Chỉ root có quyền đọc/ghi 0700)
chmod 0700 /etc/aegisnode/pki
chmod 0750 /var/lib/aegisnode
chmod 0750 /run/aegisnode

# 4. Tạo user và group hệ thống aegisnode
echo -e "${CYAN}[2/6] Configuring system user and group 'aegisnode'...${NC}"
if ! getent group aegisnode >/dev/null; then
    groupadd --system aegisnode
fi

if ! getent passwd aegisnode >/dev/null; then
    useradd --system --gid aegisnode --no-create-home --shell /bin/false aegisnode
fi

# 5. Tải Binary Release từ GitHub Releases hoặc lấy từ bản build cục bộ
echo -e "${CYAN}[3/6] Downloading AegisNode binary release (${VERSION}) from GitHub...${NC}"
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

ARCH=$(uname -m)
TARBALL="aegisnode-${VERSION}-${ARCH}-unknown-linux-gnu.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${TARBALL}"

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

# 6. Đăng ký PKI Certificate với Controller (Request Client Cert đã được ký)
echo -e "${CYAN}[4/6] Bootstrapping Agent PKI & Certificate Exchange...${NC}"
AGENT_KEY="/etc/aegisnode/pki/agent.key"
AGENT_CERT="/etc/aegisnode/pki/agent.crt"
CA_CERT="/etc/aegisnode/pki/ca.crt"

if [ -n "$CONTROLLER_URL" ] && [ -n "$ENROLLMENT_TOKEN" ]; then
    echo -e "${CYAN}   Generating Agent ECDSA KeyPair & CSR locally...${NC}"
    HOSTNAME=$(hostname -f 2>/dev/null || hostname)
    MACHINE_ID=$(cat /etc/machine-id 2>/dev/null || cat /var/lib/dbus/machine-id 2>/dev/null || echo "mach_$(uname -n)")

    # Sinh ECDSA Private Key cho Agent
    openssl ecparam -name prime256v1 -genkey -noout -out "$AGENT_KEY"
    chmod 0600 "$AGENT_KEY"

    # Sinh CSR (Certificate Signing Request) cho Agent
    TMP_CSR=$(mktemp)
    openssl req -new -key "$AGENT_KEY" -out "$TMP_CSR" -subj "/O=AegisNode/CN=${HOSTNAME}"
    CSR_PEM=$(cat "$TMP_CSR")
    rm -f "$TMP_CSR"

    echo -e "${CYAN}   Sending Enrollment CSR Request to Controller (${CONTROLLER_URL})...${NC}"
    # Gửi REST API request lên Controller để nhận Client Cert đã ký
    PAYLOAD=$(jq -n \
      --arg token "$ENROLLMENT_TOKEN" \
      --arg host "$HOSTNAME" \
      --arg mach "$MACHINE_ID" \
      --arg ip "127.0.0.1" \
      --arg csr "$CSR_PEM" \
      '{enrollmentToken: $token, hostname: $host, machineId: $mach, ipAddress: $ip, csrPem: $csr}')

    RESPONSE=$(curl -sSL -X POST "${CONTROLLER_URL}/v1/enrollment/sign" \
      -H "Content-Type: application/json" \
      -d "$PAYLOAD" || echo "")

    if echo "$RESPONSE" | grep -q "certificatePem"; then
        echo "$RESPONSE" | jq -r '.certificatePem' > "$AGENT_CERT"
        echo "$RESPONSE" | jq -r '.caCertificatePem' > "$CA_CERT"
        chmod 0600 "$AGENT_CERT" "$CA_CERT"
        echo -e "${GREEN}✓ Successfully enrolled with Controller & received signed Client Certificate!${NC}"
    else
        echo -e "${YELLOW}Warning: Enrollment request failed or Controller unreached. Fallback to local self-signed cert...${NC}"
        openssl req -new -x509 -sha256 -key "$AGENT_KEY" -out "$AGENT_CERT" -days 365 -subj "/O=AegisNode/CN=${HOSTNAME}"
        cp "$AGENT_CERT" "$CA_CERT"
    fi
else
    echo -e "${YELLOW}Notice: No --controller-url or --token provided. Generating local self-signed key/cert for Agent...${NC}"
    if [ ! -f "$AGENT_KEY" ] || [ ! -f "$AGENT_CERT" ]; then
        HOSTNAME=$(hostname)
        openssl ecparam -name prime256v1 -genkey -noout -out "$AGENT_KEY"
        openssl req -new -x509 -sha256 -key "$AGENT_KEY" -out "$AGENT_CERT" -days 365 -subj "/O=AegisNode/CN=${HOSTNAME}"
        cp "$AGENT_CERT" "$CA_CERT"
        chmod 0600 "$AGENT_KEY" "$AGENT_CERT" "$CA_CERT"
        echo -e "${GREEN}✓ Created local self-signed Agent certificate.${NC}"
    fi
fi

# 7. Tạo file cấu hình agent.yaml với đường dẫn chứng chỉ mTLS
echo -e "${CYAN}[5/6] Writing configuration file /etc/aegisnode/agent.yaml...${NC}"
CONFIG_FILE="/etc/aegisnode/agent.yaml"
if [ ! -f "$CONFIG_FILE" ]; then
    cat <<EOF > "$CONFIG_FILE"
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
tls:
  enabled: true
  ca_cert_path: "$CA_CERT"
  client_cert_path: "$AGENT_CERT"
  client_key_path: "$AGENT_KEY"
EOF
    chmod 0640 "$CONFIG_FILE"
    echo -e "${GREEN}✓ Created default configuration at $CONFIG_FILE${NC}"
else
    echo -e "${YELLOW}Notice: Existing configuration file preserved at $CONFIG_FILE${NC}"
fi

# 8. Cài đặt và kích hoạt systemd service
echo -e "${CYAN}[6/6] Registering systemd service aegisnode-agent.service...${NC}"
SERVICE_SRC="packaging/systemd/aegisnode-agent.service"
if [ -f "$SERVICE_SRC" ]; then
    cp "$SERVICE_SRC" /etc/systemd/system/aegisnode-agent.service
else
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
echo -e "${GREEN}★ AegisNode Agent installed successfully! Start with:${NC}"
echo -e "${CYAN}   sudo systemctl start aegisnode-agent${NC}"
echo -e "${GREEN}==============================================================================${NC}"
