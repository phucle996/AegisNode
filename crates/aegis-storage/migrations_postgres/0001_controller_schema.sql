-- AegisNode Controller Database Schema (PostgreSQL)
-- Phục vụ nền tảng quản trị tập trung Multi-Node Cloud Native HA Cluster

-- 1. Bảng Quản trị viên (Users)
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(64) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(32) NOT NULL DEFAULT 'admin',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 2. Bảng Token API Authentication
CREATE TABLE IF NOT EXISTS api_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(64) NOT NULL,
    token_hash VARCHAR(128) NOT NULL UNIQUE,
    scopes TEXT[] NOT NULL DEFAULT '{read,write}',
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 3. Bảng Danh sách Nodes đăng ký (Linux Agents)
CREATE TABLE IF NOT EXISTS nodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hostname VARCHAR(128) NOT NULL,
    ip_address VARCHAR(45) NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'OFFLINE',
    labels JSONB NOT NULL DEFAULT '{}'::jsonb,
    version VARCHAR(32) NOT NULL DEFAULT '0.1.0',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 4. Bảng Nhóm Node (Node Groups)
CREATE TABLE IF NOT EXISTS node_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(64) NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 5. Bảng Firewall Policy Tập Trung (Centralized Firewall Policies)
CREATE TABLE IF NOT EXISTS firewall_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(128) NOT NULL UNIQUE,
    description TEXT,
    version BIGINT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 6. Bảng Lịch sử Phiên bản Policy (Policy Versions)
CREATE TABLE IF NOT EXISTS policy_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    policy_id UUID NOT NULL REFERENCES firewall_policies(id) ON DELETE CASCADE,
    version BIGINT NOT NULL,
    content JSONB NOT NULL,
    sha256_hash VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT unique_policy_version UNIQUE(policy_id, version)
);

-- 7. Bảng Kế hoạch Thay đổi Policy trên Multi-Node (Change Plans)
CREATE TABLE IF NOT EXISTS change_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    policy_id UUID NOT NULL REFERENCES firewall_policies(id),
    target_group_id UUID REFERENCES node_groups(id),
    state VARCHAR(32) NOT NULL DEFAULT 'CREATED',
    version BIGINT NOT NULL DEFAULT 1, -- Optimistic locking chống race condition
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 8. Bảng Audit Logs kiểm vết thao tác quản trị
CREATE TABLE IF NOT EXISTS audit_logs (
    id BIGSERIAL PRIMARY KEY,
    event_type VARCHAR(64) NOT NULL,
    actor VARCHAR(64) NOT NULL,
    details JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 9. Bảng Enrollment Tokens (Phase 13)
CREATE TABLE IF NOT EXISTS enrollment_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_hash VARCHAR(128) NOT NULL UNIQUE,
    max_usages INT NOT NULL DEFAULT 1,
    current_usages INT NOT NULL DEFAULT 0,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 10. Bảng Agent Certificates (Phase 13 mTLS)
CREATE TABLE IF NOT EXISTS agent_certificates (
    serial_number VARCHAR(128) PRIMARY KEY,
    node_id UUID NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    machine_id VARCHAR(128) NOT NULL,
    hostname VARCHAR(128) NOT NULL,
    cert_pem TEXT NOT NULL,
    issued_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked BOOLEAN NOT NULL DEFAULT FALSE
);

-- Indexes tối ưu hiệu năng truy vấn
CREATE INDEX IF NOT EXISTS idx_nodes_status ON nodes(status);
CREATE INDEX IF NOT EXISTS idx_nodes_last_seen ON nodes(last_seen_at);
CREATE INDEX IF NOT EXISTS idx_policy_versions_policy ON policy_versions(policy_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created ON audit_logs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_enrollment_token_hash ON enrollment_tokens(token_hash);
CREATE INDEX IF NOT EXISTS idx_agent_certs_node ON agent_certificates(node_id);
