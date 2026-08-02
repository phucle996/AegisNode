// Integration Test cho Multi-Endpoint Agent Failover Client (Phase 23 Controller HA)

use aegis_core::AegisError;
use aegis_rpc::FailoverRpcClient;

#[tokio::test]
async fn test_failover_client_endpoint_rotation() {
    let endpoints = vec![
        "https://c1.prod.internal:8443".to_string(),
        "https://c2.prod.internal:8443".to_string(),
        "https://c3.prod.internal:8443".to_string(),
    ];

    // 1. Khởi tạo FailoverRpcClient với 3 endpoints
    let client = FailoverRpcClient::new(endpoints.clone()).expect("Khởi tạo failover client thất bại");

    // Endpoint ban đầu là c1
    assert_eq!(client.active_endpoint(), endpoints[0]);

    // 2. Thất bại kết nối c1 -> Tự động rotate sang c2
    let next_1 = client.rotate_to_next_endpoint();
    assert_eq!(next_1, endpoints[1]);
    assert_eq!(client.active_endpoint(), endpoints[1]);

    // 3. Thất bại kết nối c2 -> Tự động rotate sang c3
    let next_2 = client.rotate_to_next_endpoint();
    assert_eq!(next_2, endpoints[2]);
    assert_eq!(client.active_endpoint(), endpoints[2]);

    // 4. Thất bại kết nối c3 -> Tự động quay lại c1 (Round-robin)
    let next_3 = client.rotate_to_next_endpoint();
    assert_eq!(next_3, endpoints[0]);
}

#[tokio::test]
async fn test_execute_with_failover_success_on_fallback() {
    let endpoints = vec![
        "https://c1-failing.prod.internal:8443".to_string(),
        "https://c2-working.prod.internal:8443".to_string(),
    ];

    let client = FailoverRpcClient::new(endpoints).unwrap();

    // Giả lập mock RPC request: c1 báo lỗi timeout, c2 trả về thành công
    let result = client
        .execute_with_failover(|ep| async move {
            if ep.contains("c1-failing") {
                Err(AegisError::Timeout("c1 offline".to_string()))
            } else {
                Ok("CONNECTED_TO_C2".to_string())
            }
        })
        .await;

    assert!(result.is_ok(), "Failover client phải tự chuyển vùng sang C2 và thành công");
    assert_eq!(result.unwrap(), "CONNECTED_TO_C2");
}
