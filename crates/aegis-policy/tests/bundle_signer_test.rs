// Integration Test cho Ed25519 Bundle Signer & Verifier (Phase 22 Cryptographic Integrity & Replay Protection)

use aegis_core::PolicyId;
use aegis_models::bundle::SignedPolicyBundle;
use aegis_models::firewall::{
    FirewallAction, FirewallDefaults, FirewallPolicy, PolicyMetadata, SUPPORTED_API_VERSION,
    SUPPORTED_FIREWALL_KIND,
};
use aegis_policy::{BundleSigner, BundleVerifier};
use chrono::Utc;
use std::collections::HashMap;

fn create_sample_bundle() -> SignedPolicyBundle {
    SignedPolicyBundle {
        bundle_id: "bundle-uuid-001".to_string(),
        target_node_id: "node-worker-01".to_string(),
        policy_version: "v1.4.2".to_string(),
        sequence_number: 10,
        issued_at: "2026-08-02T12:00:00Z".to_string(),
        expires_at: "2026-08-02T12:15:00Z".to_string(),
        controller_id: "controller-01".to_string(),
        payload_checksum: "".to_string(),
        firewall_policy: Some(FirewallPolicy {
            api_version: SUPPORTED_API_VERSION.to_string(),
            kind: SUPPORTED_FIREWALL_KIND.to_string(),
            metadata: PolicyMetadata {
                id: PolicyId::new_v4(),
                name: "Web Server Strict Policy".to_string(),
                labels: HashMap::new(),
                version: 1,
                created_at: Utc::now(),
            },
            defaults: FirewallDefaults {
                input: FirewallAction::Drop,
                output: FirewallAction::Accept,
                forward: FirewallAction::Drop,
            },
            rules: vec![],
        }),
        network_profile: None,
        signature_hex: "".to_string(),
    }
}

#[test]
fn test_valid_bundle_signing_and_verification() {
    // 1. Sinh ngẫu nhiên Key Pair Ed25519 cho Signer
    let signer = BundleSigner::generate_random();
    let verifier = BundleVerifier::new(vec![signer.verifying_key()]);

    // 2. Ký số Bundle
    let mut bundle = create_sample_bundle();
    signer
        .sign_bundle(&mut bundle)
        .expect("Ký số Bundle thất bại");

    assert!(
        !bundle.signature_hex.is_empty(),
        "Signature Hex phải được tạo"
    );
    assert!(
        !bundle.payload_checksum.is_empty(),
        "Payload Checksum SHA-256 phải được tạo"
    );

    // 3. Verifier xác thực thành công đối với Bundle hợp lệ
    let last_sequence = 9;
    let result = verifier.verify_bundle(&bundle, "node-worker-01", last_sequence);
    assert!(result.is_ok(), "Xác thực Bundle hợp lệ phải thành công");
}

#[test]
fn test_tampered_bundle_rejection() {
    let signer = BundleSigner::generate_random();
    let verifier = BundleVerifier::new(vec![signer.verifying_key()]);

    let mut bundle = create_sample_bundle();
    signer.sign_bundle(&mut bundle).unwrap();

    // 1. Sửa đổi 1 byte trong Target Node ID -> Phải bị từ chối
    let mut tampered_node_bundle = bundle.clone();
    tampered_node_bundle.target_node_id = "node-worker-99".to_string();
    assert!(
        verifier
            .verify_bundle(&tampered_node_bundle, "node-worker-01", 9)
            .is_err()
    );

    // 2. Sửa đổi 1 byte trong Payload Checksum -> Phải bị từ chối do Checksum mismatch
    let mut tampered_checksum_bundle = bundle.clone();
    tampered_checksum_bundle.payload_checksum =
        "0000000000000000000000000000000000000000000000000000000000000000".to_string();
    assert!(
        verifier
            .verify_bundle(&tampered_checksum_bundle, "node-worker-01", 9)
            .is_err()
    );
}

#[test]
fn test_replay_attack_rejection() {
    let signer = BundleSigner::generate_random();
    let verifier = BundleVerifier::new(vec![signer.verifying_key()]);

    let mut bundle = create_sample_bundle(); // sequence_number = 10
    signer.sign_bundle(&mut bundle).unwrap();

    // 1. Gửi lại Bundle cũ có sequence_number <= last_applied_sequence (10 <= 10) -> Tấn công Replay Attack
    let last_applied_sequence = 10;
    let result = verifier.verify_bundle(&bundle, "node-worker-01", last_applied_sequence);

    assert!(
        result.is_err(),
        "Phát hiện Replay Attack: Bundle có sequence <= sequence đã apply trước đó phải bị từ chối"
    );
}
