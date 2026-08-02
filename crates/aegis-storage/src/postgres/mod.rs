// PostgreSQL Repository Layer cho AegisNode Controller Server (`aegisnode server`)
// Phục vụ lưu trữ tập trung Multi-Node Cloud Native Cluster

use std::time::Duration;

use aegis_core::{AegisError, Result};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

pub mod enrollment;
pub mod inventory;
pub mod network;
pub mod node;
pub mod pki;
pub mod rollout;

pub use node::NodeRecord;

/// Controller PostgreSQL Repository Interface chính
#[derive(Clone)]
pub struct PgRepository {
    pub(crate) pool: Pool<Postgres>,
}

impl PgRepository {
    /// Khởi tạo PostgreSQL Connection Pool với timeout và max connections
    pub async fn connect(url: &str, max_connections: u32, timeout_sec: u64) -> Result<Self> {
        // Tạo pool kết nối PostgreSQL với các cấu hình giới hạn
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .acquire_timeout(Duration::from_secs(timeout_sec))
            .connect(url)
            .await
            .map_err(|e| {
                AegisError::Storage(format!("Failed to connect to PostgreSQL at '{url}': {e}"))
            })?;

        // Tự động kiểm tra và nâng cấp CSDL PostgreSQL (Auto-migrations) khi khởi chạy daemon
        if let Err(e) = sqlx::migrate!("./migrations_postgres").run(&pool).await {
            tracing::warn!("Auto-migration warning (schema may already be managed): {e}");
        }

        Ok(Self { pool })
    }

    /// Khởi tạo instance từ PgPool sẵn có
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    /// Trả về tham chiếu tới inner pool
    pub fn pool(&self) -> &Pool<Postgres> {
        &self.pool
    }
}
