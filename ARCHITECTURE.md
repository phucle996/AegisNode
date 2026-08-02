# AegisNode

> Lightweight Linux Firewall, Network and Service Management Platform written in Rust.

AegisNode là nền tảng quản lý node Linux tập trung vào ba nhóm chức năng:

* Firewall và NAT dựa trên nftables.
* Network interface, route, VLAN và bridge.
* Systemd service và journald.

Hệ thống cung cấp cả CLI và Web UI, hỗ trợ vận hành một node ở MVP và mở rộng thành nền tảng quản lý nhiều node ở Middle và Production Ready.

---

# 1. Product Vision

AegisNode cung cấp một lớp quản lý an toàn phía trên các thành phần Linux tiêu chuẩn:

```text
AegisNode
├── nftables
├── NetworkManager
├── systemd-networkd
├── systemd
├── journald
└── Docker Engine
```

AegisNode không thay thế Linux networking stack.

Rust chịu trách nhiệm:

```text
Desired state
    ↓
Validate
    ↓
Build change plan
    ↓
Snapshot
    ↓
Apply
    ↓
Health check
    ↓
Commit hoặc rollback
```

Linux kernel và các Linux subsystem tiếp tục chịu trách nhiệm xử lý network thực tế.

---

# 2. Core Principles

## 2.1 Không nằm trên data path

Packet không đi qua AegisNode daemon.

```text
Network packet
      ↓
Linux kernel
      ↓
nftables
      ↓
Application / Container / LAN
```

AegisNode chỉ cập nhật configuration và runtime state.

## 2.2 Desired state và runtime state tách biệt

```text
Desired State
Policy file / Controller database
              ↓
          Reconcile
              ↓
Runtime State
nftables / interfaces / systemd
```

## 2.3 Local autonomy

Mỗi node phải tiếp tục vận hành khi controller mất kết nối.

Node lưu:

* Current policy.
* Last-known-good policy.
* Network snapshot.
* Local blocklist.
* Apply history.
* Pending change plan.

## 2.4 Safe apply

Mọi thay đổi nguy hiểm phải có:

* Validation.
* Dry-run.
* Snapshot.
* Rollback timer.
* Health check.
* Explicit confirmation.

## 2.5 Không thực thi shell tùy ý

Frontend và controller không được gửi shell command xuống node.

Sai:

```text
systemctl restart nginx
nft flush ruleset
ip addr flush dev eth0
```

Đúng:

```json
{
  "operation": "service.restart",
  "unit": "nginx.service"
}
```

Privileged executor chỉ nhận các operation đã được định nghĩa bằng schema.

---

# 3. Product Stages

## Stage 1 — MVP

Một node Linux chạy độc lập.

```text
Web UI
   │
CLI
   │
   ▼
AegisNode Daemon
   │
   ├── nftables
   ├── Docker
   ├── systemd
   └── SQLite
```

Chức năng:

* Input, output và forward firewall.
* NAT và port forwarding.
* Docker container discovery.
* Temporary IP block.
* Auto-block SSH cơ bản.
* CLI local.
* Web UI local.
* Safe apply và rollback.
* SQLite local storage.

## Stage 2 — Middle

Quản lý nhiều node Linux.

```text
Web UI / Remote CLI
          │
          ▼
Aegis Controller
          │
         mTLS
          │
          ▼
Multiple Aegis Agents
```

Bổ sung:

* Central controller.
* Multi-node inventory.
* Network interface management.
* Static IP, DHCP, DNS và routes.
* WAN, LAN và Management roles.
* VLAN và bridge.
* Systemd service management.
* Journald log viewer.
* Combined change plan.
* Canary rollout.
* PostgreSQL trên controller.

## Stage 3 — Production Ready

Bổ sung:

* HA controller.
* OIDC.
* RBAC.
* Policy approval.
* Signed policy bundle.
* Privilege separation.
* Progressive rollout.
* Audit trail.
* Backup và disaster recovery.
* Bond, VRF và policy routing.
* HA gateway.

---

# 4. High-Level Architecture

```text
                       Management Plane

              ┌─────────────────────────┐
              │        Web UI           │
              └────────────┬────────────┘
                           │ HTTPS
              ┌────────────▼────────────┐
              │    Aegis Controller     │
              │                         │
              │ API                     │
              │ Authentication          │
              │ Node Inventory          │
              │ Policy Management       │
              │ Change Plan Builder     │
              │ Rollout Coordinator     │
              │ Audit                   │
              └────────────┬────────────┘
                           │ mTLS
             ┌─────────────┼─────────────┐
             │             │             │
             ▼             ▼             ▼

       ┌───────────┐ ┌───────────┐ ┌───────────┐
       │ Linux     │ │ Docker    │ │ Router    │
       │ Server    │ │ Host      │ │ Node      │
       │           │ │           │ │           │
       │ Agent     │ │ Agent     │ │ Agent     │
       │ Executor  │ │ Executor  │ │ Executor  │
       │ nftables  │ │ nftables  │ │ nftables  │
       └───────────┘ └───────────┘ └───────────┘
```

---

# 5. Node Architecture

```text
┌─────────────────────────────────────────────────────┐
│                    Linux Node                       │
│                                                     │
│  ┌───────────────────────────────────────────────┐  │
│  │ aegis-agent                                   │  │
│  │                                               │  │
│  │ Controller connection                         │  │
│  │ Desired-state reconciliation                  │  │
│  │ Policy validation                             │  │
│  │ Node inventory                                │  │
│  │ Docker discovery                              │  │
│  │ Health checks                                 │  │
│  │ Metrics and audit                             │  │
│  └───────────────────────┬───────────────────────┘  │
│                          │ Unix socket              │
│                          ▼                          │
│  ┌───────────────────────────────────────────────┐  │
│  │ aegis-execd                                   │  │
│  │ Privileged execution helper                   │  │
│  │                                               │  │
│  │ Firewall Executor                             │  │
│  │ Network Executor                              │  │
│  │ Systemd Executor                              │  │
│  └───────┬────────────────┬────────────────┬─────┘  │
│          │                │                │        │
│          ▼                ▼                ▼        │
│      nftables       NetworkManager      systemd     │
│                    systemd-networkd      D-Bus      │
│                                                     │
└─────────────────────────────────────────────────────┘
```

Trong MVP, `aegis-agent` và `aegis-execd` có thể được gộp thành một process `aegisd`.

Từ Middle trở đi nên tách riêng hai process.

---

# 6. Main Components

## 6.1 aegisctl

CLI cho local và remote operation.

Local mode:

```text
aegisctl
    ↓
Unix socket
    ↓
aegis-agent
```

Remote mode:

```text
aegisctl
    ↓ HTTPS
Aegis Controller
```

Các command chính:

```bash
aegisctl status

aegisctl firewall rule list
aegisctl firewall policy check
aegisctl firewall policy apply

aegisctl network interface list
aegisctl network profile apply

aegisctl service list
aegisctl service restart nginx.service

aegisctl block add 203.0.113.20 --duration 30m

aegisctl change-plan inspect
aegisctl change-plan apply
aegisctl change-plan confirm
aegisctl change-plan rollback
```

## 6.2 aegis-agent

Chạy trên mỗi node.

Trách nhiệm:

* Kết nối controller.
* Nhận desired state.
* Thu thập node inventory.
* Validate policy trên node.
* Quản lý local state.
* Thực hiện reconciliation.
* Chạy health check.
* Buffer audit event khi controller mất kết nối.
* Giao tiếp với privileged executor.

## 6.3 aegis-execd

Privileged helper.

Trách nhiệm:

* Apply nftables rules.
* Snapshot nftables.
* Restore nftables.
* Apply network profile.
* Restore network profile.
* Thực hiện systemd operation.
* Đọc interface và rule counters.

Không được:

* Mở HTTP API.
* Kết nối controller.
* Thực thi arbitrary shell command.
* Nhận raw command string.

## 6.4 aegis-controller

Control plane cho Middle và Production Ready.

Trách nhiệm:

* Authentication.
* Node enrollment.
* Node inventory.
* Policy management.
* Network profile management.
* Service policy management.
* Change plan generation.
* Rollout coordination.
* Audit.
* API cho Web UI và CLI.

## 6.5 aegis-web

Frontend quản trị.

Các page chính:

```text
Dashboard
Nodes
Firewall
Network Interfaces
Routes
NAT
Docker
Services
Blocked IPs
Change Plans
Audit Logs
Settings
```

---

# 7. Domain Modules

## 7.1 Firewall Domain

Quản lý:

* nftables table.
* Input chain.
* Output chain.
* Forward chain.
* NAT.
* DNAT.
* SNAT.
* Masquerade.
* Stateful filtering.
* Temporary blocklist.
* Rule counters.

Table riêng:

```text
table inet aegis_filter
table ip aegis_nat
table ip6 aegis_nat
```

Không flush toàn bộ nftables ruleset.

## 7.2 Network Domain

Quản lý:

* Physical interfaces.
* Static IPv4 và IPv6.
* DHCP.
* DNS.
* Gateway.
* Static routes.
* Interface roles.
* VLAN.
* Bridge.
* MTU.

Network backend:

```text
NetworkManager
systemd-networkd
Read-only fallback
```

## 7.3 Systemd Domain

Quản lý:

* Unit inventory.
* Unit status.
* Start.
* Stop.
* Restart.
* Reload.
* Enable.
* Disable.
* Journald logs.
* Protected units.
* Allowed units.

## 7.4 Docker Domain

Quản lý:

* Container inventory.
* Docker networks.
* Published ports.
* Container labels.
* Public exposure warnings.
* Dynamic container IP resolution.

AegisNode không chỉnh trực tiếp Docker-managed chains.

## 7.5 Blocker Domain

Quản lý:

* Manual block.
* Temporary block.
* SSH brute-force detector.
* Nginx detector.
* nftables set timeout.
* Block history.

## 7.6 Change Plan Domain

Kết hợp:

```text
Firewall changes
Network changes
Systemd operations
Docker-derived changes
```

Thành một transaction logic thống nhất.

---

# 8. Change Plan Architecture

Ví dụ một change plan:

```yaml
apiVersion: aegisnode.io/v1
kind: NodeChangePlan

metadata:
  id: change-00042
  node: router-01

spec:
  firewallPolicyVersion: 27
  networkProfileVersion: 12

  serviceOperations:
    - unit: nginx.service
      action: reload

  rollback:
    enabled: true
    timeoutSeconds: 60

  healthChecks:
    - type: controller-connectivity

    - type: gateway
      interface: enp3s0

    - type: tcp
      address: 127.0.0.1
      port: 22
```

Apply flow:

```text
Validate schemas
      ↓
Validate dependencies
      ↓
Generate execution order
      ↓
Snapshot firewall
      ↓
Snapshot network
      ↓
Snapshot service state
      ↓
Install temporary management rule
      ↓
Stage network configuration
      ↓
Stage firewall configuration
      ↓
Create local rollback timer
      ↓
Activate changes
      ↓
Run health checks
      ↓
Confirm or rollback
```

Rollback được điều khiển tại node, không phụ thuộc controller.

---

# 9. Policy Priority

Thứ tự ưu tiên:

```text
1. Emergency policy
2. Node-specific policy
3. Node group policy
4. Global baseline policy
5. Default policy
```

Ví dụ:

```yaml
apiVersion: aegisnode.io/v1
kind: FirewallPolicy

metadata:
  name: web-server-policy

spec:
  priority: 100

  defaults:
    input: drop
    output: accept
    forward: drop

  rules:
    - id: allow-established
      direction: input
      connectionStates:
        - established
        - related
      action: accept

    - id: allow-http
      direction: input
      interfaceRole: wan
      protocol: tcp
      destinationPorts:
        - 80
        - 443
      action: accept

    - id: allow-management
      direction: input
      interfaceRole: management
      protocol: tcp
      destinationPorts:
        - 22
        - 8443
      sourceCidrs:
        - 10.10.0.0/16
      action: accept
```

---

# 10. Communication

## Local communication

```text
/run/aegisnode/agent.sock
/run/aegisnode/execd.sock
```

CLI local:

```text
aegisctl
    ↓
agent.sock
    ↓
aegis-agent
```

Agent đến privileged helper:

```text
aegis-agent
    ↓
execd.sock
    ↓
aegis-execd
```

## Remote communication

```text
Web UI / CLI
      ↓ HTTPS
Aegis Controller
      ↓ mTLS
Aegis Agent
```

Protocol có thể dùng:

* REST/JSON cho Web UI.
* gRPC hoặc HTTP/2 cho controller-agent.
* Unix socket cho local communication.

MVP có thể dùng HTTP/JSON cho toàn bộ internal API để đơn giản hóa.

---

# 11. Storage

## MVP

```text
/etc/aegisnode/
├── config.yaml
├── firewall.yaml
├── network.yaml
└── services.yaml

/var/lib/aegisnode/
├── aegis.db
├── snapshots/
├── policies/
├── change-plans/
└── state/

/var/log/aegisnode/
└── audit.log
```

SQLite lưu:

* Policy versions.
* Apply history.
* Audit events.
* Block history.
* Local users.
* Change plan state.

## Middle

Controller dùng PostgreSQL:

```text
users
roles
nodes
node_groups
firewall_policies
network_profiles
service_policies
change_plans
rollouts
audit_events
```

Agent vẫn dùng SQLite để giữ local state.

---

# 12. Security Model

## MVP

```text
Local Web UI
Bind 127.0.0.1

Local CLI
Unix socket permission

Daemon
CAP_NET_ADMIN hoặc root
```

## Middle

```text
Controller ↔ Agent
mTLS

Web UI ↔ Controller
HTTPS

CLI ↔ Controller
API token hoặc local login
```

## Production Ready

```text
OIDC
RBAC
Signed policy bundles
Privilege-separated executor
Approval workflow
Audit immutability
Secret rotation
Certificate rotation
```

Protected systemd units:

```text
aegis-agent.service
aegis-execd.service
dbus.service
systemd-logind.service
systemd-networkd.service
NetworkManager.service
```

---

# 13. Observability

Metrics:

```text
aegis_agent_up
aegis_policy_apply_total
aegis_policy_rollback_total
aegis_change_plan_duration_seconds
aegis_firewall_drop_packets_total
aegis_blocked_ip_total
aegis_interface_rx_bytes_total
aegis_interface_tx_bytes_total
aegis_service_restart_total
aegis_controller_connection_status
```

Không log từng packet.

Chỉ log:

* Policy apply.
* Policy rollback.
* Rule counter delta.
* Block và unblock event.
* Service operation.
* Network profile change.
* Authentication event.
* User configuration change.

---

# 14. Monorepo Folder Structure

```text
aegisnode/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── README.md
├── ARCHITECTURE.md
├── CONTRIBUTING.md
├── LICENSE
├── Makefile
├── justfile
│
├── apps/
│   ├── aegisctl/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── command.rs
│   │       ├── output.rs
│   │       └── client.rs
│   │
│   ├── aegis-agent/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── config.rs
│   │       ├── runtime.rs
│   │       ├── reconcile.rs
│   │       ├── inventory.rs
│   │       └── health.rs
│   │
│   ├── aegis-execd/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── server.rs
│   │       ├── authorization.rs
│   │       └── executor.rs
│   │
│   └── aegis-controller/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── config.rs
│           ├── api.rs
│           ├── auth.rs
│           ├── node_manager.rs
│           ├── policy_manager.rs
│           └── rollout_manager.rs
│
├── crates/
│   ├── aegis-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       ├── identifiers.rs
│   │       ├── time.rs
│   │       └── result.rs
│   │
│   ├── aegis-models/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── node.rs
│   │       ├── firewall.rs
│   │       ├── network.rs
│   │       ├── service.rs
│   │       ├── block.rs
│   │       └── change_plan.rs
│   │
│   ├── aegis-policy/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── parser.rs
│   │       ├── validator.rs
│   │       ├── conflict.rs
│   │       ├── merger.rs
│   │       └── priority.rs
│   │
│   ├── aegis-firewall/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── backend.rs
│   │       ├── compiler.rs
│   │       ├── nftables.rs
│   │       ├── snapshot.rs
│   │       ├── rollback.rs
│   │       ├── nat.rs
│   │       ├── counters.rs
│   │       └── blocklist.rs
│   │
│   ├── aegis-network/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── backend.rs
│   │       ├── detector.rs
│   │       ├── inventory.rs
│   │       ├── profile.rs
│   │       ├── route.rs
│   │       ├── snapshot.rs
│   │       ├── rollback.rs
│   │       └── adapters/
│   │           ├── mod.rs
│   │           ├── network_manager.rs
│   │           └── networkd.rs
│   │
│   ├── aegis-systemd/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── client.rs
│   │       ├── inventory.rs
│   │       ├── operations.rs
│   │       ├── policy.rs
│   │       ├── protected_units.rs
│   │       └── journald.rs
│   │
│   ├── aegis-docker/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── client.rs
│   │       ├── inventory.rs
│   │       ├── watcher.rs
│   │       ├── labels.rs
│   │       └── exposure.rs
│   │
│   ├── aegis-blocker/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── engine.rs
│   │       ├── threshold.rs
│   │       ├── source.rs
│   │       └── detectors/
│   │           ├── mod.rs
│   │           ├── ssh.rs
│   │           └── nginx.rs
│   │
│   ├── aegis-change-plan/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── planner.rs
│   │       ├── validator.rs
│   │       ├── dependency.rs
│   │       ├── executor.rs
│   │       ├── health_check.rs
│   │       └── rollback.rs
│   │
│   ├── aegis-protocol/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── local.rs
│   │       ├── controller.rs
│   │       ├── agent.rs
│   │       └── messages.rs
│   │
│   ├── aegis-storage/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── repository.rs
│   │       ├── sqlite.rs
│   │       ├── postgres.rs
│   │       └── migrations.rs
│   │
│   ├── aegis-auth/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── identity.rs
│   │       ├── rbac.rs
│   │       ├── token.rs
│   │       └── certificate.rs
│   │
│   ├── aegis-audit/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── event.rs
│   │       ├── writer.rs
│   │       └── buffer.rs
│   │
│   └── aegis-observability/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── metrics.rs
│           ├── logging.rs
│           └── tracing.rs
│
├── web/
│   ├── package.json
│   ├── tsconfig.json
│   ├── vite.config.ts
│   ├── index.html
│   └── src/
│       ├── main.tsx
│       ├── app/
│       ├── api/
│       ├── components/
│       ├── layouts/
│       ├── hooks/
│       ├── stores/
│       ├── types/
│       └── pages/
│           ├── Dashboard/
│           ├── Nodes/
│           ├── Firewall/
│           ├── Network/
│           ├── Routes/
│           ├── Nat/
│           ├── Docker/
│           ├── Services/
│           ├── BlockedIps/
│           ├── ChangePlans/
│           ├── AuditLogs/
│           └── Settings/
│
├── api/
│   ├── openapi.yaml
│   ├── local-api.yaml
│   └── controller-agent.proto
│
├── configs/
│   ├── aegis-agent.example.yaml
│   ├── aegis-controller.example.yaml
│   ├── firewall.example.yaml
│   ├── network.example.yaml
│   ├── services.example.yaml
│   └── change-plan.example.yaml
│
├── migrations/
│   ├── sqlite/
│   └── postgres/
│
├── packaging/
│   ├── systemd/
│   │   ├── aegis-agent.service
│   │   ├── aegis-execd.service
│   │   └── aegis-controller.service
│   ├── deb/
│   ├── rpm/
│   └── container/
│       ├── controller.Dockerfile
│       └── web.Dockerfile
│
├── deploy/
│   ├── compose/
│   │   └── docker-compose.yaml
│   ├── kubernetes/
│   └── baremetal/
│
├── scripts/
│   ├── install.sh
│   ├── uninstall.sh
│   ├── dev-setup.sh
│   ├── generate-api.sh
│   └── create-test-network.sh
│
├── tests/
│   ├── integration/
│   ├── end-to-end/
│   ├── fixtures/
│   └── network-lab/
│
└── docs/
    ├── concepts/
    ├── policies/
    ├── network/
    ├── firewall/
    ├── systemd/
    ├── security/
    ├── operations/
    └── development/
```

---

# 15. MVP Skeleton

MVP chưa cần khởi tạo toàn bộ source tree.

Skeleton ban đầu:

```text
aegisnode/
├── Cargo.toml
├── README.md
├── ARCHITECTURE.md
│
├── apps/
│   ├── aegisctl/
│   └── aegis-agent/
│
├── crates/
│   ├── aegis-core/
│   ├── aegis-models/
│   ├── aegis-policy/
│   ├── aegis-firewall/
│   ├── aegis-docker/
│   ├── aegis-blocker/
│   ├── aegis-change-plan/
│   ├── aegis-storage/
│   └── aegis-protocol/
│
├── web/
├── configs/
├── migrations/sqlite/
├── packaging/systemd/
├── tests/
└── docs/
```

MVP dùng `aegis-agent` như một monolithic local daemon:

```text
aegisctl
    ↓
aegis-agent
    ├── HTTP API
    ├── Unix socket API
    ├── Policy engine
    ├── Firewall backend
    ├── Docker watcher
    ├── Blocker
    ├── SQLite
    └── Web static files
```

---

# 16. Workspace Cargo Skeleton

```toml
[workspace]
resolver = "2"

members = [
    "apps/aegisctl",
    "apps/aegis-agent",

    "crates/aegis-core",
    "crates/aegis-models",
    "crates/aegis-policy",
    "crates/aegis-firewall",
    "crates/aegis-docker",
    "crates/aegis-blocker",
    "crates/aegis-change-plan",
    "crates/aegis-storage",
    "crates/aegis-protocol",
    "crates/aegis-audit",
    "crates/aegis-observability",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "Apache-2.0"
repository = "https://github.com/example/aegisnode"

[workspace.dependencies]
anyhow = "1"
async-trait = "0.1"
axum = "0.8"
chrono = { version = "0.4", features = ["serde"] }
clap = { version = "4", features = ["derive"] }
config = "0.15"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
thiserror = "2"
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
uuid = { version = "1", features = ["v4", "serde"] }
```

Version cụ thể sẽ được khóa lại trong giai đoạn setup repository.

---

# 17. Important Rust Interfaces

## Firewall backend

```rust
#[async_trait::async_trait]
pub trait FirewallBackend: Send + Sync {
    async fn inspect(&self) -> Result<FirewallState>;

    async fn validate(
        &self,
        policy: &FirewallPolicy,
    ) -> Result<ValidationReport>;

    async fn compile(
        &self,
        policy: &FirewallPolicy,
    ) -> Result<CompiledFirewallPolicy>;

    async fn snapshot(&self) -> Result<FirewallSnapshot>;

    async fn apply(
        &self,
        policy: &CompiledFirewallPolicy,
    ) -> Result<ApplyResult>;

    async fn rollback(
        &self,
        snapshot: &FirewallSnapshot,
    ) -> Result<()>;
}
```

## Network backend

```rust
#[async_trait::async_trait]
pub trait NetworkBackend: Send + Sync {
    async fn inspect(&self) -> Result<NetworkState>;

    async fn validate(
        &self,
        profile: &NetworkProfile,
    ) -> Result<ValidationReport>;

    async fn snapshot(&self) -> Result<NetworkSnapshot>;

    async fn stage(
        &self,
        profile: &NetworkProfile,
    ) -> Result<StagedNetworkProfile>;

    async fn activate(
        &self,
        profile: &StagedNetworkProfile,
    ) -> Result<()>;

    async fn rollback(
        &self,
        snapshot: &NetworkSnapshot,
    ) -> Result<()>;
}
```

## Systemd backend

```rust
#[async_trait::async_trait]
pub trait ServiceManager: Send + Sync {
    async fn list_units(&self) -> Result<Vec<ServiceUnit>>;

    async fn inspect(
        &self,
        unit: &str,
    ) -> Result<ServiceState>;

    async fn execute(
        &self,
        operation: &ServiceOperation,
    ) -> Result<ServiceOperationResult>;

    async fn logs(
        &self,
        query: &JournalQuery,
    ) -> Result<Vec<JournalEntry>>;
}
```

## Change plan executor

```rust
#[async_trait::async_trait]
pub trait ChangePlanExecutor: Send + Sync {
    async fn validate(
        &self,
        plan: &NodeChangePlan,
    ) -> Result<ValidationReport>;

    async fn prepare(
        &self,
        plan: &NodeChangePlan,
    ) -> Result<PreparedChangePlan>;

    async fn apply(
        &self,
        plan: &PreparedChangePlan,
    ) -> Result<ChangeExecution>;

    async fn confirm(
        &self,
        execution_id: &str,
    ) -> Result<()>;

    async fn rollback(
        &self,
        execution_id: &str,
    ) -> Result<()>;
}
```

---

# 18. Initial Implementation Order

```text
1. aegis-models
2. aegis-policy
3. aegis-firewall
4. aegis-storage
5. aegis-agent local API
6. aegisctl
7. Safe apply và rollback
8. Web UI
9. Docker discovery
10. Auto blocker
11. Network module
12. Systemd module
13. Central controller
```

Không bắt đầu bằng frontend hoặc controller.

Thứ cần chứng minh đầu tiên:

```text
YAML firewall policy
        ↓
Rust validation
        ↓
Compile nftables
        ↓
Safe apply
        ↓
Automatic rollback
```

---

# 19. MVP Success Criteria

MVP được xem là hoàn thành khi:

* Cài được trên một Linux host.
* Có thể apply firewall policy bằng CLI.
* Có thể chỉnh policy bằng Web UI.
* Không flush rules do Docker hoặc hệ thống khác tạo.
* Có safe apply.
* Tự rollback nếu người dùng mất kết nối.
* Quản lý được input, output, forward và NAT.
* Hiển thị được Docker published ports.
* Có manual block và SSH auto-block.
* Có audit log.
* Restart daemon không làm mất firewall policy.
* nftables vẫn hoạt động khi Web UI bị lỗi.

---

# 20. Middle Success Criteria

Middle được xem là hoàn thành khi:

* Controller quản lý được nhiều node.
* Node mất controller vẫn giữ last-known-good state.
* Có mTLS giữa controller và agent.
* Quản lý được static IP, DHCP, DNS và route.
* Hỗ trợ NetworkManager và systemd-networkd.
* Có WAN, LAN và Management interface roles.
* Quản lý được systemd units qua allowlist.
* Có combined change plan.
* Có local rollback timer.
* Có canary rollout.
* Có central audit và metrics.

---

# 21. Final Architecture Direction

```text
AegisNode Controller
        │
        │ Desired state
        ▼
AegisNode Agent
        │
        │ Validated operations
        ▼
AegisNode Executor
        │
        ├── nftables
        ├── NetworkManager
        ├── systemd-networkd
        └── systemd
```

AegisNode phải giữ ranh giới rõ ràng:

```text
Controller:
Quyết định hệ thống nên ở trạng thái nào.

Agent:
Đối chiếu desired state với local state.

Executor:
Thực hiện operation đặc quyền đã được validate.

Linux kernel và subsystem:
Xử lý network và service runtime.
```

Mục tiêu dài hạn:

> AegisNode trở thành một nền tảng quản lý network, firewall và service cho Linux node có độ an toàn cao, sử dụng ít tài nguyên, hoạt động độc lập tại node và có khả năng mở rộng từ một máy đơn lẻ đến hệ thống quản lý nhiều node production.
