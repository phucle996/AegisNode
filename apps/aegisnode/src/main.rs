// AegisNode Main Application Entrypoint
// Hỗ trợ chế độ Multi-call binary thông qua các Subcommands: local, server, agent, execd, ctl

use clap::{Parser, Subcommand};
use tracing::info;

/// CLI Parser định nghĩa cấu trúc lệnh ứng dụng AegisNode
#[derive(Parser, Debug)]
#[command(name = "aegisnode")]
#[command(author = "AegisNode Team")]
#[command(version = "0.1.0")]
#[command(about = "Linux Firewall, Network and Service Management Platform written in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Danh sách các Subcommand được hỗ trợ theo thiết kế kiến trúc khóa
#[derive(Subcommand, Debug)]
enum Commands {
    /// Chạy ở chế độ Standalone Single-Node (MVP mode: Local Agent + HTTP/Socket API)
    Local,

    /// Chạy ở chế độ Aegis Central Controller (Middle & Prod Stage)
    Server,

    /// Chạy ở chế độ Unprivileged Node Agent
    Agent,

    /// Chạy ở chế độ Privileged Executor Helper
    Execd,

    /// Lệnh CLI quản trị hệ thống (Local hoặc Remote)
    Ctl,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Khởi tạo hệ thống Tracing và Logging
    aegis_observability::init_logging();

    // Parse tham số đầu vào từ CLI
    let cli = Cli::parse();

    match cli.command {
        Commands::Local => {
            info!("Starting AegisNode in LOCAL mode (Standalone Monolith MVP)...");
            println!("AegisNode local daemon started successfully.");
        }
        Commands::Server => {
            info!("Starting AegisNode in SERVER mode (Central Controller)...");
            println!("AegisNode controller server mode initialized.");
        }
        Commands::Agent => {
            info!("Starting AegisNode in AGENT mode...");
            println!("AegisNode agent mode initialized.");
        }
        Commands::Execd => {
            info!("Starting AegisNode in EXECD mode (Privileged Executor)...");
            println!("AegisNode execd mode initialized.");
        }
        Commands::Ctl => {
            info!("Running AegisNode CTL command...");
            println!("AegisNode ctl mode executed.");
        }
    }

    Ok(())
}
