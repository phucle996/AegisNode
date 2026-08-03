// Docker Inspector & Public Exposure Analyzer cho AegisNode
// Kết nối Docker Engine, phát hiện container inventory, published ports ra 0.0.0.0 WAN thực tế từ Linux OS Host

use std::path::PathBuf;
use std::process::Command;

use aegis_core::Result;
use aegis_models::docker::{
    ContainerLabelPolicy, DockerContainer, PublishedPort,
};
use aegis_models::firewall::TransportProtocol;
use serde::{Deserialize, Serialize};

/// Cảnh báo phơi nhiễm cổng public của container ra ngoài Internet
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExposureWarning {
    pub container_id: String,
    pub container_name: String,
    pub published_port: PublishedPort,
    pub is_database: bool,
    pub warning_message: String,
}

/// Báo cáo phân tích rủi ro phơi nhiễm cổng của các Docker Containers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerExposureReport {
    pub docker_available: bool,
    pub containers: Vec<DockerContainer>,
    pub public_exposures: Vec<ExposureWarning>,
    pub label_policies: Vec<ContainerLabelPolicy>,
}

/// Trình kiểm định và phân tích Docker Containers
pub struct DockerInspector {
    socket_path: PathBuf,
}

impl DockerInspector {
    pub fn new(socket_path: impl Into<PathBuf>) -> Self {
        Self {
            socket_path: socket_path.into(),
        }
    }

    pub fn default_prod() -> Self {
        Self::new("/var/run/docker.sock")
    }

    /// Thực hiện kiểm tra inventory và phân tích các rủi ro phơi nhiễm cổng
    pub async fn inspect(&self) -> Result<DockerExposureReport> {
        let mut report = DockerExposureReport {
            docker_available: false,
            containers: Vec::new(),
            public_exposures: Vec::new(),
            label_policies: Vec::new(),
        };

        // 1. Kiểm tra xem Docker Daemon Socket hoặc Docker CLI có tồn tại dưới OS hay không
        let socket_exists = self.socket_path.exists();
        let check_cli = Command::new("sudo").args(["docker", "info"]).output();
        let docker_active = socket_exists || check_cli.map(|o| o.status.success()).unwrap_or(false);

        if !docker_active {
            // Trả về báo cáo docker_available = false nếu máy chủ chưa cài Docker Engine
            return Ok(report);
        }

        report.docker_available = true;

        // 2. Chạy lệnh `docker ps --all --format '{{json .}}'` đọc danh sách container thực tế từ OS Host
        let output = Command::new("sudo")
            .args(["docker", "ps", "--all", "--format", "{{json .}}"])
            .output()
            .or_else(|_| Command::new("docker").args(["ps", "--all", "--format", "{{json .}}"]).output());

        if let Ok(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                for line in stdout.lines() {
                    let line_str = line.trim();
                    if line_str.is_empty() {
                        continue;
                    }

                    // Giải mã JSON từ docker ps CLI output
                    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(line_str) {
                        let id = json_val.get("ID").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                        let name = json_val.get("Names").and_then(|v| v.as_str()).unwrap_or("unnamed").to_string();
                        let image = json_val.get("Image").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                        let state = json_val.get("State").and_then(|v| v.as_str()).unwrap_or("running").to_string();
                        let ports_raw = json_val.get("Ports").and_then(|v| v.as_str()).unwrap_or("");
                        let networks_raw = json_val.get("Networks").and_then(|v| v.as_str()).unwrap_or("bridge");

                        let mut published_ports = Vec::new();

                        // Phân tích chuỗi cổng mapped thực tế (VD: "0.0.0.0:80->80/tcp, :::80->80/tcp")
                        for p in ports_raw.split(',') {
                            let p_str = p.trim();
                            if p_str.contains("->") {
                                if let Some((host_part, container_part)) = p_str.split_once("->") {
                                    let host_ip_port = host_part.trim();
                                    let (host_ip, host_port_str) = if let Some((ip, port)) = host_ip_port.rsplit_once(':') {
                                        (ip.to_string(), port.to_string())
                                    } else {
                                        ("0.0.0.0".to_string(), host_ip_port.to_string())
                                    };

                                    let (container_port_str, proto_str) = if let Some((cport, proto)) = container_part.trim().split_once('/') {
                                        (cport.to_string(), proto.to_string())
                                    } else {
                                        (container_part.trim().to_string(), "tcp".to_string())
                                    };

                                    let h_port = host_port_str.parse::<u16>().unwrap_or(0);
                                    let c_port = container_port_str.parse::<u16>().unwrap_or(0);
                                    let protocol = if proto_str.to_lowercase() == "udp" {
                                        TransportProtocol::Udp
                                    } else {
                                        TransportProtocol::Tcp
                                    };

                                    if h_port > 0 {
                                        let pub_port = PublishedPort {
                                            host_ip: host_ip.clone(),
                                            host_port: h_port,
                                            container_port: c_port,
                                            protocol,
                                        };

                                        // Kiểm tra nguy cơ phơi nhiễm cổng 0.0.0.0 công khai
                                        if host_ip == "0.0.0.0" || host_ip == "::" {
                                            let is_db = matches!(h_port, 5432 | 3306 | 6379 | 27017 | 9200 | 1433);
                                            let msg = if is_db {
                                                format!("Cơ sở dữ liệu {image} (Port {h_port}) phơi nhiễm công khai ra ngoài 0.0.0.0 WAN!")
                                            } else {
                                                format!("Cổng {h_port} của container {name} đang phơi nhiễm công khai trên 0.0.0.0 WAN.")
                                            };

                                            report.public_exposures.push(ExposureWarning {
                                                container_id: id.clone(),
                                                container_name: name.clone(),
                                                published_port: pub_port.clone(),
                                                is_database: is_db,
                                                warning_message: msg,
                                            });
                                        }

                                        published_ports.push(pub_port);
                                    }
                                }
                            }
                        }

                        let container = DockerContainer {
                            id,
                            name,
                            image,
                            state,
                            cpu_perc: None,
                            mem_usage: None,
                            networks: vec![networks_raw.to_string()],
                            published_ports,
                            labels: std::collections::HashMap::new(),
                        };

                        report.containers.push(container);
                    }
                }
            }
        }

        // 3. Đọc chỉ số tài nguyên thực tế (CPU & Memory) bằng `docker stats --no-stream --format '{{json .}}'`
        if !report.containers.is_empty() {
            let stats_output = Command::new("sudo")
                .args(["docker", "stats", "--no-stream", "--format", "{{json .}}"])
                .output()
                .or_else(|_| Command::new("docker").args(["stats", "--no-stream", "--format", "{{json .}}"]).output());

            if let Ok(out) = stats_output {
                if out.status.success() {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let mut stats_map: std::collections::HashMap<String, (String, String)> = std::collections::HashMap::new();

                    for line in stdout.lines() {
                        let line_str = line.trim();
                        if line_str.is_empty() {
                            continue;
                        }
                        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(line_str) {
                            let cid = json_val.get("ID").or_else(|| json_val.get("Container")).and_then(|v| v.as_str()).unwrap_or("");
                            let cname = json_val.get("Name").and_then(|v| v.as_str()).unwrap_or("");
                            let cpu = json_val.get("CPUPerc").and_then(|v| v.as_str()).unwrap_or("0.00%").to_string();
                            let mem_u = json_val.get("MemUsage").and_then(|v| v.as_str()).unwrap_or("0B / 0B");
                            let mem_p = json_val.get("MemPerc").and_then(|v| v.as_str()).unwrap_or("0.00%");

                            let mem_full = format!("{mem_u} ({mem_p})");

                            if !cid.is_empty() {
                                stats_map.insert(cid.to_string(), (cpu.clone(), mem_full.clone()));
                            }
                            if !cname.is_empty() {
                                stats_map.insert(cname.to_string(), (cpu, mem_full));
                            }
                        }
                    }

                    // Khớp thông số CPU / RAM vào từng Container thực tế
                    for c in &mut report.containers {
                        if let Some((cpu, mem)) = stats_map.get(&c.id).or_else(|| stats_map.get(&c.name)).or_else(|| {
                            let short_id = if c.id.len() >= 12 { &c.id[..12] } else { &c.id };
                            stats_map.get(short_id)
                        }) {
                            c.cpu_perc = Some(cpu.clone());
                            c.mem_usage = Some(mem.clone());
                        } else {
                            c.cpu_perc = Some("0.00%".to_string());
                            c.mem_usage = Some("0.00 B / 0.00 B (0.00%)".to_string());
                        }
                    }
                }
            }
        }

        Ok(report)
    }
}
