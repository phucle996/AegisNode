-- Migration 0002: Bảng lưu trữ Root Certificate Authority (CA) dùng cho Controller High Availability (HA)
-- Đảm bảo khi chạy nhiều Replicas Controller trong Cloud Native Cluster, tất cả Replicas sử dụng chung 1 Root CA duy nhất.

CREATE TABLE IF NOT EXISTS cluster_pki_ca (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- Định danh UUIDv4 của bản ghi Root CA
    ca_cert_pem TEXT NOT NULL,                     -- Nội dung X.509 Root CA Certificate ở định dạng PEM
    ca_key_pem TEXT NOT NULL,                      -- Nội dung Root CA Private Key ở định dạng PEM (được phân quyền chặt chẽ)
    active BOOLEAN NOT NULL DEFAULT TRUE,          -- Trạng thái cờ đánh dấu Root CA đang hoạt động
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP -- Thời điểm khởi tạo Root CA trong Cluster
);

-- Index tra cứu nhanh Root CA active duy nhất cho Controller Replicas
CREATE INDEX IF NOT EXISTS idx_cluster_pki_ca_active ON cluster_pki_ca(active) WHERE active = TRUE;
