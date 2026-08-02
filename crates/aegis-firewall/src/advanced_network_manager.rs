//! Advanced Network & Firewall Security Manager (Phase 24 Enterprise Bonding, VRF & SYN Flood)
//! Quản lý Bonding, VRF isolation, SYN Flood Protection rules và Dynamic nftables IP Sets updates.

use aegis_core::AegisError;
use aegis_models::advanced_network::{BondingProfile, SynProxyConfig, VrfProfile};

/// Bộ Quản lý Mạng & Bảo mật Nâng cao (AdvancedNetworkManager)
pub struct AdvancedNetworkManager;

impl AdvancedNetworkManager {
    /// Kiểm tra bảo vệ Giao diện Card mạng Quản trị (Management Interface Protection)
    /// Ngăn chặn tuyệt đối việc biến card mạng SSH/Management chính thành slave bond gây ngắt kết nối hệ thống
    pub fn validate_management_interface_protection(
        slaves: &[String],
        management_iface: &str,
    ) -> Result<(), AegisError> {
        if slaves.contains(&management_iface.to_string()) {
            return Err(AegisError::Validation(format!(
                "Từ chối cấu hình Bond: Card mạng Quản trị SSH hiện tại ('{management_iface}') không được phép đưa vào danh sách slave bond!"
            )));
        }
        Ok(())
    }

    /// Kiểm tra tính hợp lệ của cấu hình VRF Profile
    pub fn validate_vrf_profile(profile: &VrfProfile) -> Result<(), AegisError> {
        if profile.vrf_name.is_empty() {
            return Err(AegisError::Validation(
                "Tên giao diện VRF không được để trống".to_string(),
            ));
        }

        if profile.table_id == 0 || profile.table_id > 65535 {
            return Err(AegisError::Validation(format!(
                "Routing Table ID của VRF ({}) phải nằm trong khoảng từ 1 đến 65535",
                profile.table_id
            )));
        }

        Ok(())
    }

    /// Sinh rule nftables Chống Tấn công SYN Flood Protection sử dụng `synproxy` và Rate Limiting
    pub fn generate_synproxy_nft_rule(config: &SynProxyConfig) -> String {
        format!(
            "tcp flags syn tcp option maxseg size set {} synproxy mss {} wscale {} limit rate {}/second burst 200 packets accept",
            config.mss, config.mss, config.wscale, config.syn_rate_limit
        )
    }

    /// Sinh câu lệnh nftables Cập nhật động IP Set mà KHÔNG cần reload/flush toàn bộ ruleset (Zero-Downtime Update)
    pub fn generate_dynamic_set_add_command(set_name: &str, ips: &[String]) -> String {
        if ips.is_empty() {
            return format!("nft add element inet aegisnode {set_name} {{ }}");
        }

        let ip_list = ips.join(", ");
        format!("nft add element inet aegisnode {set_name} {{ {ip_list} }}")
    }

    /// Sinh câu lệnh xóa phần tử khỏi IP Set trong nftables
    pub fn generate_dynamic_set_delete_command(set_name: &str, ips: &[String]) -> String {
        if ips.is_empty() {
            return format!("nft delete element inet aegisnode {set_name} {{ }}");
        }

        let ip_list = ips.join(", ");
        format!("nft delete element inet aegisnode {set_name} {{ {ip_list} }}")
    }
}
