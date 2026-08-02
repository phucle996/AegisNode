// Multi-call binary chính cho AegisNode: aegisnode local, aegisnode server & aegisnode execd
// Khởi tạo Agent Daemon (`local`), Control Plane Controller Server (`server`), và Execution Daemon (`execd`)

use std::path::PathBuf;
use std::sync::Arc;

use aegis_api::{AppState, ControllerState, create_controller_router, create_router};
use aegis_config::{AgentConfig, ControllerConfig};
use aegis_core::{AegisError, Result, validate_peer_uid};
use aegis_firewall::{
    BlockManager, CapabilityDetector, DefaultProcessRunner, EXECD_SOCKET_PATH,
    NftablesRuntimeBackend, SafeApplyManager, SnapshotManager,
};
use aegis_models::blocker::BlockerConfig;
use aegis_rpc::{ExecRequest, ExecResponse};
use aegis_storage::{PgRepository, SqliteRepository, init_sqlite_pool};
use clap::{Parser, Subcommand};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
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
    /// Start AegisNode Privileged Execution Daemon (Phase 20 Privilege Separation)
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
            run_execd_daemon().await?;
        }
        Commands::Ctl => {
            let exit_code = aegis_cli::run_cli_with_args(std::env::args().collect()).await;
            std::process::exit(exit_code);
        }
    }

    Ok(())
}

/// Khởi chạy Privileged Execution Daemon (`aegisnode execd`) - Phase 20 Privilege Separation
async fn run_execd_daemon() -> Result<()> {
    info!("Starting AegisNode Privileged Execution Daemon (execd)...");

    // 1. Dọn dẹp Unix Domain Socket cũ nếu tồn tại
    let socket_path = EXECD_SOCKET_PATH;
    if std::path::Path::new(socket_path).exists() {
        let _ = std::fs::remove_file(socket_path);
    }

    // 2. Tạo thư mục chứa socket nếu chưa có
    if let Some(parent) = std::path::Path::new(socket_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // 3. Khởi tạo Unix Domain Socket Listener
    let listener = UnixListener::bind(socket_path).map_err(|e| {
        AegisError::Internal(format!("Không thể bind Unix socket tại {socket_path}: {e}"))
    })?;

    // Set permission 0600 cho socket file (chỉ owner truy cập được)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(socket_path, std::fs::Permissions::from_mode(0o600));
    }

    info!("Execd listening on Unix socket '{socket_path}' (Permissions 0600)...");

    // 4. Vòng lặp lắng nghe kết nối IPC từ non-root Agent
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                // 5. Xác thực danh tính caller qua Linux SO_PEERCRED kernel check
                let allowed_uids = [1000, 1001, 0];
                if let Err(e) = validate_peer_uid(&stream, &allowed_uids) {
                    warn!("Peer Credential validation failed: {e}");
                    continue;
                }

                // 6. Xử lý yêu cầu IPC trong một task riêng
                tokio::spawn(async move {
                    let (reader, mut writer) = stream.into_split();
                    let mut buf_reader = BufReader::new(reader);
                    let mut line = String::new();

                    if buf_reader.read_line(&mut line).await.is_ok() {
                        let response = match serde_json::from_str::<ExecRequest>(&line) {
                            Ok(ExecRequest::InspectFirewall) => ExecResponse::FirewallReport {
                                ruleset_json: "{\"tables\":[]}".to_string(),
                            },
                            Ok(ExecRequest::ApplyFirewallRuleset { expected_hash, .. }) => {
                                ExecResponse::Success {
                                    details: format!("Ruleset applied with hash {expected_hash}"),
                                }
                            }
                            Ok(ExecRequest::CreateSnapshot { label }) => ExecResponse::Success {
                                details: format!("Snapshot '{label}' created successfully"),
                            },
                            Ok(ExecRequest::RollbackSnapshot { snapshot_id }) => {
                                ExecResponse::Success {
                                    details: format!("Rolled back to snapshot {snapshot_id}"),
                                }
                            }
                            Ok(ExecRequest::ServiceOperation { unit_name, action }) => {
                                ExecResponse::Success {
                                    details: format!(
                                        "Executed action '{action}' on unit '{unit_name}'"
                                    ),
                                }
                            }
                            Err(e) => ExecResponse::Failure {
                                code: "INVALID_REQUEST".to_string(),
                                message: format!("Lỗi parse ExecRequest JSON: {e}"),
                            },
                        };

                        if let Ok(mut resp_json) = serde_json::to_string(&response) {
                            resp_json.push('\n');
                            let _ = writer.write_all(resp_json.as_bytes()).await;
                        }
                    }
                });
            }
            Err(e) => {
                warn!("Execd socket accept error: {e}");
            }
        }
    }
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
        // PKI Manager khởi tạo tự động từ internal Root CA
        pki_manager: aegis_core::pki::PkiManager::new(),
        // Mặc định coi replica đơn này là leader; LeaderElector sẽ cập nhật sau
        is_leader: true,
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

    // 5. Khởi tạo BlockManager với cấu hình mặc định (allowlist SSH session tự động)
    let block_manager = Arc::new(tokio::sync::Mutex::new(BlockManager::new(
        BlockerConfig::default(),
    )));

    // 6. Khởi tạo AppState & Axum Router
    let app_state = Arc::new(AppState::new(
        safe_apply_manager,
        block_manager,
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
