// Clap CLI Argument Structure cho `aegisctl` / `aegisnode ctl`
// Định nghĩa toàn bộ cây lệnh: status, firewall, docker, audit, version

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

/// Định dạng xuất dữ liệu ra màn hình
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
}

/// Cấu trúc tham số dòng lệnh AegisNode CLI
#[derive(Parser, Debug)]
#[command(name = "aegisctl")]
#[command(author = "AegisNode Team")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "AegisNode CLI Control Utility", long_about = None)]
pub struct CliArgs {
    /// Định dạng dữ liệu xuất ra (table, json, yaml)
    #[arg(short, long, global = true, value_enum, default_value_t = OutputFormat::Table)]
    pub output: OutputFormat,

    /// Địa chỉ Endpoint API Agent (Ví dụ: http://127.0.0.1:8080)
    #[arg(short, long, global = true, default_value = "http://127.0.0.1:8080")]
    pub endpoint: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Kiểm tra trạng thái Daemon và khả năng tương thích của hệ thống
    Status,

    /// Quản lý và xử lý Firewall Policy (validate, compile, apply, confirm, rollback, rules)
    Firewall {
        #[command(subcommand)]
        subcommand: FirewallCommands,
    },

    /// Kiểm định và phân tích rủi ro phơi nhiễm cổng của Docker Containers
    Docker {
        #[command(subcommand)]
        subcommand: DockerCommands,
    },

    /// Truy vấn lịch sử Audit log hệ thống
    Audit {
        /// Số lượng bản ghi cần hiển thị
        #[arg(short, long, default_value_t = 20)]
        limit: usize,
    },

    /// In thông tin phiên bản AegisNode CLI
    Version,
}

#[derive(Subcommand, Debug)]
pub enum FirewallCommands {
    /// Validate định dạng và kiểm tra cảnh báo an ninh cho tập tin Policy
    Validate {
        /// Đường dẫn tập tin policy YAML/JSON
        file: PathBuf,
    },

    /// Biên dịch Policy thành kịch bản nftables (Compile Preview)
    Compile {
        /// Đường dẫn tập tin policy YAML/JSON
        file: PathBuf,
    },

    /// Thực thi nạp Policy một cách an toàn (Safe Apply với Rollback Timer)
    Apply {
        /// Đường dẫn tập tin policy YAML/JSON
        file: PathBuf,

        /// Thời hạn tự động Rollback nếu không được confirm (giây)
        #[arg(short, long, default_value_t = 30)]
        rollback_after: u64,

        /// Tự động đồng ý qua tất cả cảnh báo an ninh không cần nhắc interactive
        #[arg(short, long, default_value_t = false)]
        yes: bool,
    },

    /// Xác nhận nạp thành công đợt Apply Execution đang ở trạng thái Pending
    Confirm {
        /// Mã đợt thực thi (Execution ID)
        execution_id: String,
    },

    /// Khôi phục thủ công về trạng thái snapshot trước Apply
    Rollback {
        /// Mã đợt thực thi (Execution ID)
        execution_id: String,
    },

    /// Hiển thị danh sách các ruleset đang active
    Rules,

    /// Hiển thị thông số lưu lượng đếm (Rule Counters)
    Counters,
}

#[derive(Subcommand, Debug)]
pub enum DockerCommands {
    /// Hiển thị danh sách Docker Containers inventory
    Containers,

    /// Phân tích rủi ro phơi nhiễm cổng public ra WAN (0.0.0.0)
    Exposure,
}
