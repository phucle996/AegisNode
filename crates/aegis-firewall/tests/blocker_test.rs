// Integration tests cho BlockManager (Allowlist protection) và SshDetector (Sliding Window Threshold Engine)

use aegis_firewall::{BlockManager, SshDetector};
use aegis_models::blocker::BlockerConfig;

#[test]
fn test_block_manager_allowlist_protection() {
    let mut mgr = BlockManager::new(BlockerConfig::default());

    // 1. Block IP hợp lệ
    let res = mgr.add_block("203.0.113.100", Some(1800), "suspicious activity", "admin");
    assert!(res.is_ok());
    let entry = res.unwrap();
    assert_eq!(entry.ip, "203.0.113.100");

    // 2. Thử block IP thuộc Allowlist (127.0.0.1) -> phải bị từ chối
    let res_loopback = mgr.add_block("127.0.0.1", Some(1800), "loopback block", "admin");
    assert!(res_loopback.is_err());

    // 3. Thử block IP thuộc Management CIDR (10.1.2.3) -> phải bị từ chối
    let res_mgmt = mgr.add_block("10.1.2.3", Some(1800), "mgmt block", "admin");
    assert!(res_mgmt.is_err());
}

#[test]
fn test_ssh_detector_sliding_window_trigger() {
    let mut mgr = BlockManager::new(BlockerConfig::default());
    let mut detector = SshDetector::new(5, 60, 1800); // Threshold: 5 failures in 60s

    let attacker_ip = "198.51.100.42";

    // Simulates 4 failures -> not triggered yet
    for _ in 0..4 {
        let res = detector.record_failure(attacker_ip, &mut mgr).unwrap();
        assert!(res.is_none());
    }

    // 5th failure -> triggers automatic block
    let res = detector.record_failure(attacker_ip, &mut mgr).unwrap();
    assert!(res.is_some());
    let block = res.unwrap();
    assert_eq!(block.ip, attacker_ip);
    assert_eq!(block.actor, "ssh_detector");
}
