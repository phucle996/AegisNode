// SSH Brute-Force Detector & Sliding Window Threshold Engine cho AegisNode
// Theo dõi số lần đăng nhập thất bại của mỗi IP trong khoảng window_seconds
// Tự động kích hoạt BlockManager khóa IP khi vượt ngưỡng threshold mà không làm rò rỉ bộ nhớ

use std::collections::HashMap;

use aegis_core::Result;
use aegis_models::blocker::BlockEntry;
use chrono::{DateTime, Utc};

use super::block_manager::BlockManager;

/// Bản ghi vết đợt thử đăng nhập của một IP
#[derive(Debug, Clone)]
struct IpAttemptTrack {
    timestamps: Vec<DateTime<Utc>>,
}

/// Động cơ phát hiện dò quét mật khẩu SSH tự động
pub struct SshDetector {
    threshold: u32,
    window_seconds: u64,
    block_seconds: u64,
    tracks: HashMap<String, IpAttemptTrack>,
}

impl SshDetector {
    pub fn new(threshold: u32, window_seconds: u64, block_seconds: u64) -> Self {
        Self {
            threshold,
            window_seconds,
            block_seconds,
            tracks: HashMap::new(),
        }
    }

    /// Ghi nhận một sự kiện SSH authentication failure từ journal/log
    pub fn record_failure(
        &mut self,
        ip: &str,
        block_mgr: &mut BlockManager,
    ) -> Result<Option<BlockEntry>> {
        let trimmed_ip = ip.trim();
        let now = Utc::now();
        let cutoff = now - chrono::Duration::seconds(self.window_seconds as i64);

        let track = self
            .tracks
            .entry(trimmed_ip.to_string())
            .or_insert_with(|| IpAttemptTrack {
                timestamps: Vec::new(),
            });

        // 1. Loại bỏ các timestamps đã trôi qua ngoài sliding window
        track.timestamps.retain(|t| *t >= cutoff);
        track.timestamps.push(now);

        // 2. Kiểm tra xem số đợt thất bại trong cửa sổ thời gian có vượt ngưỡng không
        if track.timestamps.len() >= self.threshold as usize {
            track.timestamps.clear(); // Clear track sau khi block
            let entry = block_mgr.add_block(
                trimmed_ip,
                Some(self.block_seconds),
                "SSH authentication brute-force attempt detected",
                "ssh_detector",
            )?;
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    /// Dọn dẹp bộ nhớ theo định kỳ
    pub fn cleanup(&mut self) {
        let now = Utc::now();
        let cutoff = now - chrono::Duration::seconds(self.window_seconds as i64);
        self.tracks.retain(|_, track| {
            track.timestamps.retain(|t| *t >= cutoff);
            !track.timestamps.is_empty()
        });
    }
}
