// Integration tests cho BlockManager (Dynamic Allowlist & Active Admin IP detection) và SshDetector

use aegis_firewall::{BlockManager, SshDetector};
use aegis_models::blocker::BlockerConfig;
use aegis_models::firewall::CidrSpec;

#[test]
fn test_block_manager_allowlist_protection() {
    let mut config = BlockerConfig::default();
    config.allowlist.push(CidrSpec("10.50.0.0/16".to_string()));

    let mut mgr = BlockManager::new(config);

    // 1. Block IP hợp lệ không thuộc allowlist
    let res = mgr.add_block("203.0.113.100", Some(1800), "suspicious activity", "admin");
    assert!(res.is_ok());
    let entry = res.unwrap();
    assert_eq!(entry.ip, "203.0.113.100");

    // 2. Thử block IP thuộc Loopback (127.0.0.1) -> phải bị từ chối
    let res_loopback = mgr.add_block("127.0.0.1", Some(1800), "loopback block", "admin");
    assert!(res_loopback.is_err());

    // 3. Thử block IP thuộc Explicit Allowlist (10.50.1.2) -> phải bị từ chối
    let res_explicit = mgr.add_block("10.50.1.2", Some(1800), "explicit allowlist block", "admin");
    assert!(res_explicit.is_err());

    // 4. Thử block IP thuộc môi trường SSH_CLIENT giả lập (Admin active session) -> phải bị từ chối
    unsafe {
        std::env::set_var("SSH_CLIENT", "192.168.1.100 54321 22");
    }
    let res_admin_session =
        mgr.add_block("192.168.1.100", Some(1800), "admin session block", "admin");
    assert!(res_admin_session.is_err());
    unsafe {
        std::env::remove_var("SSH_CLIENT");
    }
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
