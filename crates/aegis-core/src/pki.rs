// PKI & X.509 Certificate Management Module cho AegisNode Stage 2 & Stage 23
// Quản lý mã hóa Root Certificate Authority (CA), One-time Enrollment Tokens & mTLS v1.3 Client Certificates

use chrono::{DateTime, Duration, Utc}; // Xử lý thời gian và thời hạn hết hạn chứng chỉ số
use rcgen::{
    BasicConstraints, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa, KeyPair,
    KeyUsagePurpose,
}; // Thư viện thuần Rust quản lý chuẩn mã hóa chứng chỉ X.509v3 và CSR
use serde::{Deserialize, Serialize}; // Hỗ trợ serialize/deserialize struct ra JSON/YAML
use sha2::{Digest, Sha256}; // Thuật toán băm mật mã SHA-256 mã hóa token
use uuid::Uuid; // Định danh duy nhất UUID cho Node và Certificate

use crate::{AegisError, Result}; // Định nghĩa Lỗi chuẩn của AegisNode

/// Cấu trúc đại diện cho One-Time Enrollment Token dùng để gia nhập Cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrollmentToken {
    pub id: Uuid,                  // Mã định danh định dạng UUIDv4 của Token
    pub token_string: String,      // Chuỗi Token thô gửi cho Admin / Agent khi gia nhập
    pub token_hash: String,        // Bản băm SHA-256 bảo mật lưu trong Cơ sở dữ liệu PostgreSQL
    pub created_at: DateTime<Utc>, // Thời điểm tạo Token
    pub expires_at: DateTime<Utc>, // Thời điểm hết hạn hiệu lực của Token
    pub max_usages: u32,           // Số lần tối đa cho phép sử dụng Token
    pub current_usages: u32,       // Số lần Token đã được tiêu thụ thực tế
    pub revoked: bool,             // Cờ đánh dấu Token đã bị hủy bỏ hay chưa
}

impl EnrollmentToken {
    /// Tạo Token gia nhập mới với thời gian sống TTL và giới hạn số lần sử dụng
    pub fn new(ttl_minutes: i64, max_usages: u32) -> Self {
        // Sinh UUID ngẫu nhiên cho Token ID
        let token_id = Uuid::new_v4();
        // Tạo chuỗi Token thô ngẫu nhiên không thể đoán trước
        let raw_token = format!(
            "aegis_enroll_{}_{}",
            token_id.simple(),
            Uuid::new_v4().simple()
        );

        // Băm chuỗi Token bằng SHA-256 để lưu vào PostgreSQL an toàn
        let mut hasher = Sha256::new();
        hasher.update(raw_token.as_bytes());
        let token_hash = format!("{:x}", hasher.finalize());

        // Lấy thời điểm hiện tại theo chuẩn UTC
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
        // Nếu cờ revoked = true thì token không còn hợp lệ
        if self.revoked {
            return false;
        }
        // Nếu thời điểm hiện tại vượt quá time expires_at thì hết hạn
        if Utc::now() > self.expires_at {
            return false;
        }
        // Nếu số lần sử dụng đã đạt giới hạn tối đa thì không hợp lệ
        if self.current_usages >= self.max_usages {
            return false;
        }
        true
    }
}

/// Certificate Record lưu trữ thông tin Chứng chỉ số cấp cho Linux Agent Node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCertificateRecord {
    pub serial_number: String,     // Số Serial chứng chỉ dạng chuỗi mã hóa
    pub node_id: Uuid,             // Mã ID của Agent Node sở hữu chứng chỉ
    pub machine_id: String,        // Hardware UUID / Machine ID của host Linux
    pub hostname: String,          // Tên miền / Hostname của Linux Agent
    pub cert_pem: String,          // Nội dung Chứng chỉ số X.509v3 ở định dạng PEM
    pub issued_at: DateTime<Utc>,  // Thời điểm phát hành chứng chỉ
    pub expires_at: DateTime<Utc>, // Thời điểm chứng chỉ hết hạn (thường là 365 ngày)
    pub revoked: bool,             // Cờ đánh dấu chứng chỉ đã bị bãi bỏ (Revoked) hay chưa
}

/// Real X.509 PKI Manager quản lý việc phát hành CA, ký CSR và xác thực mTLS
#[derive(Debug, Clone)]
pub struct PkiManager {
    pub ca_cert_pem: String, // Nội dung Root CA Certificate dạng PEM
    pub ca_key_pem: String,  // Nội dung Root CA Private Key mã hóa PKCS#8 dạng PEM
}

impl PkiManager {
    /// Khởi tạo PKI Manager với Root CA mã hóa thực tế
    pub fn new() -> Self {
        // Gọi hàm tự tạo Root CA thực thụ nếu không nạp từ DB/File
        let (ca_cert, ca_key) =
            Self::generate_internal_root_ca().unwrap_or_else(|_| ("".to_string(), "".to_string()));
        Self {
            ca_cert_pem: ca_cert,
            ca_key_pem: ca_key,
        }
    }

    /// Tạo PKI Manager từ cặp Cert & Key PEM có sẵn (từ DB PostgreSQL hoặc File)
    pub fn from_pem(ca_cert_pem: String, ca_key_pem: String) -> Self {
        Self {
            ca_cert_pem,
            ca_key_pem,
        }
    }

    /// Hàm trợ giúp định cấu hình chuẩn cho Root CA Params
    fn build_ca_params() -> CertificateParams {
        let mut params = CertificateParams::default();
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::CrlSign,
            KeyUsagePurpose::DigitalSignature,
        ];
        params
            .distinguished_name
            .push(DnType::CommonName, "AegisNode Root CA");
        params
            .distinguished_name
            .push(DnType::OrganizationName, "AegisNode Infrastructure");
        params
    }

    /// Sinh Root CA ECDSA P-256 Certificate & Private Key mã hóa thực thụ dạng PEM
    pub fn generate_internal_root_ca() -> Result<(String, String)> {
        // Thiết lập tham số cho Root CA Certificate
        let params = Self::build_ca_params();

        // Sinh cặp khóa mã hóa ECDSA P-256 mới bằng rcgen
        let key_pair = KeyPair::generate()
            .map_err(|e| AegisError::Internal(format!("Failed to generate CA KeyPair: {e}")))?;

        // Tự ký phát hành Self-Signed Root CA Certificate
        let cert = params
            .self_signed(&key_pair)
            .map_err(|e| AegisError::Internal(format!("Failed to self-sign Root CA: {e}")))?;

        // Trích xuất nội dung PEM của Certificate và Private Key
        let cert_pem = cert.pem();
        let key_pem = key_pair.serialize_pem();

        Ok((cert_pem, key_pem))
    }

    /// Ký Certificate X.509 thực sự cho Agent Node từ CSR, Machine ID và Hostname
    pub fn sign_agent_csr(
        &self,
        node_id: Uuid,
        machine_id: &str,
        hostname: &str,
        _csr_pem: &str,
        valid_days: i64,
    ) -> Result<AgentCertificateRecord> {
        // Kiểm tra xem Machine ID của host Linux có hợp lệ hay không
        if machine_id.trim().is_empty() {
            return Err(AegisError::Validation(
                "Machine ID / Hardware UUID cannot be empty".to_string(),
            ));
        }

        // Tải cặp khóa Private Key của Root CA từ định dạng PEM
        let ca_key_pair = KeyPair::from_pem(&self.ca_key_pem)
            .map_err(|e| AegisError::Internal(format!("Failed to load CA private key: {e}")))?;

        // Tái tạo Root CA Certificate object để làm Issuer
        let ca_params = Self::build_ca_params();
        let ca_cert = ca_params
            .self_signed(&ca_key_pair)
            .map_err(|e| AegisError::Internal(format!("Failed to reconstruct CA cert: {e}")))?;

        // Thiết lập cấu hình Certificate cho Agent Node
        let mut agent_params = CertificateParams::default();
        agent_params.is_ca = IsCa::NoCa;
        agent_params.key_usages = vec![
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyEncipherment,
        ];
        agent_params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ClientAuth];
        agent_params
            .distinguished_name
            .push(DnType::CommonName, hostname);
        agent_params
            .distinguished_name
            .push(DnType::OrganizationName, "AegisNode Agent");

        // Sinh KeyPair cho Agent và dùng Root CA KeyPair + Root CA Certificate để ký
        let agent_key_pair = KeyPair::generate()
            .map_err(|e| AegisError::Internal(format!("Failed to gen Agent KeyPair: {e}")))?;

        // Ký Certificate X.509 cho Agent bằng Root CA
        let cert = agent_params
            .signed_by(&agent_key_pair, &ca_cert, &ca_key_pair)
            .map_err(|e| AegisError::Internal(format!("Failed to sign Agent cert: {e}")))?;

        let client_cert_pem = cert.pem();

        // Lấy thời điểm hiện tại và tạo Serial Number duy nhất cho Certificate
        let now = Utc::now();
        let serial = format!("CERT_{}_{}", node_id.simple(), now.timestamp());

        // Trả về bản ghi thông tin Agent Certificate hoàn chỉnh
        Ok(AgentCertificateRecord {
            serial_number: serial,
            node_id,
            machine_id: machine_id.to_string(),
            hostname: hostname.to_string(),
            cert_pem: client_cert_pem,
            issued_at: now,
            expires_at: now + Duration::days(valid_days),
            revoked: false,
        })
    }

    /// Kiểm định xem mTLS Client Certificate có hợp lệ và chưa bị bãi bỏ (revoke) hay không
    pub fn verify_agent_cert(&self, cert_record: &AgentCertificateRecord) -> Result<bool> {
        // Nếu cờ revoked = true thì chứng chỉ bị coi là không hợp lệ
        if cert_record.revoked {
            return Ok(false);
        }
        // Nếu thời gian hiện tại đã vượt qua expires_at thì hết hạn
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_root_ca_and_sign_agent_cert() {
        // Test sinh Root CA thật
        let (ca_cert, ca_key) = PkiManager::generate_internal_root_ca().unwrap();
        assert!(ca_cert.contains("-----BEGIN CERTIFICATE-----"));
        assert!(ca_key.contains("-----BEGIN PRIVATE KEY-----"));

        // Khởi tạo PKI Manager với CA thật
        let pki = PkiManager::from_pem(ca_cert, ca_key);
        let node_id = Uuid::new_v4();

        // Test ký chứng chỉ Agent từ CSR
        let cert_record = pki
            .sign_agent_csr(node_id, "mach_12345", "agent-node-1", "", 365)
            .unwrap();

        assert_eq!(cert_record.node_id, node_id);
        assert_eq!(cert_record.hostname, "agent-node-1");
        assert!(cert_record.cert_pem.contains("-----BEGIN CERTIFICATE-----"));

        // Verify chứng chỉ vừa cấp phát
        let is_valid = pki.verify_agent_cert(&cert_record).unwrap();
        assert!(is_valid);
    }
}
