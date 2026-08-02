//! Peer Credential Validation Module (SO_PEERCRED)
//! Kiểm tra danh tính tiến trình (UID/GID/PID) kết nối qua Unix Domain Socket trên Linux Kernel.

use crate::error::AegisError;
use tokio::net::UnixStream;

/// Cấu trúc chứa thông tin danh tính Credential thu thập từ Kernel Linux
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerCredentials {
    /// User ID của tiến trình gọi
    pub uid: u32,
    /// Group ID của tiến trình gọi
    pub gid: u32,
    /// Process ID của tiến trình gọi
    pub pid: Option<i32>,
}

/// Trích xuất danh tính Peer Credentials từ Tokio UnixStream
pub fn extract_peer_credentials(stream: &UnixStream) -> Result<PeerCredentials, AegisError> {
    // Trích xuất UCred thông qua phương thức peer_cred() của Tokio trên Linux
    let cred = stream.peer_cred().map_err(|e| {
        AegisError::Permission(format!(
            "Không thể trích xuất SO_PEERCRED từ Unix socket: {e}"
        ))
    })?;

    Ok(PeerCredentials {
        uid: cred.uid(),
        gid: cred.gid(),
        pid: cred.pid(),
    })
}

/// Xác thực xem UID của tiến trình gọi socket có thuộc danh sách UID được phép hay không
pub fn validate_peer_uid(
    stream: &UnixStream,
    allowed_uids: &[u32],
) -> Result<PeerCredentials, AegisError> {
    // Trích xuất danh tính caller
    let creds = extract_peer_credentials(stream)?;

    // Nếu UID của caller nằm trong danh sách cho phép (ví dụ UID của aegisnode-agent) hoặc root (0)
    if allowed_uids.contains(&creds.uid) || creds.uid == 0 {
        Ok(creds)
    } else {
        Err(AegisError::Permission(format!(
            "Từ chối kết nối Unix socket: UID {} không nằm trong danh sách được phép",
            creds.uid
        )))
    }
}
