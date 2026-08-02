// Integration tests cho AegisNode API Crate và Axum Web Router

use std::sync::Arc;

use aegis_api::{AppState, create_router};
use aegis_config::AgentConfig;
use aegis_firewall::{
    BlockManager, CapabilityDetector, MockProcessRunner, NftablesRuntimeBackend, SafeApplyManager,
    SnapshotManager,
};
use aegis_models::blocker::BlockerConfig;
use aegis_models::firewall::FirewallPolicy;
use aegis_storage::{SqliteRepository, init_in_memory_pool};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn test_api_status_and_validate_endpoints() {
    let pool = init_in_memory_pool().await.unwrap();
    let repo = Arc::new(SqliteRepository::new(pool));

    let runner = Arc::new(MockProcessRunner::new());
    let cap_detector = Arc::new(CapabilityDetector::new(runner.clone()));

    let temp_dir = std::env::temp_dir().join(format!("aegis_api_snap_{}", uuid::Uuid::new_v4()));
    let snap_mgr = Arc::new(SnapshotManager::new(&temp_dir, 5));
    let backend = Arc::new(NftablesRuntimeBackend::new(
        runner.clone(),
        snap_mgr,
        temp_dir.join("cand"),
    ));
    let safe_mgr = Arc::new(SafeApplyManager::new(backend, runner));

    let config = Arc::new(AgentConfig::default());
    // Khởi tạo BlockManager với cấu hình mặc định cho test environment
    let block_manager = Arc::new(tokio::sync::Mutex::new(BlockManager::new(
        BlockerConfig::default(),
    )));
    let state = Arc::new(AppState::new(
        safe_mgr,
        block_manager,
        cap_detector,
        repo,
        config,
    ));

    let app = create_router(state);

    // 1. Test GET /v1/status
    let req = Request::builder()
        .uri("/v1/status")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 2. Test POST /v1/firewall/validate
    let yaml_str = include_str!("../../../tests/fixtures/policies/minimal.yaml");
    let policy: FirewallPolicy = serde_yaml::from_str(yaml_str).unwrap();
    let body_json = serde_json::to_string(&policy).unwrap();

    let req_val = Request::builder()
        .uri("/v1/firewall/validate")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(body_json))
        .unwrap();

    let res_val = app.oneshot(req_val).await.unwrap();
    assert_eq!(res_val.status(), StatusCode::OK);

    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
}
