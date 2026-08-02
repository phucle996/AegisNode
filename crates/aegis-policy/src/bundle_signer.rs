//! Ed25519 Policy Bundle Signer & Verifier (Phase 22 Cryptographic Integrity & Anti-Replay)
//! Cung cấp các phương thức ký số Ed25519 cho Controller và xác thực Chữ ký, Checksum, Target Node ID và Replay Protection cho Agent.

use aegis_core::AegisError;
use aegis_models::bundle::SignedPolicyBundle;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;

/// Đối tượng Ký số Ed25519 phía Controller (BundleSigner)
pub struct BundleSigner {
    signing_key: SigningKey,
}

impl BundleSigner {
    /// Sinh ngẫu nhiên một Cặp chìa khóa Ký số Ed25519 mới
    pub fn generate_random() -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        Self { signing_key }
    }

    /// Khởi tạo BundleSigner từ chuỗi 32-byte secret key hex
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(bytes);
        Self { signing_key }
    }

    /// Trích xuất VerifyingKey (Chìa khóa công khai) tương ứng
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// Thực hiện Tính toán Checksum và Ký số Ed25519 cho SignedPolicyBundle
    pub fn sign_bundle(&self, bundle: &mut SignedPolicyBundle) -> Result<(), AegisError> {
        // 1. Tự động tính toán lại mã SHA-256 Payload Checksum
        bundle.payload_checksum = bundle.compute_payload_checksum();

        // 2. Lấy dữ liệu dạng byte chuẩn hóa cần ký số
        let bytes_to_sign = bundle.signing_bytes();

        // 3. Thực hiện tạo chữ ký Ed25519 (64-byte signature)
        let signature: Signature = self.signing_key.sign(&bytes_to_sign);

        // 4. Lưu chữ ký dạng chuỗi Hexadecimal vào Bundle
        bundle.signature_hex = hex::encode(signature.to_bytes());

        Ok(())
    }
}

/// Đối tượng Xác thực Chữ ký số Ed25519 & Anti-Replay phía Agent (BundleVerifier)
pub struct BundleVerifier {
    trusted_public_keys: Vec<VerifyingKey>,
}

impl BundleVerifier {
    /// Khởi tạo Verifier với danh sách các Public Key tin cậy (Key Ring)
    pub fn new(trusted_public_keys: Vec<VerifyingKey>) -> Self {
        Self { trusted_public_keys }
    }

    /// Xác thực toàn bộ tính hợp lệ của SignedPolicyBundle:
    /// - Chữ ký số Ed25519 hợp lệ từ Controller công nhận
    /// - Checksum SHA-256 không bị sửa đổi 1 byte nào
    /// - Target Node ID trùng khớp với Node hiện tại
    /// - Sequence Number tăng đơn điệu (Chống Replay Attack)
    pub fn verify_bundle(
        &self,
        bundle: &SignedPolicyBundle,
        expected_node_id: &str,
        last_applied_sequence: u64,
    ) -> Result<(), AegisError> {
        // 1. Kiểm tra khóa Target Node ID
        if bundle.target_node_id != expected_node_id {
            return Err(AegisError::Validation(format!(
                "Từ chối Bundle: Node ID đích không khớp (Nhận: {}, Kỳ vọng: {})",
                bundle.target_node_id, expected_node_id
            )));
        }

        // 2. Kiểm tra Replay Attack bằng Sequence Number
        if bundle.sequence_number <= last_applied_sequence {
            return Err(AegisError::Conflict(format!(
                "Phát hiện tấn công Replay Attack: Sequence number {} nhỏ hơn hoặc bằng sequence cũ {}",
                bundle.sequence_number, last_applied_sequence
            )));
        }

        // 3. Kiểm tra tính toàn vẹn của SHA-256 Checksum
        let computed_checksum = bundle.compute_payload_checksum();
        if bundle.payload_checksum != computed_checksum {
            return Err(AegisError::Validation(format!(
                "Phát hiện dữ liệu Bundle bị sửa đổi (Tampered): Checksum không khớp (Nhận: {}, Tính toán: {})",
                bundle.payload_checksum, computed_checksum
            )));
        }

        // 4. Decode chữ ký số Ed25519 Hex thành 64-byte array
        let sig_bytes = hex::decode(&bundle.signature_hex).map_err(|e| {
            AegisError::Validation(format!("Chữ ký số Hex không đúng định dạng: {e}"))
        })?;

        if sig_bytes.len() != 64 {
            return Err(AegisError::Validation(
                "Độ dài chữ ký số Ed25519 không đúng (yêu cầu 64 bytes)".to_string(),
            ));
        }

        let mut sig_array = [0u8; 64];
        sig_array.copy_from_slice(&sig_bytes);
        let signature = Signature::from_bytes(&sig_array);

        // 5. Kiểm tra chữ ký Ed25519 với danh sách Trusted Public Keys
        let bytes_to_verify = bundle.signing_bytes();
        let mut is_valid = false;

        for pub_key in &self.trusted_public_keys {
            if pub_key.verify(&bytes_to_verify, &signature).is_ok() {
                is_valid = true;
                break;
            }
        }

        if !is_valid {
            return Err(AegisError::Permission(
                "Chữ ký số Ed25519 không hợp lệ hoặc không thuộc danh sách Public Key tin cậy".to_string(),
            ));
        }

        Ok(())
    }
}
