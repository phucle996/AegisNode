// Trình quản lý Safe Apply & Automatic Rollback (SafeApplyManager) cho AegisNode
// Bảo vệ chống tự khóa server bằng cơ chế Rollback Timer, Health Checking và Concurrent Apply Lock an toàn chống Mutex Poisoning Panic

use std::collections::HashMap; // Import HashMap lưu trữ trạng thái transaction
use std::sync::{Arc, Mutex}; // Mutex và Arc xử lý đồng bộ đa luồng
use std::time::Duration; // Duration cấu hình timeout

use aegis_core::{AegisError, ExecutionId, Result}; // Định nghĩa Lỗi và ExecutionId
use aegis_models::firewall::FirewallPolicy; // FirewallPolicy data model
use tokio::task::JoinHandle; // Handle quản lý async timer task

use super::execution::{ApplyExecution, ExecutionState};
use super::health_check::HealthChecker;
use super::snapshot::FirewallSnapshot;
use crate::runtime::backend::FirewallBackend;
use crate::runtime::process_runner::ProcessRunner;

/// Struct SafeApplyManager điều phối toàn bộ chu trình Safe Apply
pub struct SafeApplyManager {
    backend: Arc<dyn FirewallBackend>,
    health_checker: Arc<HealthChecker>,
    active_lock: Arc<Mutex<Option<ExecutionId>>>,
    executions: Arc<Mutex<HashMap<ExecutionId, ApplyExecution>>>,
    snapshots: Arc<Mutex<HashMap<ExecutionId, FirewallSnapshot>>>,
    timer_handles: Arc<Mutex<HashMap<ExecutionId, JoinHandle<()>>>>,
}

impl SafeApplyManager {
    pub fn new(backend: Arc<dyn FirewallBackend>, runner: Arc<dyn ProcessRunner>) -> Self {
        Self {
            backend,
            health_checker: Arc::new(HealthChecker::new(runner)),
            active_lock: Arc::new(Mutex::new(None)),
            executions: Arc::new(Mutex::new(HashMap::new())),
            snapshots: Arc::new(Mutex::new(HashMap::new())),
            timer_handles: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Kiểm tra và chiếm quyền Concurrent Apply Lock (Tránh panic khi lock bị poisoned bằng unwrap_or_else)
    fn acquire_lock(&self, execution_id: &ExecutionId) -> Result<()> {
        let mut lock = self
            .active_lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(active_id) = lock.as_ref() {
            return Err(AegisError::Conflict(format!(
                "Another apply execution '{active_id}' is currently in progress. Operation rejected!"
            )));
        }
        *lock = Some(execution_id.clone());
        Ok(())
    }

    /// Giải phóng Concurrent Apply Lock an toàn
    fn release_lock(&self, execution_id: &ExecutionId) {
        let mut lock = self
            .active_lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if lock.as_ref() == Some(execution_id) {
            *lock = None;
        }
    }

    /// Thực thi Safe Apply với tùy chọn rollback timeout (mặc định 30 giây)
    pub async fn execute_safe_apply(
        &self,
        policy: &FirewallPolicy,
        timeout_seconds: u64,
    ) -> Result<ApplyExecution> {
        let execution_id = ExecutionId::new_v4();

        // 1. Concurrent Apply Lock check
        self.acquire_lock(&execution_id)?;

        // 2. Validate Policy
        let val_report = self.backend.validate(policy).await?;
        if !val_report.is_valid() {
            self.release_lock(&execution_id);
            return Err(AegisError::Validation(
                "Policy failed validation rules!".to_string(),
            ));
        }

        // 3. Compile Policy
        let compiled = match self.backend.compile(policy).await {
            Ok(c) => c,
            Err(e) => {
                self.release_lock(&execution_id);
                return Err(e);
            }
        };

        // 4. Snapshot current ruleset
        let snapshot = match self
            .backend
            .snapshot(&format!("Pre-apply snapshot for execution {execution_id}"))
            .await
        {
            Ok(s) => s,
            Err(e) => {
                self.release_lock(&execution_id);
                return Err(e);
            }
        };

        // 5. Apply compiled ruleset via backend
        match self.backend.apply(&compiled).await {
            Ok(res) => res,
            Err(e) => {
                self.release_lock(&execution_id);
                return Err(e);
            }
        };

        // 6. Post-Apply Health Check
        let health_report = match self.health_checker.run_checks().await {
            Ok(h) => h,
            Err(e) => {
                let _ = self.backend.rollback(&snapshot).await;
                self.release_lock(&execution_id);
                return Err(AegisError::Firewall(format!(
                    "Health check system failed: {e}. Auto-rolled back."
                )));
            }
        };

        if !health_report.success {
            let _ = self.backend.rollback(&snapshot).await;
            self.release_lock(&execution_id);
            return Err(AegisError::Firewall(format!(
                "Post-apply health checks failed: {}. Automatically rolled back!",
                health_report.failed_checks.join("; ")
            )));
        }

        // 7. Lưu thông tin execution & snapshot
        let mut execution = ApplyExecution::new(
            execution_id.clone(),
            policy.metadata.id.clone(),
            snapshot.snapshot_id.clone(),
            timeout_seconds,
        );
        execution.state = ExecutionState::AppliedPendingConfirmation;

        {
            let mut execs = self
                .executions
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            execs.insert(execution_id.clone(), execution.clone());
        }
        {
            let mut snaps = self
                .snapshots
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            snaps.insert(execution_id.clone(), snapshot.clone());
        }

        // 8. Khởi chạy Local Rollback Timer bất đồng bộ trong Daemon
        let self_clone = Arc::new(self.clone_refs());
        let timer_exec_id = execution_id.clone();
        let timer_snap = snapshot.clone();

        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(timeout_seconds)).await;
            // Nếu timer kích hoạt mà execution vẫn ở trạng thái PendingConfirmation -> Tự động Rollback
            self_clone
                .auto_rollback_on_timeout(&timer_exec_id, &timer_snap)
                .await;
        });

        {
            let mut handles = self
                .timer_handles
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            handles.insert(execution_id, handle);
        }

        Ok(execution)
    }

    /// Xác nhận thay đổi Policy (Confirm Apply Execution)
    pub async fn confirm(&self, execution_id: &ExecutionId) -> Result<ApplyExecution> {
        let mut execution = {
            let mut execs = self
                .executions
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            execs.get_mut(execution_id).cloned().ok_or_else(|| {
                AegisError::NotFound(format!("Apply execution '{execution_id}' not found"))
            })?
        };

        if execution.state != ExecutionState::AppliedPendingConfirmation {
            return Err(AegisError::Conflict(format!(
                "Cannot confirm execution '{execution_id}' in state {:?}",
                execution.state
            )));
        }

        // 1. Hủy Rollback Timer
        if let Some(handle) = self
            .timer_handles
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(execution_id)
        {
            handle.abort();
        }

        // 2. Chuyển trạng thái sang Committed
        execution.state = ExecutionState::Committed;
        {
            let mut execs = self
                .executions
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            execs.insert(execution_id.clone(), execution.clone());
        }

        // 3. Giải phóng Apply Lock
        self.release_lock(execution_id);

        Ok(execution)
    }

    /// Khôi phục thủ công (Manual Rollback)
    pub async fn rollback(&self, execution_id: &ExecutionId) -> Result<ApplyExecution> {
        let snapshot = {
            let snaps = self
                .snapshots
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            snaps.get(execution_id).cloned().ok_or_else(|| {
                AegisError::NotFound(format!("Snapshot for execution '{execution_id}' not found"))
            })?
        };

        // 1. Hủy Rollback Timer nếu có
        if let Some(handle) = self
            .timer_handles
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(execution_id)
        {
            handle.abort();
        }

        // 2. Cập nhật state sang RollingBack
        self.update_state(execution_id, ExecutionState::RollingBack);

        // 3. Gọi backend rollback
        let rollback_res = self.backend.rollback(&snapshot).await;

        let mut execution = self.get_execution(execution_id)?;
        if rollback_res.is_ok() {
            execution.state = ExecutionState::RolledBack;
        } else {
            execution.state = ExecutionState::Failed;
            execution.error_message = Some(format!("Rollback failed: {:?}", rollback_res.err()));
        }

        {
            let mut execs = self
                .executions
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            execs.insert(execution_id.clone(), execution.clone());
        }

        // 4. Giải phóng Apply Lock
        self.release_lock(execution_id);

        Ok(execution)
    }

    /// Tự động Rollback khi Rollback Timer trong Daemon hết hạn
    async fn auto_rollback_on_timeout(
        &self,
        execution_id: &ExecutionId,
        snapshot: &FirewallSnapshot,
    ) {
        let is_pending = {
            let execs = self
                .executions
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            execs
                .get(execution_id)
                .map(|e| e.state == ExecutionState::AppliedPendingConfirmation)
                .unwrap_or(false)
        };

        if is_pending {
            self.update_state(execution_id, ExecutionState::RollingBack);
            let res = self.backend.rollback(snapshot).await;

            let mut execs = self
                .executions
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(exec) = execs.get_mut(execution_id) {
                if res.is_ok() {
                    exec.state = ExecutionState::RolledBack;
                    exec.error_message =
                        Some("Auto-rolled back due to confirmation timeout".to_string());
                } else {
                    exec.state = ExecutionState::Failed;
                    exec.error_message = Some("Auto-rollback failed on timeout!".to_string());
                }
            }
            self.release_lock(execution_id);
        }
    }

    fn update_state(&self, execution_id: &ExecutionId, state: ExecutionState) {
        let mut execs = self
            .executions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(exec) = execs.get_mut(execution_id) {
            exec.state = state;
        }
    }

    pub fn get_execution(&self, execution_id: &ExecutionId) -> Result<ApplyExecution> {
        let execs = self
            .executions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        execs.get(execution_id).cloned().ok_or_else(|| {
            AegisError::NotFound(format!("Apply execution '{execution_id}' not found"))
        })
    }

    fn clone_refs(&self) -> Self {
        Self {
            backend: self.backend.clone(),
            health_checker: self.health_checker.clone(),
            active_lock: self.active_lock.clone(),
            executions: self.executions.clone(),
            snapshots: self.snapshots.clone(),
            timer_handles: self.timer_handles.clone(),
        }
    }
}
