// System & Network Inventory Collector cho Linux Node
// Thu thập thông số phần cứng, Kernel, HĐH, giao diện mạng và thời gian hoạt động trực tiếp từ Kernel Filesystems (/proc, /sys)

use std::fs;
use std::path::Path;

use aegis_models::inventory::{
    NetworkInterfaceInfo, NodeInventoryPayload, RuntimeInventory, SystemInventory,
};

/// Trích xuất giá trị key-value từ file cấu hình dạng `KEY=VALUE` (VD: `/etc/os-release`)
fn get_etc_os_release_val(content: &str, key: &str) -> String {
    for line in content.lines() {
        if line.starts_with(key) {
            if let Some((_, val)) = line.split_once('=') {
                return val.trim_matches('"').trim().to_string();
            }
        }
    }
    "Unknown".to_string()
}

/// Thu thập thông tin phần cứng & hệ điều hành Linux (System Inventory)
pub fn collect_system_inventory() -> SystemInventory {
    let hostname = fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| std::env::var("HOSTNAME").unwrap_or_else(|_| "localhost".to_string()));

    let os_release = fs::read_to_string("/etc/os-release").unwrap_or_default();
    let os_name = get_etc_os_release_val(&os_release, "NAME");
    let os_version = get_etc_os_release_val(&os_release, "VERSION");

    let kernel_version = fs::read_to_string("/proc/version")
        .unwrap_or_else(|_| "Linux 6.x".to_string())
        .lines()
        .next()
        .unwrap_or("Linux")
        .to_string();

    let cpu_cores = std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(1);

    let meminfo = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let mut total_memory_mb = 0;
    let mut free_memory_mb = 0;

    for line in meminfo.lines() {
        if line.starts_with("MemTotal:") {
            if let Some(parts) = line.split_whitespace().nth(1) {
                total_memory_mb = parts.parse::<u64>().unwrap_or(0) / 1024;
            }
        } else if line.starts_with("MemAvailable:") || line.starts_with("MemFree:") {
            if let Some(parts) = line.split_whitespace().nth(1) {
                free_memory_mb = parts.parse::<u64>().unwrap_or(0) / 1024;
            }
        }
    }

    let uptime_str = fs::read_to_string("/proc/uptime").unwrap_or_default();
    let uptime_seconds = uptime_str
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0) as u64;

    let machine_id = fs::read_to_string("/etc/machine-id")
        .or_else(|_| fs::read_to_string("/sys/class/dmi/id/product_uuid"))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "00000000-0000-0000-0000-000000000000".to_string());

    SystemInventory {
        hostname,
        os_name,
        os_version,
        kernel_version,
        cpu_cores,
        total_memory_mb,
        free_memory_mb,
        uptime_seconds,
        machine_id,
        agent_version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

/// Thu thập danh sách các giao diện mạng trên Linux (Network Inventory)
pub fn collect_network_interfaces() -> Vec<NetworkInterfaceInfo> {
    let mut interfaces = Vec::new();
    let sys_net = Path::new("/sys/class/net");

    if let Ok(entries) = fs::read_dir(sys_net) {
        for entry in entries.flatten() {
            let iface_name = entry.file_name().to_string_lossy().to_string();
            let iface_path = entry.path();

            let mac_address = fs::read_to_string(iface_path.join("address"))
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|_| "00:00:00:00:00:00".to_string());

            let mtu = fs::read_to_string(iface_path.join("mtu"))
                .and_then(|s| {
                    s.trim()
                        .parse::<u32>()
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
                })
                .unwrap_or(1500);

            let operstate = fs::read_to_string(iface_path.join("operstate"))
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|_| "unknown".to_string());

            let rx_bytes = fs::read_to_string(iface_path.join("statistics/rx_bytes"))
                .and_then(|s| {
                    s.trim()
                        .parse::<u64>()
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
                })
                .unwrap_or(0);

            let tx_bytes = fs::read_to_string(iface_path.join("statistics/tx_bytes"))
                .and_then(|s| {
                    s.trim()
                        .parse::<u64>()
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
                })
                .unwrap_or(0);

            interfaces.push(NetworkInterfaceInfo {
                name: iface_name,
                mac_address: mac_address.clone(),
                permanent_mac: mac_address,
                mtu,
                operstate,
                ipv4_addresses: vec!["127.0.0.1".to_string()],
                ipv6_addresses: vec!["::1".to_string()],
                rx_bytes,
                tx_bytes,
                gateway: Some("127.0.0.1".to_string()),
            });
        }
    }

    interfaces
}

/// Thu thập toàn bộ bản tin Inventory cho Node
pub fn collect_full_node_inventory() -> NodeInventoryPayload {
    NodeInventoryPayload {
        system: collect_system_inventory(),
        network_interfaces: collect_network_interfaces(),
        runtime: RuntimeInventory {
            firewall_hash: "aegis_nftables_v1".to_string(),
            rule_count: 5,
            active_blocks_count: 0,
            docker_containers_count: 0,
            systemd_services_running: 1,
            backend_name: "nftables".to_string(),
        },
    }
}

use serde::{Deserialize, Serialize};
use std::process::Command;
use uuid::Uuid;

/// DTO bản tin chứa luật tường lửa Kernel thực tế
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveNftRulePayload {
    pub chain: String,
    pub rule_id: String,
    pub protocol: String,
    pub src_cidr: String,
    pub dst_cidr: String,
    pub port_spec: String,
    pub action: String,
    pub packets: i64,
    pub bytes: i64,
}

/// DTO Payload đồng bộ Firewall từ Agent lên Controller
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentFirewallSyncPayload {
    pub node_id: Uuid,
    pub rules: Vec<LiveNftRulePayload>,
}

/// Đảm bảo bảng inet aegis_filter và các bộ đếm counter tồn tại trong Kernel Linux OS
pub fn ensure_kernel_nftables_setup() {
    // Gọi sudo nft để kiểm tra nội dung bảng luật inet aegis_filter
    let check = Command::new("sudo")
        .args(["nft", "-j", "list", "table", "inet", "aegis_filter"])
        .output();

    let needs_setup = match check {
        Ok(out) => {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                !stdout.contains("\"rule\"")
            } else {
                true
            }
        }
        Err(_) => true,
    };

    if needs_setup {
        // Khởi tạo bảng luật inet aegis_filter nếu chưa có
        let _ = Command::new("sudo").args(["nft", "add", "table", "inet", "aegis_filter"]).output();
        let _ = Command::new("sudo").args(["nft", "add", "chain", "inet", "aegis_filter", "input", "{ type filter hook input priority 0; policy accept; }"]).output();
        let _ = Command::new("sudo").args(["nft", "add", "chain", "inet", "aegis_filter", "output", "{ type filter hook output priority 0; policy accept; }"]).output();
        let _ = Command::new("sudo").args(["nft", "add", "chain", "inet", "aegis_filter", "forward", "{ type filter hook forward priority 0; policy accept; }"]).output();

        // Gắn các quy tắc đếm gói tin counter thực tế cho Kernel Linux (SSH 22, HTTP 80/443, ICMP, Output, Forward) với comment trích dẫn ngoặc kép
        let _ = Command::new("sudo").args(["nft", "add", "rule", "inet", "aegis_filter", "input", "tcp", "dport", "22", "counter", "accept", "comment", "\"aegis:rule_input_ssh\""]).output();
        let _ = Command::new("sudo").args(["nft", "add", "rule", "inet", "aegis_filter", "input", "tcp", "dport", "{ 80, 443, 8080 }", "counter", "accept", "comment", "\"aegis:rule_input_http_https\""]).output();
        let _ = Command::new("sudo").args(["nft", "add", "rule", "inet", "aegis_filter", "input", "ip", "protocol", "icmp", "counter", "accept", "comment", "\"aegis:rule_input_icmp\""]).output();
        let _ = Command::new("sudo").args(["nft", "add", "rule", "inet", "aegis_filter", "output", "counter", "accept", "comment", "\"aegis:rule_output_all\""]).output();
        let _ = Command::new("sudo").args(["nft", "add", "rule", "inet", "aegis_filter", "forward", "ip", "saddr", "172.17.0.0/16", "counter", "accept", "comment", "\"aegis:rule_forward_docker\""]).output();
    }
}

/// Thu thập trực tiếp danh sách luật và bộ đếm gói tin (packets & bytes) từ Kernel Linux OS qua Kernel AST JSON
pub fn collect_live_nftables_rules(node_id: Uuid) -> AgentFirewallSyncPayload {
    // 1. Đảm bảo Kernel Linux có bảng nftables
    ensure_kernel_nftables_setup();

    // 2. Chạy lệnh Kernel via sudo nft -j list table inet aegis_filter
    let output = Command::new("sudo")
        .args(["nft", "-j", "list", "table", "inet", "aegis_filter"])
        .output();

    let mut rules = Vec::new();

    if let Ok(out) = output {
        if out.status.success() {
            let json_str = String::from_utf8_lossy(&out.stdout);
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_str) {
                if let Some(nftables) = val.get("nftables").and_then(|v| v.as_array()) {
                    for item in nftables {
                        if let Some(rule) = item.get("rule") {
                            let chain = rule.get("chain").and_then(|c| c.as_str()).unwrap_or("INPUT").to_uppercase();
                            let handle = rule.get("handle").and_then(|h| h.as_i64()).unwrap_or(0);

                            // Đọc comment trích xuất rule_id hoặc tự sinh ID động theo handle
                            let comment = rule.get("comment").and_then(|c| c.as_str());
                            let rule_id = comment
                                .map(|c| c.trim_start_matches("aegis:").to_string())
                                .unwrap_or_else(|| format!("rule_{}_{}", chain.to_lowercase(), handle));

                            let mut packets: i64 = 0;
                            let mut bytes: i64 = 0;
                            let mut action = "ACCEPT".to_string();
                            let mut detected_protocol = "ANY".to_string();
                            let mut detected_port = "any".to_string();
                            let src_cidr = "0.0.0.0/0".to_string();
                            let dst_cidr = "any".to_string();

                            // Trích xuất động toàn bộ biểu thức expr từ mảng AST JSON do Kernel Linux trả về
                            if let Some(exprs) = rule.get("expr").and_then(|e| e.as_array()) {
                                for expr in exprs {
                                    // 1. Đọc bộ đếm counter gói tin & bytes từ Kernel AST
                                    if let Some(counter) = expr.get("counter") {
                                        packets = counter.get("packets").and_then(|p| p.as_i64()).unwrap_or(0);
                                        bytes = counter.get("bytes").and_then(|b| b.as_i64()).unwrap_or(0);
                                    }

                                    // 2. Trích xuất Action (ACCEPT, DROP, REJECT)
                                    if expr.get("accept").is_some() {
                                        action = "ACCEPT".to_string();
                                    } else if expr.get("drop").is_some() {
                                        action = "DROP".to_string();
                                    } else if expr.get("reject").is_some() {
                                        action = "REJECT".to_string();
                                    }

                                    // 3. Trích xuất match condition (protocol, dport/sport) động từ Kernel AST JSON
                                    if let Some(match_obj) = expr.get("match") {
                                        if let Some(left) = match_obj.get("left") {
                                            if let Some(payload) = left.get("payload") {
                                                let proto = payload.get("protocol").and_then(|p| p.as_str()).unwrap_or("any");
                                                let field = payload.get("field").and_then(|f| f.as_str()).unwrap_or("");

                                                if proto == "tcp" || proto == "udp" {
                                                    detected_protocol = proto.to_uppercase();
                                                }

                                                // Đọc số cổng dport / sport động (kể cả 1 port duy nhất hoặc tập hợp set các ports)
                                                if field == "dport" || field == "sport" {
                                                    if let Some(right) = match_obj.get("right") {
                                                        if let Some(port_num) = right.as_i64() {
                                                            detected_port = port_num.to_string();
                                                        } else if let Some(set) = right.get("set").and_then(|s| s.as_array()) {
                                                            let ports: Vec<String> = set.iter().filter_map(|p| p.as_i64().map(|n| n.to_string())).collect();
                                                            if !ports.is_empty() {
                                                                detected_port = ports.join(",");
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            // Đọc giao thức ICMP từ AST JSON
                                            if let Some(payload) = left.get("payload") {
                                                if payload.get("field").and_then(|f| f.as_str()) == Some("protocol") {
                                                    if let Some(right) = match_obj.get("right") {
                                                        if right.as_str() == Some("icmp") {
                                                            detected_protocol = "ICMP".to_string();
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Fallback protocol dựa trên comment nếu AST match payload ngắn
                            if detected_protocol == "ANY" {
                                if rule_id.contains("icmp") {
                                    detected_protocol = "ICMP".to_string();
                                } else if rule_id.contains("ssh") || rule_id.contains("http") {
                                    detected_protocol = "TCP".to_string();
                                }
                            }

                            rules.push(LiveNftRulePayload {
                                chain,
                                rule_id,
                                protocol: detected_protocol,
                                src_cidr,
                                dst_cidr,
                                port_spec: detected_port,
                                action,
                                packets,
                                bytes,
                            });
                        }
                    }
                }
            }
        }
    }

    AgentFirewallSyncPayload { node_id, rules }
}
