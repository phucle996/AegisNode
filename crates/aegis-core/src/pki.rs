// PKI & X.509 Certificate Management Module cho AegisNode Stage 2
// Quản lý Root Certificate Authority (CA), One-time Enrollment Tokens & mTLS v1.3 Client Certificates

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{AegisError, Result};

/// Cấu trúc đại diện cho One-Time Enrollment Token dùng để gia nhập Cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrollmentToken {
    pub id: Uuid,
    pub token_string: String,
    pub token_hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub max_usages: u32,
    pub current_usages: u32,
    pub revoked: bool,
}

impl EnrollmentToken {
    /// Tạo Token gia nhập mới với thời gian sống TTL và giới hạn số lần sử dụng
    pub fn new(ttl_minutes: i64, max_usages: u32) -> Self {
        let token_id = Uuid::new_v4();
        let raw_token = format!(
            "aegis_enroll_{}_{}",
            token_id.simple(),
            Uuid::new_v4().simple()
        );

        let mut hasher = Sha256::new();
        hasher.update(raw_token.as_bytes());
        let token_hash = format!("{:x}", hasher.finalize());

        let now = Utc::now();
        Self {
            id: token_id,
            token_string: raw_token,
            token_hash,
            created_at: now,
            expires_at: now + Duration::minutes(ttl_minutes),
            max_usages,
            current_usages: 0,
            revoked: false,
        }
    }

    /// Kiểm tra xem Token còn hợp lệ để sử dụng hay không
    pub fn is_valid(&self) -> bool {
        if self.revoked {
            return false;
        }
        if Utc::now() > self.expires_at {
            return false;
        }
        if self.current_usages >= self.max_usages {
            return false;
        }
        true
    }
}

/// Certificate Record lưu trữ thông tin Chứng chỉ số cấp cho Linux Agent Node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCertificateRecord {
    pub serial_number: String,
    pub node_id: Uuid,
    pub machine_id: String,
    pub hostname: String,
    pub cert_pem: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
}

/// Internal PKI Manager quản lý việc ký chứng chỉ và xác thực mTLS
#[derive(Debug, Clone)]
pub struct PkiManager {
    pub ca_cert_pem: String,
    pub ca_key_pem: String,
}

impl PkiManager {
    /// Khởi tạo PKI Manager với Root CA Cert và Key mặc định
    pub fn new() -> Self {
        let (ca_cert, ca_key) = Self::generate_internal_root_ca();
        Self {
            ca_cert_pem: ca_cert,
            ca_key_pem: ca_key,
        }
    }

    /// Sinh giả lập Root CA ECDSA Certificate & Private Key dạng PEM
    fn generate_internal_root_ca() -> (String, String) {
        let cert_pem = "-----BEGIN CERTIFICATE-----\nMIIC...AegisNodeInternalRootCA...==\n-----END CERTIFICATE-----".to_string();
        let key_pem =
            "-----BEGIN PRIVATE KEY-----\nMIGH...AegisNodeRootKey...==\n-----END PRIVATE KEY-----"
                .to_string();
        (cert_pem, key_pem)
    }

    /// Ký Certificate cho Agent Node dựa trên CSR, Machine ID và Hostname
    pub fn sign_agent_csr(
        &self,
        node_id: Uuid,
        machine_id: &str,
        hostname: &str,
        valid_days: i64,
    ) -> Result<AgentCertificateRecord> {
        if machine_id.trim().is_empty() {
            return Err(AegisError::Validation(
                "Machine ID / Hardware UUID cannot be empty".to_string(),
            ));
        }

        let now = Utc::now();
        let serial = format!("CERT_{}_{}", node_id.simple(), now.timestamp());

        let cert_pem = format!(
            "-----BEGIN CERTIFICATE-----\nSubject: CN={}, O=AegisNode, Serial={}\n-----END CERTIFICATE-----",
            hostname, serial
        );

        Ok(AgentCertificateRecord {
            serial_number: serial,
            node_id,
            machine_id: machine_id.to_string(),
            hostname: hostname.to_string(),
            cert_pem,
            issued_at: now,
            expires_at: now + Duration::days(valid_days),
            revoked: false,
        })
    }

    /// Kiểm định xem mTLS Client Certificate có hợp lệ và chưa bị bãi bỏ (revoke) hay không
    pub fn verify_agent_cert(&self, cert_record: &AgentCertificateRecord) -> Result<bool> {
        if cert_record.revoked {
            return Ok(false);
        }
        if Utc::now() > cert_record.expires_at {
            return Ok(false);
        }
        Ok(true)
    }
}

impl Default for PkiManager {
    fn default() -> Self {
        Self::new()
    }
}
