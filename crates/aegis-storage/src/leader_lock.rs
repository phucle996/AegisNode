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

    /// Khởi chạy vòng lặp bầu chọn Leader background (Duy trì session connection liên tục)
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

        // Giữ kết nối Session duy nhất cho PostgreSQL Advisory Lock
        let mut dedicated_conn: Option<sqlx::pool::PoolConnection<sqlx::Postgres>> = None;
        let mut interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            interval.tick().await;

            if dedicated_conn.is_none() {
                // Nếu chưa có lock, thử mở connection mới và chiếm pg_try_advisory_lock
                if let Ok(mut conn) = pool.acquire().await {
                    let row: Result<(bool,), _> = sqlx::query_as("SELECT pg_try_advisory_lock($1)")
                        .bind(self.lock_key)
                        .fetch_one(&mut *conn)
                        .await;

                    if let Ok((true,)) = row {
                        // Giữ nguyên kết nối connection này trong dedicated_conn để duy trì Lock ở cấp độ Session
                        dedicated_conn = Some(conn);
                        let was_leader = self.is_leader.swap(true, Ordering::SeqCst);
                        if !was_leader {
                            info!(
                                "🎉 Replica này vừa chiếm được PostgreSQL Advisory Lock. Trở thành ACTIVE LEADER!"
                            );
                        }
                    } else {
                        self.is_leader.store(false, Ordering::SeqCst);
                    }
                }
            } else {
                // Đã có Lock, gửi ping kiểm tra kết nối định kỳ 5 giây
                if let Some(ref mut conn) = dedicated_conn {
                    if sqlx::query("SELECT 1").execute(&mut **conn).await.is_err() {
                        // Nếu kết nối tới CSDL bị gián đoạn, giải phóng dedicated_conn và hạ cấp xuống FOLLOWER
                        dedicated_conn = None;
                        let was_leader = self.is_leader.swap(false, Ordering::SeqCst);
                        if was_leader {
                            warn!("⚠️ Mất kết nối PostgreSQL Session. Chuyển sang trạng thái FOLLOWER!");
                        }
                    }
                }
            }
        }
    }
}
