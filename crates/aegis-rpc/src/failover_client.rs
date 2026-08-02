//! Multi-Endpoint Agent Failover Client (Phase 23 Controller HA)
//! Cung cấp khả năng xoay vòng (Endpoint rotation) và chuyển vùng dự phòng (Failover) khi một Controller replica bị ngắt kết nối.

use aegis_core::AegisError;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::{info, warn};

/// Agent Failover RPC Client quản lý danh sách Controller Endpoints
pub struct FailoverRpcClient {
    endpoints: Vec<String>,
    current_index: AtomicUsize,
}

impl FailoverRpcClient {
    /// Khởi tạo Client với danh sách Controller endpoints
    pub fn new(endpoints: Vec<String>) -> Result<Self, AegisError> {
        if endpoints.is_empty() {
            return Err(AegisError::Configuration(
                "Danh sách Controller endpoints không được để trống".to_string(),
            ));
        }

        Ok(Self {
            endpoints,
            current_index: AtomicUsize::new(0),
        })
    }

    /// Trả về Endpoint hiện đang được kết nối chính (Active Endpoint)
    pub fn active_endpoint(&self) -> String {
        let idx = self.current_index.load(Ordering::Relaxed) % self.endpoints.len();
        self.endpoints[idx].clone()
    }

    /// Thất bại kết nối -> Tự động xoay vòng sang Controller Endpoint tiếp theo (Failover)
    pub fn rotate_to_next_endpoint(&self) -> String {
        let old_idx = self.current_index.fetch_add(1, Ordering::SeqCst);
        let new_idx = (old_idx + 1) % self.endpoints.len();
        let next_endpoint = self.endpoints[new_idx].clone();

        warn!(
            "Mất kết nối tới Controller! Tự động chuyển vùng dự phòng (Failover) sang Controller Endpoint mới: {}",
            next_endpoint
        );

        next_endpoint
    }

    /// Gửi request qua active endpoint, tự động retry failover sang endpoint tiếp theo nếu lỗi
    pub async fn execute_with_failover<F, Fut, T>(&self, f: F) -> Result<T, AegisError>
    where
        F: Fn(String) -> Fut,
        Fut: std::future::Future<Output = Result<T, AegisError>>,
    {
        let max_attempts = self.endpoints.len();

        for attempt in 0..max_attempts {
            let endpoint = self.active_endpoint();
            info!(
                "Thử kết nối mTLS tới Controller endpoint ({}/{}): {}",
                attempt + 1,
                max_attempts,
                endpoint
            );

            match f(endpoint).await {
                Ok(val) => return Ok(val),
                Err(e) => {
                    warn!("Lỗi kết nối tới Controller: {e}");
                    self.rotate_to_next_endpoint();
                }
            }
        }

        Err(AegisError::Timeout(
            "Đã thử kết nối tới toàn bộ Controller endpoints trong danh sách nhưng đều thất bại!"
                .to_string(),
        ))
    }
}
