// Quản lý Network Profiles trong PostgreSQL

use aegis_core::{AegisError, Result};
use aegis_models::network_profile::NetworkProfile;

use super::PgRepository;

impl PgRepository {
    /// Lưu Network Profile vào PostgreSQL Database
    pub async fn save_network_profile(&self, profile: &NetworkProfile) -> Result<()> {
        let profile_json = serde_json::to_value(profile).map_err(|e| {
            AegisError::Storage(format!("Failed to serialize network profile: {e}"))
        })?;

        // Ghi bản ghi network profile dạng JSON vào CSDL
        sqlx::query(
            r#"
            INSERT INTO network_profiles (id, name, description, profile_data, updated_at)
            VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP)
            ON CONFLICT (name) DO UPDATE SET
                description = EXCLUDED.description,
                profile_data = EXCLUDED.profile_data,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(profile.id)
        .bind(&profile.name)
        .bind(&profile.description)
        .bind(profile_json)
        .execute(&self.pool)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to save network profile: {e}")))?;

        Ok(())
    }

    /// Truy vấn tất cả Network Profiles đã lưu trong PostgreSQL Database
    pub async fn list_network_profiles(&self) -> Result<Vec<NetworkProfile>> {
        // Lấy danh sách cột profile_data dạng JSON từ bảng network_profiles
        let rows = sqlx::query_scalar::<_, serde_json::Value>(
            r#"
            SELECT profile_data
            FROM network_profiles
            ORDER BY updated_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to list network profiles: {e}")))?;

        // Giải mã danh sách JSON Value thành Vec<NetworkProfile>
        let profiles = rows
            .into_iter()
            .filter_map(|val| serde_json::from_value::<NetworkProfile>(val).ok())
            .collect();

        Ok(profiles)
    }
}
