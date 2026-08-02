//! Distributed Leader Election & PostgreSQL Advisory Locks (Phase 23 Controller HA)
//! Cung cấp cơ chế Bầu chọn Leader phân tán giữa nhiều Controller Replicas thông qua PostgreSQL Advisory Locks.

use aegis_core::AegisError;
use sqlx::PgPool;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tracing::{info, warn};

/// Mã khóa duy nhất đại diện cho Leader Election Advisory Lock trong PostgreSQL (Key ID 88888888)
pub const CONTROLLER_LEADER_LOCK_KEY: i64 = 88888888;

/// Bộ Quản lý Lock Bầu chọn Leader phân tán (PostgresLeaderLock)
pub struct PostgresLeaderLock;

impl PostgresLeaderLock {
    /// Thử chiếm giữ PostgreSQL Advisory Lock (`pg_try_advisory_lock`)
    pub async fn try_acquire_lock(pool: &PgPool, lock_key: i64) -> Result<bool, AegisError> {
        // Thực hiện query pg_try_advisory_lock(key)
        let row: (bool,) = sqlx::query_as("SELECT pg_try_advisory_lock($1)")
            .bind(lock_key)
            .fetch_one(pool)
            .await
            .map_err(|e| {
                AegisError::Storage(format!(
                    "Lỗi thực thi pg_try_advisory_lock({lock_key}): {e}"
                ))
            })?;

        Ok(row.0)
    }

    /// Giải phóng PostgreSQL Advisory Lock (`pg_advisory_unlock`)
    pub async fn release_lock(pool: &PgPool, lock_key: i64) -> Result<bool, AegisError> {
        // Thực hiện query pg_advisory_unlock(key)
        let row: (bool,) = sqlx::query_as("SELECT pg_advisory_unlock($1)")
            .bind(lock_key)
            .fetch_one(pool)
            .await
            .map_err(|e| {
                AegisError::Storage(format!("Lỗi thực thi pg_advisory_unlock({lock_key}): {e}"))
            })?;

        Ok(row.0)
    }
}

/// Task duy trì vòng lặp Bầu chọn Leader (LeaderElector)
pub struct LeaderElector {
    pool: Option<PgPool>,
    is_leader: Arc<AtomicBool>,
    lock_key: i64,
}

impl LeaderElector {
    /// Khởi tạo LeaderElector mới
    pub fn new(pool: Option<PgPool>, is_leader: Arc<AtomicBool>) -> Self {
        Self {
            pool,
            is_leader,
            lock_key: CONTROLLER_LEADER_LOCK_KEY,
        }
    }

    /// Khởi chạy vòng lặp bầu chọn Leader background (Renew lock mỗi 5 giây)
    pub async fn run_election_loop(self) {
        let pool = match self.pool {
            Some(p) => p,
            None => {
                warn!(
                    "Controller running without PostgreSQL pool. Defaulting to standalone Leader mode."
                );
                self.is_leader.store(true, Ordering::SeqCst);
                return;
            }
        };

        let mut interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            interval.tick().await;

            match PostgresLeaderLock::try_acquire_lock(&pool, self.lock_key).await {
                Ok(acquired) => {
                    let was_leader = self.is_leader.swap(acquired, Ordering::SeqCst);
                    if acquired && !was_leader {
                        info!(
                            "🎉 Replica này vừa chiếm được PostgreSQL Advisory Lock. Trở thành ACTIVE LEADER!"
                        );
                    } else if !acquired && was_leader {
                        warn!("⚠️ Mất PostgreSQL Advisory Lock. Chuyển sang trạng thái FOLLOWER!");
                    }
                }
                Err(e) => {
                    warn!("Lỗi khi thử chiếm giữ Leader Advisory Lock: {e}");
                    self.is_leader.store(false, Ordering::SeqCst);
                }
            }
        }
    }
}
