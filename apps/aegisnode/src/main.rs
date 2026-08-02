// Multi-call binary chính cho AegisNode: aegisnode local & aegisnode server
// Khởi tạo Agent Daemon (`local`) và Control Plane Controller Server (`server`) cho nền tảng Multi-Node

use std::path::PathBuf;
use std::sync::Arc;

use aegis_api::{AppState, ControllerState, create_controller_router, create_router};
use aegis_config::{AgentConfig, ControllerConfig};
use aegis_core::Result;
use aegis_firewall::{
    CapabilityDetector, DefaultProcessRunner, NftablesRuntimeBackend, SafeApplyManager,
    SnapshotManager,
};
use aegis_storage::{PgRepository, SqliteRepository, init_sqlite_pool};
use clap::{Parser, Subcommand};
use tracing::{info, warn};

#[derive(Parser)]
#[command(name = "aegisnode")]
#[command(about = "AegisNode - Cloud-Native Host & Microsegmentation Firewall Engine", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start AegisNode in Single-node Local Standalone Daemon mode
    Local {
        #[arg(short, long, default_value = "/etc/aegisnode/agent.yaml")]
        config: PathBuf,
    },
    /// Start AegisNode in Control Plane Server (Controller) mode
    Server {
        #[arg(short, long, default_value = "/etc/aegisnode/controller.yaml")]
        config: PathBuf,
    },
    /// Start AegisNode in Managed Agent mode
    Agent,
    /// Start AegisNode Execution Daemon
    Execd,
    /// AegisNode Control CLI (Alias for aegisctl)
    Ctl,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Khởi tạo hệ thống Observability Logging
    aegis_observability::init_logging();

    let cli = Cli::parse();

    match cli.command {
        Commands::Local { config } => run_local_daemon(config).await?,
        Commands::Server { config } => run_controller_server(config).await?,
        Commands::Agent => {
            info!("Starting AegisNode Managed Agent (Stage 2)...");
        }
        Commands::Execd => {
            info!("Starting AegisNode Execution Daemon (Stage 2)...");
        }
        Commands::Ctl => {
            let exit_code = aegis_cli::run_cli_with_args(std::env::args().collect()).await;
            std::process::exit(exit_code);
        }
    }

    Ok(())
}

/// Khởi chạy Controller Server (`aegisnode server`) cho Stage 2 Multi-Node Platform
async fn run_controller_server(config_path: PathBuf) -> Result<()> {
    info!(
        "Loading Controller Configuration from '{:?}'...",
        config_path
    );
    let config = match ControllerConfig::load_from_file(&config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            warn!(
                "Could not load controller config from '{:?}': {e}. Using default controller config...",
                config_path
            );
            ControllerConfig::default()
        }
    };

    info!("Initializing Controller Server (Stage 2 Multi-Node Platform)...");

    // Khởi tạo kết nối PostgreSQL Cluster
    let repository = match PgRepository::connect(
        &config.database.url,
        config.database.max_connections,
        config.database.connect_timeout_seconds,
    )
    .await
    {
        Ok(repo) => {
            info!(
                "Successfully connected to PostgreSQL at {}",
                config.database.url
            );
            Some(repo)
        }
        Err(e) => {
            warn!("Could not connect to PostgreSQL: {e}. Controller running in fallback mode...");
            None
        }
    };

    let controller_state = Arc::new(ControllerState {
        repository,
        config: config.clone(),
    });

    let router = create_controller_router(controller_state);
    let bind_addr = format!("{}:{}", config.server.host, config.server.port);
    info!("Listening Controller REST API on http://{}...", bind_addr);

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .map_err(|e| {
            aegis_core::AegisError::Configuration(format!(
                "Failed to bind Controller HTTP socket '{bind_addr}': {e}"
            ))
        })?;

    axum::serve(listener, router).await.map_err(|e| {
        aegis_core::AegisError::Internal(format!("Controller API Server error: {e}"))
    })?;

    Ok(())
}

/// Khởi chạy AegisNode Local Agent Daemon
async fn run_local_daemon(config_path: PathBuf) -> Result<()> {
    info!("Starting AegisNode Local Daemon...");

    // 1. Nạp tập tin cấu hình
    let config = if config_path.exists() {
        let content = tokio::fs::read_to_string(&config_path)
            .await
            .unwrap_or_default();
        AgentConfig::from_yaml(&content).unwrap_or_default()
    } else {
        warn!("Config file '{config_path:?}' not found. Using safe default configuration.");
        AgentConfig::default()
    };

    let config_arc = Arc::new(config.clone());

    // 2. Khởi tạo SQLite Storage Engine & Repositories
    info!(
        "Initializing SQLite Storage Engine at '{:?}'...",
        config.storage.database
    );
    let pool = init_sqlite_pool(&config.storage.database).await?;
    let repository = Arc::new(SqliteRepository::new(pool));

    // 3. Khởi tạo Process Runner, Capability Detector, Snapshot Manager & Backend
    let runner = Arc::new(DefaultProcessRunner::new());
    let capability_detector = Arc::new(CapabilityDetector::new(runner.clone()));

    let snapshot_dir = config
        .storage
        .database
        .parent()
        .unwrap_or_else(|| std::path::Path::new("/var/lib/aegisnode"))
        .join("snapshots");
    let snapshot_manager = Arc::new(SnapshotManager::new(&snapshot_dir, 10));

    let candidate_dir = snapshot_dir.join("candidates");
    let backend = Arc::new(NftablesRuntimeBackend::new(
        runner.clone(),
        snapshot_manager,
        candidate_dir,
    ));

    // 4. Khởi tạo SafeApplyManager
    let safe_apply_manager = Arc::new(SafeApplyManager::new(backend, runner));

    // 5. Khởi tạo AppState & Axum Router
    let app_state = Arc::new(AppState::new(
        safe_apply_manager,
        capability_detector,
        repository,
        config_arc,
    ));

    let app = create_router(app_state);

    // 6. Khởi chạy HTTP Server trên localhost
    if config.server.http.enabled {
        let bind_addr = config.server.http.bind.clone();
        info!("Listening HTTP API on http://{bind_addr}...");
        let listener = tokio::net::TcpListener::bind(&bind_addr)
            .await
            .map_err(|e| {
                aegis_core::AegisError::Configuration(format!("Failed to bind HTTP socket: {e}"))
            })?;

        axum::serve(listener, app)
            .await
            .map_err(|e| aegis_core::AegisError::Internal(format!("HTTP Server error: {e}")))?;
    }

    Ok(())
}
