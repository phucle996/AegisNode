#!/usr/bin/env bash
# ==============================================================================
# AegisNode Controller Automatic Installation & PKI Bootstrapping Script
# Khởi tạo AegisNode Controller Server, thiết lập thư mục PKI an toàn và sinh Bộ Cert Nguồn
# ==============================================================================

set -euo pipefail # Thoát ngay nếu có lỗi, chưa khai báo biến hoặc pipeline lỗi

# Định nghĩa màu sắc hiển thị log trên Terminal
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # Clear color

echo -e "${CYAN}[AegisNode Controller Install] Starting installation and PKI bootstrapping...${NC}"

# 1. Kiểm tra quyền root tối cao
if [ "$(id -u)" -ne 0 ]; then
    echo -e "${RED}Error: Installing AegisNode Controller requires root permissions. Please run with sudo.${NC}" >&2
    exit 1
fi

# 2. Kiểm tra systemd có sẵn trên host
if ! command -v systemctl &> /dev/null; then
    echo -e "${RED}Error: 'systemctl' command not found. AegisNode Controller requires systemd.${NC}" >&2
    exit 1
fi

# 3. Tạo cấu trúc thư mục hệ thống và PKI với phân quyền bảo mật
echo -e "${CYAN}[1/5] Setting up system and PKI directories...${NC}"
mkdir -p /etc/aegisnode/pki    # Thư mục lưu trữ bộ chứng chỉ số PKI
mkdir -p /var/lib/aegisnode    # Thư mục dữ liệu runtime
mkdir -p /run/aegisnode        # Thư mục chứa Unix Domain Socket

# Thiết lập phân quyền chặt chẽ cho thư mục PKI (Chỉ owner root/aegisnode truy cập 0700)
chmod 0700 /etc/aegisnode/pki
chmod 0750 /var/lib/aegisnode
chmod 0750 /run/aegisnode

# 4. Kiểm tra hoặc tạo user/group hệ thống aegisnode
echo -e "${CYAN}[2/5] Configuring system user and group 'aegisnode'...${NC}"
if ! getent group aegisnode >/dev/null; then
    groupadd --system aegisnode
fi

if ! getent passwd aegisnode >/dev/null; then
    useradd --system --gid aegisnode --no-create-home --shell /bin/false aegisnode
fi

# Gán quyền sở hữu thư mục PKI cho aegisnode:aegisnode
chown -R aegisnode:aegisnode /etc/aegisnode/pki

# 5. Cài đặt binary aegisnode vào /usr/local/bin/
echo -e "${CYAN}[3/5] Installing AegisNode Controller binary...${NC}"
LOCAL_BIN="target/release/aegisnode"
if [ -f "$LOCAL_BIN" ]; then
    cp "$LOCAL_BIN" /usr/local/bin/aegisnode
    chmod 0755 /usr/local/bin/aegisnode
    echo -e "${GREEN}✓ Installed binary from $LOCAL_BIN to /usr/local/bin/aegisnode${NC}"
elif [ -f "/usr/local/bin/aegisnode" ]; then
    echo -e "${GREEN}✓ Using existing binary at /usr/local/bin/aegisnode${NC}"
else
    echo -e "${RED}Error: Binary /usr/local/bin/aegisnode not found. Please build release first with 'cargo build --release'.${NC}" >&2
    exit 1
fi

# 6. Khởi tạo Bộ Cert Nguồn (Root CA + Controller Server TLS Cert/Key) nếu chưa tồn tại
echo -e "${CYAN}[4/5] Bootstrapping Source PKI Certificates (Root CA & Server Cert)...${NC}"
CA_CERT="/etc/aegisnode/pki/ca.crt"
CA_KEY="/etc/aegisnode/pki/ca.key"
SERVER_CERT="/etc/aegisnode/pki/server.crt"
SERVER_KEY="/etc/aegisnode/pki/server.key"

if [ ! -f "$CA_CERT" ] || [ ! -f "$CA_KEY" ]; then
    echo -e "${CYAN}   Generating OpenSSL X.509 Root CA for AegisNode...${NC}"
    # Sinh Root CA Private Key (ECDSA prime256v1)
    openssl ecparam -name prime256v1 -genkey -noout -out "$CA_KEY"
    # Sinh Self-Signed Root CA Certificate
    openssl req -new -x509 -sha256 -key "$CA_KEY" -out "$CA_CERT" -days 3650 \
        -subj "/O=AegisNode/CN=AegisNode Root CA"
    
    chmod 0600 "$CA_KEY"
    chmod 0644 "$CA_CERT"
    echo -e "${GREEN}✓ Generated Root CA Certificate and Key${NC}"
else
    echo -e "${YELLOW}Notice: Root CA Certificate already exists at $CA_CERT${NC}"
fi

if [ ! -f "$SERVER_CERT" ] || [ ! -f "$SERVER_KEY" ]; then
    echo -e "${CYAN}   Generating Controller Server mTLS Certificate...${NC}"
    # Sinh Private Key cho Controller Server
    openssl ecparam -name prime256v1 -genkey -noout -out "$SERVER_KEY"
    # Sinh CSR cho Controller Server
    TMP_CSR=$(mktemp)
    openssl req -new -key "$SERVER_KEY" -out "$TMP_CSR" \
        -subj "/O=AegisNode/CN=controller.aegisnode.local"
    # Dùng Root CA ký Server Certificate
    openssl x509 -req -in "$TMP_CSR" -CA "$CA_CERT" -CAkey "$CA_KEY" -CAcreateserial \
        -out "$SERVER_CERT" -days 365 -sha256
    rm -f "$TMP_CSR"

    chmod 0600 "$SERVER_KEY"
    chmod 0644 "$SERVER_CERT"
    echo -e "${GREEN}✓ Generated Controller Server Certificate and Key${NC}"
else
    echo -e "${YELLOW}Notice: Controller Server Certificate already exists at $SERVER_CERT${NC}"
fi

# Đảm bảo phân quyền file cert/key đúng cho user aegisnode
chown -R aegisnode:aegisnode /etc/aegisnode/pki

# 7. Tạo file cấu hình controller.yaml nếu chưa tồn tại
CONFIG_FILE="/etc/aegisnode/controller.yaml"
if [ ! -f "$CONFIG_FILE" ]; then
    cat <<EOF > "$CONFIG_FILE"
# AegisNode Controller Server Configuration File
version: "1.0"
server:
  host: "0.0.0.0"
  port: 8080
  auth_secret: "aegis_secure_controller_secret_change_me"
  session_ttl_seconds: 86400
database:
  url: "postgres://postgres:postgres@localhost:5432/aegisnode"
  max_connections: 20
  connect_timeout_seconds: 10
tls:
  enabled: true
  ca_cert_path: "$CA_CERT"
  ca_key_path: "$CA_KEY"
  server_cert_path: "$SERVER_CERT"
  server_key_path: "$SERVER_KEY"
EOF
    chmod 0640 "$CONFIG_FILE"
    chown aegisnode:aegisnode "$CONFIG_FILE"
    echo -e "${GREEN}✓ Created Controller configuration file at $CONFIG_FILE${NC}"
fi

# 8. Cài đặt Systemd unit service cho Controller
echo -e "${CYAN}[5/5] Registering systemd service aegisnode-controller.service...${NC}"
cat <<EOF > /etc/systemd/system/aegisnode-controller.service
[Unit]
Description=AegisNode Central Controller Server
After=network.target network-online.target postgresql.service
Wants=network-online.target

[Service]
Type=exec
ExecStart=/usr/local/bin/aegisnode server --config /etc/aegisnode/controller.yaml
Restart=on-failure
RestartSec=5s
RuntimeDirectory=aegisnode
StateDirectory=aegisnode
ConfigurationDirectory=aegisnode
User=aegisnode
Group=aegisnode

[Install]
WantedBy=multi-user.target
EOF

chmod 0644 /etc/systemd/system/aegisnode-controller.service
systemctl daemon-reload
systemctl enable aegisnode-controller.service
echo -e "${GREEN}✓ AegisNode Controller systemd service registered and enabled.${NC}"

echo -e "${GREEN}==============================================================================${NC}"
echo -e "${GREEN}★ AegisNode Controller installed successfully! Start service with:${NC}"
echo -e "${CYAN}   sudo systemctl start aegisnode-controller${NC}"
echo -e "${GREEN}==============================================================================${NC}"
