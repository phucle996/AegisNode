// Multi-Node Rollout Coordinator Engine cho AegisNode Stage 2
// Điều phối chiến lược Canary / Batch / AllAtOnce / Manual trên fleet node,
// phát hiện ngưỡng lỗi, bảo vệ Canary guard và tính toán thứ tự rollback.

use aegis_models::change_plan::{
    BatchConfig, NodeRolloutState, NodeRolloutStatus, RolloutSpec, RolloutStrategy,
};
use uuid::Uuid;

/// Engine điều phối Multi-Node Rollout
#[derive(Debug, Clone, Default)]
pub struct RolloutCoordinator;

impl RolloutCoordinator {
    pub fn new() -> Self {
        Self
    }

    /// Chọn Canary node (luôn là node đầu tiên trong targets)
    pub fn select_canary_node(targets: &[Uuid]) -> Option<Uuid> {
        targets.first().copied()
    }

    /// Kiểm tra Canary node đã fail chưa — nếu fail thì dừng toàn fleet
    pub fn should_stop_on_canary_fail(
        canary_id: Uuid,
        node_statuses: &[NodeRolloutStatus],
    ) -> bool {
        node_statuses
            .iter()
            .find(|s| s.node_id == canary_id)
            .map(|s| s.state == NodeRolloutState::Failed)
            .unwrap_or(false)
    }

    /// Tính batch nodes tiếp theo cần rollout dựa trên strategy
    pub fn compute_next_batch(
        spec: &RolloutSpec,
        node_statuses: &[NodeRolloutStatus],
    ) -> Vec<Uuid> {
        let pending: Vec<Uuid> = node_statuses
            .iter()
            .filter(|s| s.state == NodeRolloutState::Pending)
            .map(|s| s.node_id)
            .collect();

        match spec.strategy {
            RolloutStrategy::AllAtOnce => pending,

            RolloutStrategy::Manual => {
                // Manual: Admin phải xác nhận từng node — trả về node đầu tiên đang Pending
                pending.into_iter().take(1).collect()
            }

            RolloutStrategy::Canary => {
                // Canary: node đầu tiên là canary, sau khi Succeeded mới cho tiếp
                let canary = Self::select_canary_node(&spec.targets);
                let canary_done = canary.map_or(false, |cid| {
                    node_statuses
                        .iter()
                        .find(|s| s.node_id == cid)
                        .map(|s| s.state == NodeRolloutState::Succeeded)
                        .unwrap_or(false)
                });
                let canary_failed = canary.map_or(false, |cid| {
                    Self::should_stop_on_canary_fail(cid, node_statuses)
                });

                if canary_failed {
                    // Canary fail → DỪNG toàn fleet, không rollout thêm node nào
                    vec![]
                } else if canary_done {
                    // Canary pass → rollout phần còn lại theo batch_size
                    pending
                        .into_iter()
                        .filter(|id| Some(*id) != canary)
                        .take(spec.batch_config.batch_size.max(1))
                        .collect()
                } else {
                    // Canary chưa xong → chỉ rollout canary node nếu nó đang Pending
                    match canary {
                        Some(cid) if pending.contains(&cid) => vec![cid],
                        _ => vec![],
                    }
                }
            }

            RolloutStrategy::Batch => {
                // Batch: tính số running hiện tại, không vượt quá max_unavailable
                let running_count = node_statuses
                    .iter()
                    .filter(|s| s.state == NodeRolloutState::Running)
                    .count();
                let BatchConfig {
                    batch_size,
                    max_unavailable,
                    ..
                } = spec.batch_config;
                let can_dispatch = max_unavailable.saturating_sub(running_count);
                pending
                    .into_iter()
                    .take(batch_size.min(can_dispatch))
                    .collect()
            }
        }
    }

    /// Kiểm tra tỉ lệ thất bại có vượt failure_threshold_percent chưa
    pub fn check_failure_threshold(
        spec: &RolloutSpec,
        node_statuses: &[NodeRolloutStatus],
    ) -> bool {
        let total = spec.targets.len();
        if total == 0 {
            return false;
        }
        let failed_count = node_statuses
            .iter()
            .filter(|s| s.state == NodeRolloutState::Failed)
            .count();
        let percent = (failed_count * 100) / total;
        percent >= spec.failure_threshold_percent as usize
    }

    /// Tính danh sách nodes cần Rollback (các nodes đã Succeeded cần hoàn tác)
    pub fn compute_rollback_targets(node_statuses: &[NodeRolloutStatus]) -> Vec<Uuid> {
        // Rollback theo thứ tự ngược lại (nodes succeed cuối cùng rollback trước)
        let mut succeeded: Vec<Uuid> = node_statuses
            .iter()
            .filter(|s| s.state == NodeRolloutState::Succeeded)
            .map(|s| s.node_id)
            .collect();
        succeeded.reverse();
        succeeded
    }

    /// Khởi tạo danh sách NodeRolloutStatus từ danh sách targets
    pub fn init_node_statuses(targets: &[Uuid]) -> Vec<NodeRolloutStatus> {
        targets
            .iter()
            .map(|&node_id| NodeRolloutStatus {
                node_id,
                state: NodeRolloutState::Pending,
                current_step: None,
                error_message: None,
            })
            .collect()
    }
}
