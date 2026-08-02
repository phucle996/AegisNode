// Integration tests cho DockerInspector (Graceful Degradation) và RouterManager (Sysctl Management) trong aegis-firewall

use aegis_firewall::{DockerInspector, RouterManager};

#[tokio::test]
async fn test_docker_inspector_graceful_degradation() {
    let temp_socket =
        std::env::temp_dir().join(format!("non_existent_docker_{}.sock", uuid::Uuid::new_v4()));
    let inspector = DockerInspector::new(&temp_socket);

    let report = inspector.inspect().await.expect("Inspect must not panic");
    assert!(!report.docker_available);
    assert!(report.containers.is_empty());
    assert!(report.public_exposures.is_empty());
}

#[tokio::test]
async fn test_router_manager_sysctl() {
    let snapshot = RouterManager::read_sysctl_forwarding().await;
    assert!(snapshot.is_ok());

    let snap_val = snapshot.unwrap();
    assert!(snap_val.old_ipv4_forward == "0" || snap_val.old_ipv4_forward == "1");

    let restore_res = RouterManager::restore_sysctl(&snap_val).await;
    assert!(restore_res.is_ok());
}
