// Quản lý Combined Change Plan Rollouts và Service Policies trong PostgreSQL

use aegis_core::{AegisError, Result};
use aegis_models::change_plan::NodeChangePlan;
use uuid::Uuid;

use super::PgRepository;

impl PgRepository {
    /// Lưu Service Policy (Allowlist & Protected Units) vào PostgreSQL
    pub async fn save_service_policy(
        &self,
        name: &str,
        allowed_units: &[String],
        protected_units: &[String],
    ) -> Result<Uuid> {
        let id = Uuid::new_v4();
        // Ghi bản ghi Service Policy giới hạn các unit systemd được phép thao tác
        sqlx::query(
            r#"
            INSERT INTO service_policies (id, name, allowed_units, protected_units, updated_at)
            VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP)
            ON CONFLICT (name) DO UPDATE SET
                allowed_units = EXCLUDED.allowed_units,
                protected_units = EXCLUDED.protected_units,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(allowed_units)
        .bind(protected_units)
        .execute(&self.pool)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to save service policy: {e}")))?;

        Ok(id)
    }

    /// Khởi tạo Combined Rollout Plan với Idempotency Key
    pub async fn create_rollout(&self, plan: &NodeChangePlan) -> Result<Uuid> {
        let risk_str = format!("{:?}", plan.risk_level);
        // Ghi kế hoạch rollout mới với trạng thái CREATED
        sqlx::query(
            r#"
            INSERT INTO rollouts (id, idempotency_key, risk_level, state, updated_at)
            VALUES ($1, $2, $3, 'CREATED', CURRENT_TIMESTAMP)
            ON CONFLICT (idempotency_key) DO UPDATE SET
                risk_level = EXCLUDED.risk_level,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(plan.id)
        .bind(&plan.idempotency_key)
        .bind(risk_str)
        .execute(&self.pool)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to create rollout: {e}")))?;

        // Ghi mục tiêu Node cần rollout vào bảng rollout_targets
        sqlx::query(
            r#"
            INSERT INTO rollout_targets (rollout_id, node_id, state, current_step, updated_at)
            VALUES ($1, $2, 'RUNNING', 'step_01_snapshot', CURRENT_TIMESTAMP)
            ON CONFLICT (rollout_id, node_id) DO UPDATE SET
                state = EXCLUDED.state,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(plan.id)
        .bind(plan.target_node_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to create rollout target: {e}")))?;

        Ok(plan.id)
    }

    /// Cập nhật trạng thái tổng thể Rollout
    pub async fn update_rollout_state(&self, rollout_id: Uuid, state: &str) -> Result<()> {
        // Cập nhật trạng thái Rollout Plan theo Id
        sqlx::query(
            r#"
            UPDATE rollouts
            SET state = $1, updated_at = CURRENT_TIMESTAMP
            WHERE id = $2
            "#,
        )
        .bind(state)
        .bind(rollout_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to update rollout state: {e}")))?;
        Ok(())
    }

    /// Cập nhật trạng thái Node trong Rollout
    pub async fn update_node_rollout_status(
        &self,
        rollout_id: Uuid,
        node_id: Uuid,
        state: &str,
        current_step: Option<&str>,
        error_message: Option<&str>,
    ) -> Result<()> {
        // Upsert bản ghi trạng thái rollout của target node
        sqlx::query(
            r#"
            INSERT INTO rollout_targets (rollout_id, node_id, state, current_step, error_message, updated_at)
            VALUES ($1, $2, $3, $4, $5, CURRENT_TIMESTAMP)
            ON CONFLICT (rollout_id, node_id) DO UPDATE SET
                state = EXCLUDED.state,
                current_step = EXCLUDED.current_step,
                error_message = EXCLUDED.error_message,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(rollout_id)
        .bind(node_id)
        .bind(state)
        .bind(current_step)
        .bind(error_message)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AegisError::Storage(format!("Failed to update node rollout status: {e}"))
        })?;
        Ok(())
    }

    /// Lấy danh sách NodeRolloutStatus để Resume sau Controller Restart
    pub async fn get_rollout_targets(&self, rollout_id: Uuid) -> Result<Vec<(Uuid, String)>> {
        // Truy vấn các node target đang thuộc kế hoạch rollout
        let rows = sqlx::query_as::<_, (Uuid, String)>(
            r#"
            SELECT node_id, state
            FROM rollout_targets
            WHERE rollout_id = $1
            ORDER BY updated_at ASC
            "#,
        )
        .bind(rollout_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to get rollout targets: {e}")))?;
        Ok(rows)
    }
}
