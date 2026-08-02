// Integration Test cho Module Peer Credential (SO_PEERCRED) - Phase 20 Privilege Separation

use aegis_core::{extract_peer_credentials, validate_peer_uid};
use tokio::net::{UnixListener, UnixStream};

#[tokio::test]
async fn test_peer_credential_extraction_and_validation() {
    // 1. Tạo thư mục tạm và Unix Listener thử nghiệm
    let dir = tempfile::tempdir().expect("Không thể tạo tempdir");
    let socket_path = dir.path().join("test_cred.sock");

    let listener = UnixListener::bind(&socket_path).expect("Không thể bind test socket");

    // 2. Kết nối từ client
    let client_task = tokio::spawn(async move {
        UnixStream::connect(socket_path)
            .await
            .expect("Client kết nối thất bại")
    });

    let (server_stream, _) = listener.accept().await.expect("Accept thất bại");
    let _client_stream = client_task.await.expect("Client task error");

    // 3. Kiểm tra extract_peer_credentials thành công
    let creds = extract_peer_credentials(&server_stream).expect("Trích xuất SO_PEERCRED thất bại");
    assert!(creds.pid.is_some(), "PID của peer phải tồn tại");

    // 4. Kiểm tra validate_peer_uid với UID hiện tại của user đang chạy test
    let current_uid = creds.uid;
    let validated =
        validate_peer_uid(&server_stream, &[current_uid]).expect("Xác thực UID hợp lệ phải pass");
    assert_eq!(validated.uid, current_uid);

    // 5. Kiểm tra validate_peer_uid bị từ chối khi UID không trùng khớp
    let invalid_uid = current_uid + 9999;
    let err = validate_peer_uid(&server_stream, &[invalid_uid]);
    assert!(
        err.is_err(),
        "UID không thuộc allowlist phải bị từ chối với PermissionDenied"
    );
}
