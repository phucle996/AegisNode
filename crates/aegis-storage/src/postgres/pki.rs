// Quản lý Root CA Rotation & Active CA Storage trong PostgreSQL

use aegis_core::{AegisError, Result};

use super::PgRepository;

impl PgRepository {
    /// Nạp Root CA đang hoạt động từ PostgreSQL cho Controller Replicas trong môi trường HA
    pub async fn get_active_root_ca(&self) -> Result<Option<(String, String)>> {
        // Truy vấn bản ghi Root CA active duy nhất trong bảng cluster_pki_ca
        let row = sqlx::query_as::<_, (String, String)>(
            r#"
            SELECT ca_cert_pem, ca_key_pem
            FROM cluster_pki_ca
            WHERE active = TRUE
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to fetch active Root CA: {e}")))?;

        Ok(row)
    }

    /// Lưu trữ bản ghi Root CA mới vào PostgreSQL để sử dụng chung toàn Cluster (Sử dụng Transaction nguyên tử chống Race Condition)
    pub async fn save_root_ca(&self, ca_cert_pem: &str, ca_key_pem: &str) -> Result<()> {
        // Bắt đầu một database transaction mới
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AegisError::Storage(format!("Failed to begin CA rotation transaction: {e}")))?;

        // Vô hiệu hóa tất cả các bản ghi Root CA cũ trong transaction
        sqlx::query("UPDATE cluster_pki_ca SET active = FALSE WHERE active = TRUE")
            .execute(&mut *tx)
            .await
            .map_err(|e| AegisError::Storage(format!("Failed to deactivate old Root CAs: {e}")))?;

        // Lưu bản ghi Root CA mới với trạng thái active = TRUE trong transaction
        sqlx::query(
            r#"
            INSERT INTO cluster_pki_ca (ca_cert_pem, ca_key_pem, active)
            VALUES ($1, $2, TRUE)
            "#,
        )
        .bind(ca_cert_pem)
        .bind(ca_key_pem)
        .execute(&mut *tx)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to save Root CA: {e}")))?;

        // Commit transaction nguyên tử ghi nhận mọi thay đổi vào CSDL
        tx.commit()
            .await
            .map_err(|e| AegisError::Storage(format!("Failed to commit CA rotation transaction: {e}")))?;

        Ok(())
    }
}
