//! Executor Client IPC (Privilege Separation Phase 20)
//! Khách hàng UDS kết nối từ non-root Agent daemon tới privileged Execd qua Unix Socket `/run/aegisnode/execd.sock`.

use aegis_core::AegisError;
use aegis_rpc::{ExecRequest, ExecResponse};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

/// Đường dẫn mặc định của Unix Socket dành cho Execd daemon
pub const EXECD_SOCKET_PATH: &str = "/run/aegisnode/execd.sock";

/// Struct `ExecutorClient` thực hiện gửi yêu cầu đặc quyền qua Unix Domain Socket
pub struct ExecutorClient {
    socket_path: String,
}

impl ExecutorClient {
    /// Khởi tạo client với đường dẫn socket tùy chọn (mặc định /run/aegisnode/execd.sock)
    pub fn new(socket_path: Option<&str>) -> Self {
        Self {
            socket_path: socket_path.unwrap_or(EXECD_SOCKET_PATH).to_string(),
        }
    }

    /// Gửi một ExecRequest và nhận kết quả ExecResponse từ Execd daemon
    pub async fn execute(&self, request: ExecRequest) -> Result<ExecResponse, AegisError> {
        // 1. Mở kết nối Unix Stream tới Execd socket
        let mut stream = UnixStream::connect(&self.socket_path).await.map_err(|e| {
            AegisError::Internal(format!(
                "Không thể kết nối tới Execd daemon tại {}: {e}",
                self.socket_path
            ))
        })?;

        // 2. Serialize ExecRequest thành chuỗi JSON dạng newline-delimited
        let mut payload = serde_json::to_string(&request)
            .map_err(|e| AegisError::Internal(format!("Lỗi serialize ExecRequest: {e}")))?;
        payload.push('\n');

        // 3. Gửi payload qua socket
        stream
            .write_all(payload.as_bytes())
            .await
            .map_err(|e| AegisError::Internal(format!("Lỗi ghi dữ liệu tới Execd socket: {e}")))?;
        stream
            .flush()
            .await
            .map_err(|e| AegisError::Internal(format!("Lỗi flush Execd stream: {e}")))?;

        // 4. Đọc phản hồi phản hồi dạng dòng (Line reader)
        let (reader, _) = stream.into_split();
        let mut buf_reader = BufReader::new(reader);
        let mut response_line = String::new();
        buf_reader
            .read_line(&mut response_line)
            .await
            .map_err(|e| AegisError::Internal(format!("Lỗi đọc kết quả từ Execd socket: {e}")))?;

        // 5. Deserialize JSON thành ExecResponse
        let response: ExecResponse = serde_json::from_str(&response_line)
            .map_err(|e| AegisError::Internal(format!("Lỗi deserialize ExecResponse: {e}")))?;

        Ok(response)
    }
}
