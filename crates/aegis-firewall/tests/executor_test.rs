// Integration Test cho ExecutorClient & Execd Protocol - Phase 20 Privilege Separation

use aegis_firewall::ExecutorClient;
use aegis_rpc::{ExecRequest, ExecResponse};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

#[tokio::test]
async fn test_executor_client_and_server_protocol() {
    // 1. Tạo temp socket path cho test
    let dir = tempfile::tempdir().expect("Tạo tempdir thất bại");
    let socket_path = dir.path().join("execd_test.sock");
    let socket_str = socket_path.to_str().unwrap();

    let listener = UnixListener::bind(&socket_path).expect("Bind test execd socket thất bại");

    // 2. Mock Execd Server Handler
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let (reader, mut writer) = stream.into_split();
            let mut buf_reader = BufReader::new(reader);
            let mut line = String::new();

            if buf_reader.read_line(&mut line).await.is_ok() {
                if let Ok(req) = serde_json::from_str::<ExecRequest>(&line) {
                    let resp = match req {
                        ExecRequest::InspectFirewall => ExecResponse::FirewallReport {
                            ruleset_json: "{\"tables\":[\"inet aegis_filter\"]}".to_string(),
                        },
                        _ => ExecResponse::Success { details: "OK".to_string() },
                    };
                    let mut resp_str = serde_json::to_string(&resp).unwrap();
                    resp_str.push('\n');
                    let _ = writer.write_all(resp_str.as_bytes()).await;
                }
            }
        }
    });

    // 3. Khởi tạo ExecutorClient và gửi ExecRequest::InspectFirewall
    let client = ExecutorClient::new(Some(socket_str));
    let response = client
        .execute(ExecRequest::InspectFirewall)
        .await
        .expect("Gửi ExecRequest::InspectFirewall thất bại");

    // 4. Kiểm tra phản hồi trả về đúng dạng FirewallReport
    match response {
        ExecResponse::FirewallReport { ruleset_json } => {
            assert!(ruleset_json.contains("inet aegis_filter"));
        }
        _ => panic!("Phản hồi trả về phải là ExecResponse::FirewallReport"),
    }
}
