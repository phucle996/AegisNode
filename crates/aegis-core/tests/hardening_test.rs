// Integration Test cho Security Hardening & Payload Size Caps (Phase 27 Production Hardening)

use aegis_core::{MAX_API_PAYLOAD_SIZE_BYTES, SecurityHardening};

#[test]
fn test_payload_size_validation() {
    // 1. Payload trong ngưỡng an toàn (5MB) -> Hợp lệ
    let valid_payload_size = 5 * 1024 * 1024;
    assert!(SecurityHardening::validate_payload_size(valid_payload_size).is_ok());

    // 2. Payload vượt quá 10MB -> Phải bị từ chối với lỗi Validation
    let oversized_payload_size = MAX_API_PAYLOAD_SIZE_BYTES + 1;
    let result = SecurityHardening::validate_payload_size(oversized_payload_size);

    assert!(
        result.is_err(),
        "Payload vượt quá 10MB phải bị từ chối do vi phạm giới hạn bảo mật"
    );
}

#[test]
fn test_file_permissions_validation() {
    use tempfile::NamedTempFile;

    // 1. Tạo temp file thử nghiệm
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // Đặt quyền 0600 (Strict permissions)
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();

        // Kiểm tra quyền 0600 hợp lệ
        assert!(SecurityHardening::validate_file_permissions(path, 0o600).is_ok());

        // Kiểm tra sai quyền (Mong muốn 0777) -> Thất bại
        assert!(SecurityHardening::validate_file_permissions(path, 0o777).is_err());
    }
}
