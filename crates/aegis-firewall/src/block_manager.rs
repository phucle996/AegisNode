// Trình quản lý BlockManager xử lý việc khóa IP (Manual Block & Auto Block)
// Đảm bảo kiểm tra Allowlist tuyệt đối trước khi block, hỗ trợ nftables set element generation và dọn dẹp expired entries

use std::collections::HashMap;
use std::net::IpAddr;

use aegis_core::{AegisError, Result};
use aegis_models::blocker::{BlockEntry, BlockerConfig};
use chrono::Utc;
use ipnet::IpNet;

/// Trình quản lý IP Blocklist trong bộ nhớ và nftables kernel sets
pub struct BlockManager {
    config: BlockerConfig,
    entries: HashMap<String, BlockEntry>,
}

impl BlockManager {
    pub fn new(config: BlockerConfig) -> Self {
        Self {
            config,
            entries: HashMap::new(),
        }
    }

    /// Thêm một IP vào danh sách Block
    pub fn add_block(
        &mut self,
        ip: &str,
        duration_seconds: Option<u64>,
        reason: &str,
        actor: &str,
    ) -> Result<BlockEntry> {
        let trimmed_ip = ip.trim();

        // 1. Kiểm tra Allowlist bảo vệ loopback và management CIDRs
        if self.is_allowlisted(trimmed_ip) {
            return Err(AegisError::Validation(format!(
                "Cannot block IP '{trimmed_ip}' because it is in the management Allowlist!"
            )));
        }

        let now = Utc::now();
        let expires_at = duration_seconds.map(|secs| now + chrono::Duration::seconds(secs as i64));

        let entry = BlockEntry {
            ip: trimmed_ip.to_string(),
            reason: reason.to_string(),
            actor: actor.to_string(),
            duration_seconds,
            created_at: now,
            expires_at,
        };

        self.entries.insert(trimmed_ip.to_string(), entry.clone());
        Ok(entry)
    }

    /// Gỡ bỏ một IP khỏi danh sách Block
    pub fn remove_block(&mut self, ip: &str) -> Result<Option<BlockEntry>> {
        Ok(self.entries.remove(ip.trim()))
    }

    /// Lấy danh sách các IP đang bị block (loại bỏ các bản ghi đã hết hạn)
    pub fn list_blocks(&mut self) -> Vec<BlockEntry> {
        self.cleanup_expired();
        self.entries.values().cloned().collect()
    }

    /// Kiểm tra một IP có nằm trong Allowlist hay không
    pub fn is_allowlisted(&self, ip: &str) -> bool {
        let ip_addr: IpAddr = match ip.parse() {
            Ok(addr) => addr,
            Err(_) => return false,
        };

        if ip_addr.is_loopback() {
            return true;
        }

        for cidr in &self.config.allowlist {
            if let Ok(net) = cidr.0.parse::<IpNet>() {
                if net.contains(&ip_addr) {
                    return true;
                }
            }
        }

        false
    }

    /// Tự động dọn dẹp các entry đã quá hạn timeout
    pub fn cleanup_expired(&mut self) {
        self.entries.retain(|_, entry| !entry.is_expired());
    }

    /// Sinh câu lệnh nftables để nạp element vào set `blocked_ipv4` hoặc `blocked_ipv6`
    pub fn build_nft_add_element_cmd(entry: &BlockEntry) -> String {
        let set_name = if entry.ip.contains(':') {
            "blocked_ipv6"
        } else {
            "blocked_ipv4"
        };

        if let Some(dur) = entry.duration_seconds {
            format!(
                "add element inet aegis_filter {set_name} {{ {} timeout {}s }}",
                entry.ip, dur
            )
        } else {
            format!(
                "add element inet aegis_filter {set_name} {{ {} }}",
                entry.ip
            )
        }
    }
}
