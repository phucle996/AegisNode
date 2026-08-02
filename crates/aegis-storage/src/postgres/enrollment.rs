// Quản lý Enrollment Tokens và Agent Certificates trong PostgreSQL

use aegis_core::pki::AgentCertificateRecord;
use aegis_core::{AegisError, Result};
use uuid::Uuid;

use super::PgRepository;

impl PgRepository {
    /// Lưu Enrollment Token vào PostgreSQL
    pub async fn insert_enrollment_token(
        &self,
        token_hash: &str,
        max_usages: i32,
        ttl_minutes: i64,
    ) -> Result<Uuid> {
        let id = Uuid::new_v4();
        // Ghi bản ghi enrollment token mới vào CSDL với thời gian hết hạn
        sqlx::query(
            r#"
            INSERT INTO enrollment_tokens (id, token_hash, max_usages, current_usages, expires_at)
            VALUES ($1, $2, $3, 0, CURRENT_TIMESTAMP + ($4 || ' minutes')::INTERVAL)
            "#,
        )
        .bind(id)
        .bind(token_hash)
        .bind(max_usages)
        .bind(ttl_minutes.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to insert enrollment token: {e}")))?;

        Ok(id)
    }

    /// Tiêu thụ One-Time Enrollment Token nguyên tử (Atomic Usage Increment & Strict Bound Verification)
    pub async fn consume_enrollment_token(&self, token_hash: &str) -> Result<bool> {
        // Thực thi câu lệnh UPDATE kiểm tra nguyên tử điều kiện current_usages < max_usages
        let result = sqlx::query(
            r#"
            UPDATE enrollment_tokens
            SET current_usages = current_usages + 1
            WHERE token_hash = $1 AND revoked = FALSE AND expires_at > CURRENT_TIMESTAMP AND current_usages < max_usages
            RETURNING id, current_usages, max_usages
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to consume enrollment token: {e}")))?;

        Ok(result.is_some())
    }

    /// Lưu trữ Agent Certificate vào PostgreSQL
    pub async fn save_agent_certificate(&self, cert: &AgentCertificateRecord) -> Result<()> {
        // Ghi nhận chứng chỉ X.509 cấp phát cho Agent Node
        sqlx::query(
            r#"
            INSERT INTO agent_certificates (serial_number, node_id, machine_id, hostname, cert_pem, issued_at, expires_at, revoked)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (serial_number) DO UPDATE SET
                cert_pem = EXCLUDED.cert_pem,
                expires_at = EXCLUDED.expires_at,
                revoked = EXCLUDED.revoked
            "#,
        )
        .bind(&cert.serial_number)
        .bind(cert.node_id)
        .bind(&cert.machine_id)
        .bind(&cert.hostname)
        .bind(&cert.cert_pem)
        .bind(cert.issued_at)
        .bind(cert.expires_at)
        .bind(cert.revoked)
        .execute(&self.pool)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to save agent certificate: {e}")))?;

        Ok(())
    }
}
