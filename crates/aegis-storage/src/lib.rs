//! AegisNode Storage Crate
//! Quản lý SQLite (Local Agent) và PostgreSQL (Controller Server) Storage Engines & Leader Election.

pub mod db;
pub mod leader_lock;
pub mod postgres;
pub mod repository;

pub use db::{init_in_memory_pool, init_sqlite_pool};
pub use leader_lock::{CONTROLLER_LEADER_LOCK_KEY, LeaderElector, PostgresLeaderLock};
pub use postgres::{ApiTokenRecord, NodeRecord, PgRepository};
pub use repository::{
    AuditRecord, AuditRepository, ExecutionRepository, PolicyRepository, SqliteRepository,
};
