# AegisNode Implementation Roadmap

> Linux Firewall, Network and Service Management Platform written in Rust.

---

# 1. Mục tiêu tổng thể

AegisNode được chia thành ba stage:

```text
Stage 1 — MVP
Một Linux node, local CLI, local Web UI, nftables, Docker, NAT, auto-block.

Stage 2 — Middle
Nhiều node, controller tập trung, network interface, systemd, change plan và rollout.

Stage 3 — Production Final
HA, privilege separation, OIDC, RBAC, signed policy, audit, recovery và hardening.
```

Luồng phát triển:

```text
MVP
  ↓
Middle
  ↓
Production Final
```

Không phát triển ba stage song song.

---

# 2. Quy tắc thực hiện dành cho Gemini

## 2.1 Không thay đổi kiến trúc tùy ý

Gemini phải tuân theo kiến trúc:

```text
Controller
    ↓ desired state
Agent
    ↓ validated operation
Executor
    ↓
Linux subsystem
```

Trong MVP:

```text
CLI / Web UI
      ↓
aegis-agent monolith
      ↓
nftables
```

Trong Middle:

```text
Web UI / Remote CLI
          ↓
aegis-controller
          ↓
aegis-agent
          ↓
Linux subsystem
```

Trong Production Final:

```text
Controller
    ↓
Unprivileged agent
    ↓ Unix socket
Privileged executor
    ↓
nftables / NetworkManager / systemd
```

## 2.2 Không xử lý packet trong Rust userspace

Rust chỉ quản lý policy và cấu hình.

Packet vẫn chạy qua:

```text
Linux kernel
    ↓
nftables
    ↓
Application / Container / LAN
```

## 2.3 Không dùng arbitrary shell command

Không được xây API kiểu:

```json
{
  "command": "systemctl restart nginx"
}
```

Mọi operation phải dùng model có schema cố định:

```json
{
  "operation": "service.restart",
  "unit": "nginx.service"
}
```

## 2.4 Không flush toàn bộ nftables

AegisNode chỉ quản lý table của chính nó:

```text
table inet aegis_filter
table ip aegis_nat
table ip6 aegis_nat
```

Không được chạy:

```bash
nft flush ruleset
```

## 2.5 Mỗi phase phải có

* Code.
* Unit test.
* Integration test nếu phù hợp.
* Documentation.
* Example config.
* Error handling.
* Structured logging.
* Acceptance test.

## 2.6 Quy tắc commit

Mỗi task lớn nên có commit riêng.

Ví dụ:

```text
feat(policy): add firewall policy models
feat(nft): compile input rules
feat(agent): expose local unix socket
test(nft): add firewall compiler fixtures
docs(mvp): document safe apply flow
```

---

# 3. Global Definition of Done

Một phase chỉ được xem là hoàn thành khi:

* `cargo fmt --check` pass.
* `cargo clippy --workspace --all-targets -- -D warnings` pass.
* `cargo test --workspace` pass.
* Không còn `todo!()` trong luồng chính.
* Không còn `unwrap()` và `expect()` trong runtime code, trừ test.
* Error có context rõ ràng.
* Public API quan trọng có Rust documentation.
* Có ít nhất một test cho happy path.
* Có ít nhất một test cho failure path.
* Example config chạy được.
* README hoặc tài liệu tương ứng được cập nhật.

---

# 4. Stage 1 — MVP

## Mục tiêu Stage MVP

Xây AegisNode chạy hoàn chỉnh trên một Linux host.

Topology:

```text
Browser
   ↓
Web UI
   ↓ HTTP localhost
aegis-agent
   ↑ Unix socket
aegisctl
   ↓
nftables / Docker / journald / SQLite
```

MVP cần hỗ trợ:

* Firewall input, output, forward.
* IPv4 và IPv6.
* TCP, UDP, ICMP.
* CIDR và port.
* Connection state.
* NAT và port forwarding.
* Docker discovery.
* Temporary block.
* SSH auto-block cơ bản.
* Safe apply và rollback.
* CLI.
* Local Web UI.
* Audit log.
* SQLite.

---

# Phase 0 — Repository Bootstrap

## Mục tiêu

Tạo monorepo skeleton có thể build và test.

## Task 0.1 — Tạo workspace

Tạo:

```text
aegisnode/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── rustfmt.toml
├── clippy.toml
├── README.md
├── ARCHITECTURE.md
├── ROADMAP.md
├── LICENSE
├── justfile
└── .gitignore
```

Workspace members ban đầu:

```text
apps/aegisctl
apps/aegis-agent

crates/aegis-core
crates/aegis-models
crates/aegis-policy
crates/aegis-firewall
crates/aegis-storage
crates/aegis-protocol
crates/aegis-audit
crates/aegis-observability
```

## Task 0.2 — Khóa Rust toolchain

Tạo `rust-toolchain.toml`.

Yêu cầu:

* Stable Rust.
* Rust edition 2024.
* Components: `rustfmt`, `clippy`.
* Không dùng nightly ở MVP.

## Task 0.3 — Workspace dependencies

Thêm dependency chung:

```text
tokio
serde
serde_json
serde_yaml
thiserror
anyhow
tracing
tracing-subscriber
uuid
chrono
clap
axum
tower
tower-http
sqlx
async-trait
```

Không lặp version trong từng crate nếu có thể dùng `workspace.dependencies`.

## Task 0.4 — Logging bootstrap

Tạo `aegis-observability`.

Hỗ trợ:

* Human-readable logs cho development.
* JSON logs cho production.
* `RUST_LOG`.
* Correlation ID.
* Component field.

Ví dụ:

```json
{
  "level": "INFO",
  "component": "aegis-agent",
  "operation": "policy.apply",
  "policy_id": "default",
  "message": "policy validation completed"
}
```

## Task 0.5 — Error model

Tạo error types chung:

```rust
pub enum AegisError {
    Validation,
    Configuration,
    Storage,
    Firewall,
    Permission,
    NotFound,
    Conflict,
    Timeout,
    Internal,
}
```

Mỗi error cần:

* Error code.
* User-facing message.
* Internal context.
* Optional source error.

## Task 0.6 — CI

Tạo GitHub Actions:

```text
cargo fmt --check
cargo clippy
cargo test
cargo build --release
```

## Deliverables

* Workspace build được.
* Hai binary placeholder chạy được.
* CI pass.
* Logging hoạt động.

## Acceptance Criteria

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

đều pass.

---

# Phase 1 — Core Domain Models

## Mục tiêu

Định nghĩa model ổn định trước khi viết backend.

## Task 1.1 — Identifier types

Tạo strong types:

```rust
NodeId
PolicyId
RuleId
SnapshotId
ChangePlanId
ExecutionId
BlockEntryId
```

Không truyền toàn bộ bằng `String`.

## Task 1.2 — Firewall policy model

Tạo model:

```rust
FirewallPolicy
FirewallDefaults
FirewallRule
FirewallDirection
FirewallAction
TransportProtocol
ConnectionState
PortSpec
CidrSpec
InterfaceSelector
RateLimit
LogOptions
```

Rule hỗ trợ:

* `input`
* `output`
* `forward`
* `accept`
* `drop`
* `reject`
* TCP
* UDP
* ICMP
* source CIDR
* destination CIDR
* source port
* destination port
* interface
* connection state
* logging có rate limit

## Task 1.3 — NAT model

Tạo:

```rust
NatPolicy
MasqueradeRule
SnatRule
DnatRule
PortForwardRule
```

Port forwarding phải chứa:

```text
Incoming interface
Protocol
External port
Destination address
Destination port
Optional source CIDR
```

## Task 1.4 — Blocklist model

Tạo:

```rust
BlockEntry
BlockSource
BlockReason
BlockDuration
BlockStatus
```

Nguồn block:

```text
manual
ssh-detector
nginx-detector
api
```

## Task 1.5 — Docker model

Tạo:

```rust
DockerContainer
DockerNetwork
PublishedPort
ContainerExposure
ContainerLabelPolicy
```

## Task 1.6 — Policy serialization

Hỗ trợ YAML và JSON.

Tạo fixtures:

```text
tests/fixtures/policies/
├── minimal.yaml
├── web-server.yaml
├── docker-host.yaml
├── router.yaml
├── invalid-port.yaml
└── conflicting-rules.yaml
```

## Task 1.7 — Schema version

Mọi policy có:

```yaml
apiVersion: aegisnode.io/v1
kind: FirewallPolicy
```

Từ chối schema version không hỗ trợ.

## Deliverables

* Domain models.
* YAML parser.
* JSON parser.
* Fixtures.
* Serialization tests.

## Acceptance Criteria

* Parse được policy hợp lệ.
* Báo lỗi chính xác khi CIDR sai.
* Báo lỗi chính xác khi port ngoài `1..65535`.
* Round-trip YAML → model → YAML không làm mất dữ liệu quan trọng.

---

# Phase 2 — Policy Validation Engine

## Mục tiêu

Phát hiện lỗi trước khi tạo nftables ruleset.

## Task 2.1 — Schema validation

Kiểm tra:

* ID không trống.
* Rule ID không trùng.
* Port hợp lệ.
* CIDR hợp lệ.
* Protocol và port tương thích.
* Interface name hợp lệ.
* Rate limit lớn hơn 0.
* Timeout hợp lệ.

## Task 2.2 — Semantic validation

Kiểm tra:

* `default input drop` nhưng không có allow loopback.
* `default input drop` nhưng không allow established/related.
* SSH port mở toàn Internet.
* Database port mở WAN.
* NAT rule không có forward rule tương ứng.
* DNAT destination không hợp lệ.
* Rule không thể match.
* Rule bị shadow bởi rule trước.
* Rule mâu thuẫn.

## Task 2.3 — Validation severity

Phân loại:

```text
ERROR
WARNING
INFO
```

Ví dụ:

```text
ERROR:
Port 70000 không hợp lệ.

WARNING:
SSH đang mở cho 0.0.0.0/0.

INFO:
Policy không có output restrictions.
```

## Task 2.4 — Validation report

Model:

```rust
ValidationReport {
    valid: bool,
    errors: Vec<ValidationIssue>,
    warnings: Vec<ValidationIssue>,
    infos: Vec<ValidationIssue>,
}
```

## Task 2.5 — Policy normalization

Chuẩn hóa:

* Sort CIDR.
* Remove duplicate port.
* Merge adjacent port nếu an toàn.
* Canonicalize IPv6.
* Canonicalize protocol names.

## Task 2.6 — Policy hash

Sinh deterministic hash sau normalization.

Dùng để:

* Detect policy change.
* Snapshot metadata.
* Audit.
* Reconciliation.

## Deliverables

* Validator.
* Warning system.
* Conflict detector.
* Policy hash.

## Acceptance Criteria

* Invalid policy không đến được compiler.
* Hai policy có nội dung tương đương sinh cùng hash.
* Mỗi fixture lỗi có message xác định.

---

# Phase 3 — nftables Compiler

## Mục tiêu

Biến policy thành ruleset nftables nhưng chưa apply.

## Task 3.1 — Compiler architecture

Tạo interface:

```rust
pub trait FirewallCompiler {
    fn compile(
        &self,
        policy: &FirewallPolicy,
    ) -> Result<CompiledFirewallPolicy>;
}
```

Output gồm:

```rust
CompiledFirewallPolicy {
    nft_script: String,
    policy_hash: String,
    generated_at: DateTime<Utc>,
    managed_tables: Vec<String>,
}
```

## Task 3.2 — Base table và chain

Sinh:

```nft
table inet aegis_filter {
    chain input {
        type filter hook input priority filter;
        policy drop;
    }

    chain output {
        type filter hook output priority filter;
        policy accept;
    }

    chain forward {
        type filter hook forward priority filter;
        policy drop;
    }
}
```

## Task 3.3 — Base safety rules

Tùy policy cấu hình, hỗ trợ:

```text
Loopback accept
Established/related accept
Invalid drop
ICMP/ICMPv6 cơ bản
```

Không tự thêm rule bí mật. Mọi auto-generated rule phải xuất hiện trong compile report.

## Task 3.4 — Address compilation

Hỗ trợ:

```text
IPv4 source
IPv4 destination
IPv6 source
IPv6 destination
CIDR set
```

Nếu có nhiều CIDR, ưu tiên nftables set.

## Task 3.5 — Port compilation

Hỗ trợ:

```text
Single port
Multiple ports
Port range
```

Dùng nftables set:

```nft
tcp dport { 80, 443, 8080 }
```

## Task 3.6 — Connection state

Compile:

```text
new
established
related
invalid
```

## Task 3.7 — Logging

Mỗi log rule cần:

* Prefix.
* Rate limit.
* Rule ID.
* Không log không giới hạn.

## Task 3.8 — NAT compiler

Sinh table NAT riêng.

Hỗ trợ:

* Masquerade.
* SNAT.
* DNAT.
* Port forward.

## Task 3.9 — Rule counters

Mỗi rule được gắn counter.

Phải có mapping:

```text
Rule ID → nftables rule handle hoặc comment
```

Thêm comment:

```nft
comment "aegis:rule:allow-web"
```

## Task 3.10 — Compiler snapshot tests

Dùng golden files:

```text
input YAML
    ↓
expected nft output
```

## Deliverables

* Compiler hoàn chỉnh.
* NAT compiler.
* Golden tests.
* Rule comment mapping.

## Acceptance Criteria

* Output chạy qua `nft --check`.
* Không sinh lệnh flush toàn bộ ruleset.
* Compiler output deterministic.
* Docker-managed table không bị chạm vào.

---

# Phase 4 — nftables Runtime Backend

## Mục tiêu

Thực hiện inspect, check, apply và snapshot an toàn.

## Task 4.1 — Command runner abstraction

Tạo abstraction để test:

```rust
pub trait ProcessRunner {
    async fn run(&self, request: ProcessRequest) -> Result<ProcessOutput>;
}
```

Không gọi `std::process::Command` rải rác.

## Task 4.2 — nft version và capability detection

Khi agent start:

* Kiểm tra `nft` tồn tại.
* Kiểm tra quyền.
* Kiểm tra kernel hỗ trợ nftables.
* Kiểm tra IPv6 support.
* Log capability report.

## Task 4.3 — Syntax check

Thực hiện:

```bash
nft --check --file candidate.nft
```

Không apply nếu check fail.

## Task 4.4 — Runtime inspection

Đọc:

```bash
nft --json list ruleset
```

Trích xuất:

* Managed table.
* Chain.
* Rules.
* Counter.
* Handle.
* Current hash nếu lưu comment metadata.

## Task 4.5 — Snapshot

Trước apply, lưu:

```text
/var/lib/aegisnode/snapshots/<snapshot-id>/
├── ruleset.nft
├── metadata.json
└── checksum.sha256
```

Metadata:

```text
Snapshot ID
Created time
Policy hash
Agent version
Reason
```

## Task 4.6 — Apply transaction

Luồng:

```text
Compile
→ nft --check
→ snapshot
→ write candidate file
→ apply
→ inspect
→ verify managed tables
```

## Task 4.7 — Restore snapshot

Khôi phục chỉ phần AegisNode quản lý.

Không restore hoặc flush table của ứng dụng khác.

## Task 4.8 — File permission

* Snapshot chỉ root hoặc service account đọc được.
* Candidate file không world-readable.
* Dùng atomic file write.

## Deliverables

* nftables runtime backend.
* Snapshot.
* Restore.
* Capability detection.

## Acceptance Criteria

* Apply policy hợp lệ thành công.
* Syntax lỗi không thay đổi firewall.
* Restore snapshot hoạt động.
* Rule của Docker không bị mất.

---

# Phase 5 — Safe Apply and Automatic Rollback

## Mục tiêu

Ngăn người dùng tự khóa mình khỏi server.

## Task 5.1 — Apply state machine

Tạo state:

```text
CREATED
VALIDATED
SNAPSHOTTED
APPLIED_PENDING_CONFIRMATION
COMMITTED
ROLLING_BACK
ROLLED_BACK
FAILED
```

## Task 5.2 — Pending confirmation

CLI:

```bash
aegisctl firewall apply firewall.yaml --rollback-after 30s
```

Sau apply:

```text
Policy applied.
Confirmation required within 30 seconds.
```

## Task 5.3 — Confirm operation

```bash
aegisctl firewall confirm <execution-id>
```

Confirm phải:

* Kiểm tra execution còn pending.
* Hủy rollback timer.
* Mark committed.
* Audit event.

## Task 5.4 — Local rollback timer

Rollback timer phải nằm trong agent.

Không phụ thuộc CLI hoặc Web UI còn chạy.

## Task 5.5 — Agent restart recovery

Nếu agent restart khi có execution pending:

* Đọc pending state từ SQLite.
* Kiểm tra deadline.
* Nếu hết hạn, rollback.
* Nếu chưa hết hạn, tiếp tục timer.

## Task 5.6 — Basic health checks

Sau apply kiểm tra:

* Loopback.
* Managed table tồn tại.
* Required listening port chưa bị chặn theo static analysis.
* Optional gateway ping.
* Optional TCP target.

## Task 5.7 — Manual rollback

```bash
aegisctl firewall rollback <execution-id>
```

## Task 5.8 — Concurrent apply lock

Chỉ cho một change execution chạy trên một node.

## Deliverables

* Safe apply.
* Rollback timer.
* Recovery sau restart.
* Execution state persistence.

## Acceptance Criteria

* Không confirm thì tự rollback.
* Agent restart vẫn rollback đúng.
* Không thể chạy hai apply đồng thời.
* Rollback event có trong audit.

---

# Phase 6 — Local Agent and SQLite

## Mục tiêu

Xây daemon local làm trung tâm cho MVP.

## Task 6.1 — Agent config

File:

```text
/etc/aegisnode/agent.yaml
```

Config:

```yaml
server:
  unixSocket: /run/aegisnode/agent.sock
  http:
    enabled: true
    bind: 127.0.0.1:8080

storage:
  database: /var/lib/aegisnode/aegis.db

firewall:
  policyFile: /etc/aegisnode/firewall.yaml
  rollbackTimeoutSeconds: 30
```

## Task 6.2 — Config validation

Kiểm tra:

* Path tuyệt đối.
* Port hợp lệ.
* Không bind public mặc định.
* Directory permission.
* Duplicate config.

## Task 6.3 — SQLite migrations

Tạo bảng:

```text
policy_versions
apply_executions
snapshots
audit_events
block_entries
agent_state
settings
```

## Task 6.4 — Repository layer

Không truy cập SQL trực tiếp từ handler.

Tạo repository traits.

## Task 6.5 — Unix socket server

Endpoint local:

```text
GET  /v1/status
GET  /v1/firewall/policy
POST /v1/firewall/validate
POST /v1/firewall/apply
POST /v1/firewall/confirm
POST /v1/firewall/rollback
GET  /v1/firewall/rules
GET  /v1/audit
```

## Task 6.6 — Local HTTP server

Cùng handler với Unix socket.

Chỉ bind `127.0.0.1` mặc định.

## Task 6.7 — Agent reconciliation

Khi start:

```text
Load desired policy
Load last committed policy
Inspect nftables
Compare hash
Reconcile nếu cần
```

Không tự apply policy mới nếu chưa được cấu hình cho phép.

## Task 6.8 — Graceful shutdown

* Stop accepting request.
* Finish database transaction.
* Không rollback committed policy.
* Giữ nftables nguyên trạng.

## Deliverables

* Daemon local.
* SQLite.
* API.
* Config.
* Reconciliation.

## Acceptance Criteria

* Agent restart không làm mất firewall.
* Unix socket hoạt động.
* HTTP chỉ listen localhost.
* Database migrations tự chạy.

---

# Phase 7 — CLI `aegisctl`

## Mục tiêu

Cung cấp CLI hoàn chỉnh cho MVP.

## Task 7.1 — CLI structure

Command tree:

```text
aegisctl
├── status
├── firewall
│   ├── validate
│   ├── compile
│   ├── apply
│   ├── confirm
│   ├── rollback
│   ├── rules
│   └── counters
├── block
│   ├── list
│   ├── add
│   └── remove
├── docker
│   ├── containers
│   ├── networks
│   └── exposure
├── audit
└── version
```

## Task 7.2 — Output modes

Hỗ trợ:

```bash
--output table
--output json
--output yaml
```

JSON output phải ổn định để dùng cho automation.

## Task 7.3 — Exit codes

Định nghĩa:

```text
0 success
1 generic failure
2 validation failure
3 permission denied
4 not found
5 conflict
6 timeout/rollback
```

## Task 7.4 — Local socket client

Mặc định kết nối:

```text
/run/aegisnode/agent.sock
```

## Task 7.5 — Compile preview

```bash
aegisctl firewall compile firewall.yaml
```

Chỉ in generated nftables, không apply.

## Task 7.6 — Interactive confirmation

Khi apply interactive:

```text
Policy contains warnings:
- SSH exposed to all IPv4 addresses.

Continue? [y/N]
```

Có flag:

```bash
--yes
```

## Task 7.7 — Shell completion

Hỗ trợ:

* Bash.
* Zsh.
* Fish.

## Deliverables

* CLI hoàn chỉnh.
* JSON output.
* Shell completion.
* Man page cơ bản.

## Acceptance Criteria

* Mọi operation MVP thực hiện được bằng CLI.
* CLI không cần root nếu Unix socket permission cho phép.
* Script có thể parse JSON output.

---

# Phase 8 — Docker, NAT and Router Mode

## Mục tiêu

Hỗ trợ Docker host và Linux router nhỏ.

## Task 8.1 — Docker socket client

Kết nối:

```text
/var/run/docker.sock
```

Đọc:

* Containers.
* Networks.
* Published ports.
* Labels.
* Container IP.
* Container status.

## Task 8.2 — Docker event watcher

Theo dõi:

```text
container start
container stop
container destroy
network connect
network disconnect
```

## Task 8.3 — Exposure analysis

Phát hiện:

* Database port public.
* Redis public.
* Docker API public.
* Port bind `0.0.0.0`.
* Port bind IPv6 wildcard.

## Task 8.4 — Docker policy labels

Hỗ trợ:

```yaml
aegis.exposure: private
aegis.allowed-cidrs: 10.0.0.0/8
```

MVP chỉ đọc label và tạo warning hoặc desired rules.

## Task 8.5 — Router mode config

Config:

```yaml
router:
  enabled: true
  ipForwarding: true
```

## Task 8.6 — IP forwarding

Quản lý:

```text
net.ipv4.ip_forward
net.ipv6.conf.all.forwarding
```

Snapshot giá trị cũ trước thay đổi.

## Task 8.7 — Masquerade

Hỗ trợ LAN → WAN.

## Task 8.8 — Port forwarding

Hỗ trợ:

```text
WAN TCP 443
    ↓
192.168.10.20:443
```

## Task 8.9 — NAT validation

Phát hiện:

* Destination IP invalid.
* Interface không tồn tại.
* Duplicate external port.
* DNAT không có forward accept.
* Forward target trỏ vào local machine sai semantics.

## Deliverables

* Docker inventory.
* Exposure warnings.
* Router mode.
* NAT.
* Port forward.

## Acceptance Criteria

* Docker container restart không làm mất policy logic.
* Router VM có thể NAT LAN ra Internet.
* Port forward hoạt động.
* Không sửa Docker-owned chains.

---

# Phase 9 — Blocker and Journald

## Mục tiêu

Tích hợp manual block và SSH auto-block.

## Task 9.1 — nftables block set

Tạo:

```nft
set blocked_ipv4 {
    type ipv4_addr
    flags timeout
}

set blocked_ipv6 {
    type ipv6_addr
    flags timeout
}
```

Chỉ cần một lookup rule.

## Task 9.2 — Manual block

CLI:

```bash
aegisctl block add 203.0.113.10 \
  --duration 30m \
  --reason "manual investigation"
```

## Task 9.3 — Permanent block

Hỗ trợ block không timeout nhưng phải cảnh báo.

## Task 9.4 — Unblock

```bash
aegisctl block remove 203.0.113.10
```

## Task 9.5 — Journald reader

Đọc SSH logs từ journal.

Không gọi `journalctl` liên tục theo polling nếu có thể dùng journal API.

## Task 9.6 — SSH detector

Config:

```yaml
blocker:
  ssh:
    enabled: true
    threshold: 10
    windowSeconds: 60
    blockSeconds: 1800
```

## Task 9.7 — Threshold engine

Theo dõi:

```text
IP
Event type
Window
Count
Last event
```

Giới hạn memory và dọn entry cũ.

## Task 9.8 — Allowlist

Không block:

* Loopback.
* Management CIDR.
* Explicit allowlist.
* Current admin IP nếu xác định được.

## Task 9.9 — Audit

Lưu:

* IP.
* Reason.
* Detector.
* Threshold.
* Start.
* Expiry.
* Matched packets.

## Deliverables

* Blocklist.
* SSH detector.
* Journald reader.
* Allowlist.

## Acceptance Criteria

* IP vượt threshold bị block.
* Block tự hết hạn.
* Management CIDR không bị block.
* Agent restart đồng bộ được block state.

---

# Phase 10 — MVP Web UI

## Mục tiêu

Tạo giao diện local quản trị toàn bộ tính năng MVP.

## Task 10.1 — Frontend bootstrap

Dùng:

```text
React
TypeScript
Vite
```

Tạo structure:

```text
web/src/
├── api/
├── app/
├── components/
├── layouts/
├── pages/
├── hooks/
├── stores/
└── types/
```

## Task 10.2 — Dashboard

Hiển thị:

* Firewall status.
* Current policy hash.
* Allowed/dropped counters.
* Active blocks.
* Docker containers.
* Public ports.
* Pending apply.
* Recent audit events.

## Task 10.3 — Firewall policy editor

Hỗ trợ:

* Form mode.
* YAML mode.
* Validation.
* Warning list.
* Compile preview.
* Apply.
* Confirm.
* Rollback.

## Task 10.4 — Rule list

Hiển thị:

* Rule ID.
* Direction.
* Protocol.
* Source.
* Destination.
* Ports.
* Action.
* Counter.

## Task 10.5 — Docker page

Hiển thị:

* Container.
* Image.
* Network.
* Published ports.
* Public exposure warning.

## Task 10.6 — NAT page

Tạo và xem:

* Masquerade.
* Port forwarding.
* Interface mapping.

## Task 10.7 — Blocked IP page

* Add block.
* Remove block.
* Expiry countdown.
* Reason.
* Detector source.

## Task 10.8 — Audit page

Filter:

* Time.
* Event type.
* User.
* IP.
* Policy ID.
* Execution ID.

## Task 10.9 — Safe apply modal

Sau apply:

```text
Policy applied but not committed.
Rollback in 24 seconds.

[Confirm] [Rollback]
```

## Task 10.10 — Static asset serving

Build frontend và embed hoặc serve qua agent.

## Deliverables

* Local Web UI.
* API integration.
* Safe apply UX.
* Audit UI.

## Acceptance Criteria

* Có thể quản lý toàn bộ MVP không cần CLI.
* Refresh browser không làm mất pending execution.
* Web UI không bind public mặc định.
* API error được hiển thị rõ ràng.

---

# Phase 11 — MVP Packaging and End-to-End Testing

## Mục tiêu

Đóng gói MVP thành sản phẩm cài được.

## Task 11.1 — systemd unit

Tạo:

```text
aegis-agent.service
```

Yêu cầu:

* Restart on failure.
* Runtime directory.
* State directory.
* Config directory.
* Capability phù hợp.
* Hardening vừa đủ.

## Task 11.2 — Install script

Thực hiện:

* Check nftables.
* Create user/group.
* Create directories.
* Install binary.
* Install config.
* Enable service.
* Preserve existing config.

## Task 11.3 — Uninstall script

Không xóa policy hoặc database nếu chưa có flag purge.

## Task 11.4 — Debian package

Tạo `.deb` cơ bản.

## Task 11.5 — Test network lab

Dùng Linux network namespace:

```text
wan namespace
firewall namespace
lan namespace
```

Test:

* Input allow/drop.
* Forward.
* NAT.
* Port forward.
* Rollback.
* Blocklist.

## Task 11.6 — Docker integration test

Chạy container test:

* Nginx public.
* PostgreSQL accidental public.
* Redis private.

## Task 11.7 — Upgrade test

Test:

* Install v0.1.
* Có policy active.
* Upgrade binary.
* Firewall vẫn hoạt động.
* Database migration thành công.

## MVP Completion Criteria

Stage MVP hoàn thành khi:

* Cài được trên Ubuntu/Debian test VM.
* CLI hoạt động.
* Web UI hoạt động.
* Safe apply rollback hoạt động.
* Docker discovery hoạt động.
* NAT hoạt động.
* SSH auto-block hoạt động.
* E2E tests pass.
* Reboot host không làm mất policy.

---

# 5. Stage 2 — Middle

## Mục tiêu Stage Middle

Biến AegisNode thành nền tảng quản lý nhiều Linux node.

Topology:

```text
Web UI / Remote CLI
          ↓
aegis-controller
          ↓ mTLS
aegis-agent trên nhiều node
          ↓
nftables / NetworkManager / systemd
```

Bổ sung:

* Controller.
* Node enrollment.
* mTLS.
* Multi-node inventory.
* Network interface management.
* Routes.
* VLAN.
* Bridge.
* Systemd management.
* Combined change plan.
* Canary rollout.
* PostgreSQL.
* Metrics tập trung.

---

# Phase 12 — Controller Foundation

## Mục tiêu

Tạo control plane cơ bản.

## Task 12.1 — Tạo app controller

Tạo:

```text
apps/aegis-controller
```

Module:

```text
config
api
auth
nodes
policies
rollouts
audit
storage
```

## Task 12.2 — PostgreSQL storage

Tạo migration:

```text
users
api_tokens
nodes
node_groups
node_labels
firewall_policies
policy_versions
network_profiles
service_policies
change_plans
rollouts
rollout_targets
audit_events
```

## Task 12.3 — Controller API

Endpoints:

```text
POST /v1/nodes/enroll
GET  /v1/nodes
GET  /v1/nodes/{id}
POST /v1/policies
GET  /v1/policies
POST /v1/change-plans
POST /v1/rollouts
GET  /v1/audit
```

## Task 12.4 — Local authentication

Middle ban đầu hỗ trợ:

* Username/password.
* API token.
* Password hash mạnh.
* Token expiration.

OIDC để Production Final.

## Task 12.5 — Node group

Node có thể thuộc group:

```text
web-servers
docker-hosts
routers
database-servers
```

## Task 12.6 — Label selector

Hỗ trợ:

```yaml
matchLabels:
  environment: production
  role: web
```

## Task 12.7 — Controller WebSocket/SSE

Dùng cho Web UI real-time status, không dùng làm agent protocol chính.

## Deliverables

* Controller API.
* PostgreSQL.
* User login.
* Node group.
* Policy storage.

## Acceptance Criteria

* Controller start với PostgreSQL.
* Tạo user.
* Tạo policy.
* Tạo node record.
* Audit mọi write operation.

---

# Phase 13 — Agent Enrollment and mTLS

## Mục tiêu

Kết nối agent với controller an toàn.

## Task 13.1 — Enrollment token

Controller tạo token một lần:

```bash
aegisctl enrollment create --expires 15m
```

## Task 13.2 — Agent enrollment

Agent gửi:

* Token.
* Node hostname.
* Machine ID.
* Agent version.
* Public key.
* Basic inventory.

## Task 13.3 — Certificate issuance

Controller hoặc internal CA cấp client certificate.

## Task 13.4 — mTLS channel

Mọi agent-controller traffic dùng mTLS.

## Task 13.5 — Certificate storage

Agent lưu certificate với permission chặt.

## Task 13.6 — Certificate renewal

Renew trước expiry.

## Task 13.7 — Node revocation

Controller có thể revoke node certificate.

## Task 13.8 — Heartbeat

Agent gửi:

```text
node ID
agent version
current policy hash
health status
uptime
last apply result
```

## Task 13.9 — Offline detection

Controller mark node:

```text
ONLINE
DEGRADED
OFFLINE
REVOKED
```

## Deliverables

* Enrollment.
* mTLS.
* Certificate lifecycle.
* Heartbeat.

## Acceptance Criteria

* Agent không có certificate không kết nối được.
* Token hết hạn không dùng được.
* Revoked node bị từ chối.
* Controller restart không mất enrollment state.

---

# Phase 14 — Node Inventory

## Mục tiêu

Thu thập đầy đủ trạng thái Linux node.

## Task 14.1 — System inventory

Thu thập:

* Hostname.
* OS.
* Kernel.
* CPU.
* RAM.
* Uptime.
* Machine ID.
* Agent version.

## Task 14.2 — Interface inventory

Thu thập:

* Interface name.
* MAC.
* Type.
* Driver.
* MTU.
* Operational state.
* Carrier.
* Speed.
* IPv4.
* IPv6.
* RX/TX counters.

## Task 14.3 — Route inventory

Thu thập:

* Default route.
* Static route.
* Gateway.
* Metric.
* Table.

## Task 14.4 — Firewall inventory

Thu thập:

* Current policy hash.
* Managed table.
* Rule count.
* Active block count.
* Apply status.

## Task 14.5 — Service inventory

Thu thập systemd unit summary.

## Task 14.6 — Docker inventory

Thu thập nếu Docker tồn tại.

## Task 14.7 — Inventory diff

Không gửi full payload liên tục nếu không thay đổi.

Dùng hash hoặc delta.

## Task 14.8 — Controller storage

Lưu current inventory và history cần thiết.

## Deliverables

* Agent inventory collector.
* Controller inventory API.
* Node detail data.

## Acceptance Criteria

* Web UI xem được node state.
* Interface state cập nhật.
* Docker không cài thì agent không lỗi.
* Inventory payload có version.

---

# Phase 15 — Network Interface Management

## Mục tiêu

Quản lý NetworkManager và systemd-networkd an toàn.

## Task 15.1 — Backend detection

Phát hiện:

```text
NetworkManager
systemd-networkd
unsupported
```

Nếu unsupported, chuyển read-only.

## Task 15.2 — Network backend trait

Tạo:

```rust
inspect
validate
snapshot
stage
activate
rollback
```

## Task 15.3 — NetworkManager adapter

Giao tiếp qua D-Bus.

Hỗ trợ:

* DHCP.
* Static IPv4.
* Static IPv6.
* DNS.
* Gateway.
* MTU.
* Routes.
* Connection profile.

## Task 15.4 — systemd-networkd adapter

Quản lý file:

```text
.network
.netdev
.link nếu cần
```

Dùng staging directory và atomic replacement.

## Task 15.5 — Interface identity

Match ưu tiên:

```text
MAC address
Permanent MAC
PCI path
Interface name
```

Không chỉ dựa vào tên interface.

## Task 15.6 — Network profile model

Tạo:

```rust
NetworkProfile
InterfaceProfile
AddressConfig
RouteConfig
DnsConfig
VlanConfig
BridgeConfig
```

## Task 15.7 — Interface roles

Hỗ trợ:

```text
WAN
LAN
MANAGEMENT
STORAGE
UNSPECIFIED
```

## Task 15.8 — VLAN

Tạo và xóa VLAN.

## Task 15.9 — Bridge

Tạo bridge và attach interface.

## Task 15.10 — Safe network apply

Luồng:

```text
snapshot
stage
create rollback job
activate
health check
confirm
```

## Task 15.11 — Management interface protection

Không cho disable interface đang dùng kết nối controller nếu không có recovery path.

## Deliverables

* NetworkManager adapter.
* networkd adapter.
* Network profiles.
* VLAN.
* Bridge.
* Safe rollback.

## Acceptance Criteria

* Đổi static IP và rollback được.
* Agent không tự mất controller vĩnh viễn.
* Unsupported backend ở read-only.
* Network profile validation rõ ràng.

---

# Phase 16 — Systemd and Journald Management

## Mục tiêu

Quản lý service an toàn, không biến thành remote shell.

## Task 16.1 — systemd D-Bus client

Hỗ trợ:

* List units.
* Unit status.
* Start.
* Stop.
* Restart.
* Reload.
* Enable.
* Disable.

## Task 16.2 — Service model

Tạo:

```rust
ServiceUnit
ServiceState
ServiceOperation
ServiceOperationResult
ServicePolicy
```

## Task 16.3 — Allowlist

Config:

```yaml
services:
  allowedUnits:
    - nginx.service
    - docker.service
    - postgresql.service
```

## Task 16.4 — Protected units

Mặc định bảo vệ:

```text
aegis-agent.service
dbus.service
systemd-logind.service
NetworkManager.service
systemd-networkd.service
ssh.service
```

SSH có thể thao tác nhưng yêu cầu elevated confirmation.

## Task 16.5 — Journal viewer

Hỗ trợ query:

* Unit.
* Time range.
* Priority.
* Limit.
* Cursor.

## Task 16.6 — Desired service state

Ví dụ:

```yaml
unit: nginx.service
desiredState: running
enabled: true
```

## Task 16.7 — Drift detection

Nếu service desired running nhưng stopped:

* Report drift.
* Không tự start trừ khi reconcile mode bật.

## Task 16.8 — Operation audit

Lưu:

* User.
* Node.
* Unit.
* Action.
* Previous state.
* Result.

## Deliverables

* Systemd API.
* Allowlist.
* Protected unit.
* Journald.
* Service desired state.

## Acceptance Criteria

* Không chạy arbitrary unit command.
* Protected unit không bị stop.
* Log query có pagination/cursor.
* Mọi operation có audit.

---

# Phase 17 — Combined Change Plan

## Mục tiêu

Apply firewall, network và service theo một kế hoạch thống nhất.

## Task 17.1 — Change plan model

Tạo:

```rust
NodeChangePlan
FirewallChange
NetworkChange
ServiceChange
HealthCheckSpec
RollbackPolicy
ExecutionStep
```

## Task 17.2 — Dependency planner

Ví dụ dependency:

```text
Temporary management allow
    ↓
Stage network
    ↓
Stage firewall
    ↓
Activate network
    ↓
Activate firewall
    ↓
Reload service
```

## Task 17.3 — Risk classification

Phân loại:

```text
LOW
MEDIUM
HIGH
CRITICAL
```

Ví dụ:

```text
Add firewall allow rule → LOW
Restart nginx → MEDIUM
Change management IP → HIGH
Disable management NIC → CRITICAL
```

## Task 17.4 — Dry-run preview

Hiển thị diff:

```text
Network:
192.168.10.1/24 → 192.168.20.1/24

Firewall:
+ allow 192.168.20.0/24
- allow 192.168.10.0/24

Services:
reload dnsmasq.service
```

## Task 17.5 — Combined snapshot

Snapshot:

* Firewall.
* Network.
* Relevant service states.
* sysctl state.

## Task 17.6 — Local rollback coordinator

Rollback tại node theo reverse order.

## Task 17.7 — Controller confirmation

Controller confirm chỉ sau khi agent health check pass.

Nếu controller mất kết nối, local timeout quyết định rollback.

## Task 17.8 — Idempotency

Mỗi plan có idempotency key.

Retry không được apply hai lần.

## Deliverables

* Planner.
* Risk engine.
* Combined apply.
* Combined rollback.
* Dry-run diff.

## Acceptance Criteria

* Change network và firewall cùng lúc không khóa management.
* Failure ở service step rollback được network và firewall khi policy yêu cầu.
* Duplicate request không execute hai lần.

---

# Phase 18 — Multi-Node Rollout

## Mục tiêu

Rollout policy tới nhiều node an toàn.

## Task 18.1 — Rollout model

Tạo:

```rust
Rollout
RolloutTarget
RolloutStrategy
RolloutStatus
```

## Task 18.2 — Strategies

Hỗ trợ:

```text
all-at-once
canary
batch
manual
```

## Task 18.3 — Canary

Ví dụ:

```yaml
strategy:
  type: canary
  canaryCount: 1
  pauseSeconds: 60
```

## Task 18.4 — Batch rollout

Ví dụ:

```yaml
batchSize: 10
maxUnavailable: 2
```

## Task 18.5 — Health gates

Chỉ tiếp tục khi:

* Agent online.
* Apply success.
* No rollback.
* Management connectivity healthy.
* Required services healthy.

## Task 18.6 — Pause/resume/cancel

API:

```text
POST /rollouts/{id}/pause
POST /rollouts/{id}/resume
POST /rollouts/{id}/cancel
```

## Task 18.7 — Automatic halt

Dừng rollout nếu failure threshold vượt giới hạn.

## Task 18.8 — Rollback rollout

Rollback các node đã apply thành công về version trước.

## Deliverables

* Rollout coordinator.
* Canary.
* Batch.
* Pause/resume.
* Rollback.

## Acceptance Criteria

* Canary fail thì node còn lại không bị apply.
* Controller restart khôi phục rollout state.
* Không rollout song song conflict trên cùng node.

---

# Phase 19 — Middle Web UI and Observability

## Mục tiêu

Hoàn thiện trải nghiệm multi-node.

## Task 19.1 — Login

* Session.
* Logout.
* Token refresh.
* Basic role: viewer/admin.

## Task 19.2 — Nodes page

Hiển thị:

* Online/offline.
* Role.
* OS.
* Current policy.
* Interface.
* Health.
* Pending changes.

## Task 19.3 — Node detail

Tabs:

```text
Overview
Firewall
Network
Routes
Services
Docker
Blocks
Audit
```

## Task 19.4 — Network editor

Hỗ trợ:

* Static/DHCP.
* DNS.
* Routes.
* Interface role.
* VLAN.
* Bridge.
* Diff preview.
* Rollback timer.

## Task 19.5 — Service page

* Status.
* Start/stop/restart/reload.
* Protected badge.
* Logs.
* Resource summary.

## Task 19.6 — Change plans page

* Draft.
* Validate.
* Risk.
* Approve.
* Apply.
* Confirm.
* Rollback.

## Task 19.7 — Rollout page

* Target nodes.
* Canary.
* Progress.
* Failure.
* Pause.
* Rollback.

## Task 19.8 — Prometheus metrics

Controller:

```text
aegis_controller_agents_connected
aegis_rollout_active
aegis_rollout_failed_total
```

Agent:

```text
aegis_agent_up
aegis_policy_apply_total
aegis_policy_rollback_total
aegis_interface_rx_bytes_total
```

## Task 19.9 — Grafana dashboards

Tạo dashboard JSON:

* Fleet health.
* Firewall drops.
* Policy rollout.
* Agent connectivity.
* Block activity.

## Middle Completion Criteria

Stage Middle hoàn thành khi:

* Controller quản lý nhiều node.
* mTLS hoạt động.
* Network interface chỉnh được.
* systemd quản lý được.
* Change plan rollback được.
* Canary rollout hoạt động.
* Node offline vẫn giữ last-known-good policy.
* Web UI quản lý được toàn bộ luồng.

---

# 6. Stage 3 — Production Final

## Mục tiêu Stage Production Final

Đưa AegisNode tới mức vận hành production:

* Không single point of failure quan trọng.
* Agent không chạy toàn bộ bằng root.
* OIDC và RBAC.
* Approval workflow.
* Signed policy bundle.
* HA controller.
* Backup và recovery.
* Security hardening.
* Advanced networking.
* Production observability.

---

# Phase 20 — Privilege Separation

## Mục tiêu

Tách agent không đặc quyền và executor có đặc quyền.

## Task 20.1 — Tạo `aegis-execd`

Binary riêng:

```text
apps/aegis-execd
```

## Task 20.2 — Unix socket protocol

Socket:

```text
/run/aegisnode/execd.sock
```

Chỉ agent group truy cập.

## Task 20.3 — Typed operations

Executor chỉ hỗ trợ:

```text
firewall.inspect
firewall.check
firewall.apply
firewall.snapshot
firewall.rollback

network.inspect
network.stage
network.activate
network.rollback

service.inspect
service.start
service.stop
service.restart
service.reload
```

## Task 20.4 — No raw paths

Agent không được gửi path tùy ý để executor đọc/ghi.

Dùng resource ID và validated payload.

## Task 20.5 — Capability minimization

`aegis-execd` dùng capability tối thiểu:

* `CAP_NET_ADMIN`.
* Capability khác chỉ khi thật sự cần.

## Task 20.6 — seccomp/systemd hardening

Systemd unit:

* `NoNewPrivileges`.
* `ProtectSystem`.
* `ProtectHome`.
* `PrivateTmp`.
* `RestrictAddressFamilies`.
* `CapabilityBoundingSet`.

## Task 20.7 — Executor audit

Mọi privileged operation có local audit.

## Deliverables

* Separate executor.
* Typed protocol.
* Capability minimization.
* Hardened units.

## Acceptance Criteria

* Agent chạy non-root.
* Agent compromise không thể chạy shell qua executor.
* Executor không mở network port.

---

# Phase 21 — OIDC, RBAC and Approval Workflow

## Mục tiêu

Xác thực và phân quyền production.

## Task 21.1 — OIDC integration

Hỗ trợ:

* Authorization code flow.
* PKCE.
* JWKS validation.
* Issuer validation.
* Audience validation.

## Task 21.2 — Roles

Tạo:

```text
Viewer
Operator
SecurityAdmin
PlatformAdmin
Auditor
```

## Task 21.3 — Permissions

Ví dụ:

```text
nodes.read
firewall.read
firewall.write
firewall.apply
network.write
service.restart
change_plan.approve
rollout.manage
audit.read
admin.manage
```

## Task 21.4 — Scope

Permission có scope:

```text
global
node-group
specific node
environment
```

## Task 21.5 — Approval policy

Ví dụ:

```text
LOW → không cần approval
MEDIUM → 1 operator
HIGH → 1 security admin
CRITICAL → 2 người khác nhau
```

## Task 21.6 — Four-eyes principle

Người tạo critical plan không được tự approve nếu policy yêu cầu.

## Task 21.7 — Break-glass access

Tài khoản emergency:

* Disabled mặc định.
* Strong audit.
* Time-limited.
* Notification.

## Deliverables

* OIDC.
* RBAC.
* Scoped permissions.
* Approval workflow.
* Break-glass.

## Acceptance Criteria

* Viewer không apply policy.
* Operator không sửa RBAC.
* Critical plan cần approval đúng rule.
* Mọi authorization decision có audit.

---

# Phase 22 — Signed Policy Bundles

## Mục tiêu

Agent chỉ apply policy hợp lệ do controller phát hành.

## Task 22.1 — Bundle format

Bundle gồm:

```text
metadata.json
firewall-policy.json
network-profile.json
service-policy.json
execution-plan.json
checksums.json
signature
```

## Task 22.2 — Signing key management

Controller ký bundle.

Private key không lưu plaintext trong source hoặc config.

## Task 22.3 — Agent verification

Agent kiểm tra:

* Signature.
* Checksum.
* Node target.
* Expiry.
* Policy version.
* Replay protection.

## Task 22.4 — Key rotation

Hỗ trợ multiple trusted public keys trong thời gian chuyển tiếp.

## Task 22.5 — Bundle archive

Lưu bundle đã apply để audit và rollback.

## Task 22.6 — Anti-replay

Bundle có:

* Unique ID.
* Sequence/version.
* Issued time.
* Expiry.
* Target node ID.

## Deliverables

* Signed bundles.
* Verification.
* Key rotation.
* Replay protection.

## Acceptance Criteria

* Bundle bị sửa bị từ chối.
* Bundle dành cho node khác bị từ chối.
* Bundle cũ không replay được.
* Key rotation không gián đoạn rollout.

---

# Phase 23 — Controller High Availability

## Mục tiêu

Loại bỏ controller single point of failure.

## Task 23.1 — Stateless controller API

Không giữ session quan trọng trong memory local.

## Task 23.2 — Multiple controller replicas

Chạy sau load balancer.

## Task 23.3 — PostgreSQL HA

Hỗ trợ deployment với:

* Managed PostgreSQL hoặc
* Primary/replica.

## Task 23.4 — Distributed lock

Dùng PostgreSQL advisory lock hoặc cơ chế phù hợp cho:

* Rollout coordinator.
* Certificate issuance.
* Scheduled reconciliation.

## Task 23.5 — Leader election

Chỉ một controller thực hiện background job cụ thể.

## Task 23.6 — Agent reconnect

Agent có danh sách controller endpoints.

Retry với backoff và jitter.

## Task 23.7 — Controller maintenance

Agent vẫn hoạt động khi toàn bộ controller unavailable.

## Deliverables

* HA controllers.
* Distributed coordination.
* Agent failover.
* Database HA guide.

## Acceptance Criteria

* Kill một controller không làm Web/API gián đoạn đáng kể.
* Rollout không bị execute trùng.
* Agent tự kết nối replica khác.

---

# Phase 24 — Advanced Networking

## Mục tiêu

Hoàn thiện các use case network production.

## Task 24.1 — Bonding

Hỗ trợ:

* Active-backup.
* 802.3ad nếu backend hỗ trợ.

## Task 24.2 — VRF

Inventory và cấu hình VRF cơ bản.

## Task 24.3 — Policy routing

Hỗ trợ:

* Multiple routing tables.
* Source-based rule.
* Interface-based rule.

## Task 24.4 — Multiple default routes

Validation metric và conflict.

## Task 24.5 — WireGuard inventory

Ít nhất hỗ trợ read-only trước.

Write management có thể bật bằng feature flag.

## Task 24.6 — HA gateway preparation

Hỗ trợ integration với:

* Keepalived.
* VRRP status.
* Virtual IP.

Không cần tự implement VRRP.

## Task 24.7 — Network topology visualization

Frontend hiển thị:

```text
WAN
 ↓
Firewall node
 ├── LAN
 ├── Management
 ├── VLAN
 └── Bridge
```

## Deliverables

* Bond.
* VRF.
* Policy routing.
* HA gateway integration.
* Topology UI.

## Acceptance Criteria

* Advanced config có validation và rollback.
* Không apply config không được backend hỗ trợ.
* Management path protection vẫn hoạt động.

---

# Phase 25 — Audit, Backup and Disaster Recovery

## Mục tiêu

Đảm bảo khả năng điều tra và phục hồi.

## Task 25.1 — Immutable-style audit

Audit record chứa:

* Actor.
* Action.
* Resource.
* Before hash.
* After hash.
* Timestamp.
* Request ID.
* Source IP.
* Result.

## Task 25.2 — Audit integrity chain

Mỗi event có hash tham chiếu event trước.

Không phải blockchain; chỉ cần tamper-evident chain.

## Task 25.3 — Audit export

Hỗ trợ:

* JSON.
* Syslog.
* Webhook hoặc log pipeline.

## Task 25.4 — PostgreSQL backup

Document:

* Full backup.
* Point-in-time recovery.
* Restore test.

## Task 25.5 — Controller config backup

Backup:

* OIDC config.
* CA metadata.
* Signing public keys.
* Policy bundles.
* Node enrollment records.

## Task 25.6 — Signing key recovery

Có documented procedure.

Không đưa private key vào regular database dump nếu dùng external secret manager.

## Task 25.7 — Agent local recovery

Agent có command:

```bash
aegisctl recovery status
aegisctl recovery rollback-last
aegisctl recovery restore-snapshot
```

## Task 25.8 — Offline emergency mode

Local admin có thể rollback firewall khi controller unavailable.

Phải audit và sync lại sau.

## Deliverables

* Tamper-evident audit.
* Backup.
* Restore.
* Offline recovery.

## Acceptance Criteria

* Restore controller từ backup trên môi trường test.
* Agent vẫn giữ policy trong quá trình disaster.
* Audit chain phát hiện record bị chỉnh sửa.

---

# Phase 26 — Production Observability and Operations

## Mục tiêu

Chuẩn hóa vận hành.

## Task 26.1 — Metrics hoàn chỉnh

Controller:

```text
request rate
request latency
database latency
connected agents
rollout duration
rollout failures
certificate renewal failures
```

Agent:

```text
policy apply duration
rollback total
controller connection
nftables rule counters
blocklist size
network apply failures
service operation failures
```

## Task 26.2 — Tracing

Dùng tracing cho management flow:

```text
Web request
→ controller
→ rollout
→ agent
→ executor
```

Không trace từng packet.

## Task 26.3 — Alert rules

Tạo Prometheus alerts:

* Agent offline.
* Rollback occurred.
* Policy apply failure.
* Certificate expiring.
* Controller database unavailable.
* High firewall drop anomaly.
* Audit pipeline failure.

## Task 26.4 — Runbooks

Tạo:

```text
Agent offline
Controller unavailable
Policy rollout stuck
Network rollback
Certificate renewal failure
Database recovery
Emergency local access
```

## Task 26.5 — Capacity test

Test:

* Số lượng node.
* Heartbeat rate.
* Inventory payload.
* Concurrent rollout.
* Audit throughput.

## Task 26.6 — Long-running test

Chạy soak test:

* Agent reconnect.
* Controller restart.
* Network flapping.
* Docker container churn.
* Repeated policy apply.

## Deliverables

* Dashboards.
* Alerts.
* Traces.
* Runbooks.
* Capacity report.
* Soak test report.

---

# Phase 27 — Security Hardening and Final Release

## Mục tiêu

Đóng production release.

## Task 27.1 — Threat model

Phân tích:

* Compromised Web account.
* Compromised controller.
* Compromised agent.
* Compromised executor socket.
* MITM.
* Replay bundle.
* Malicious policy.
* Self-lockout.
* Audit tampering.

## Task 27.2 — Dependency audit

Chạy:

```text
cargo audit
cargo deny
npm audit
```

## Task 27.3 — Secret scanning

CI kiểm tra secret bị commit.

## Task 27.4 — Fuzzing

Fuzz:

* YAML parser.
* Policy validator.
* Bundle parser.
* Executor protocol.
* CIDR/port parser.

## Task 27.5 — API limits

Thêm:

* Request body limit.
* Rate limit.
* Timeout.
* Pagination.
* Maximum policy size.
* Maximum rule count.

## Task 27.6 — Resource limits

Agent phải có giới hạn:

* Memory.
* Open files.
* Log size.
* Audit buffer.
* Detector state.

## Task 27.7 — Compatibility matrix

Test:

```text
Ubuntu LTS
Debian stable
RHEL-compatible distribution
NetworkManager
systemd-networkd
nftables versions
Docker versions
```

## Task 27.8 — Release artifacts

Tạo:

* `.deb`.
* `.rpm`.
* Container image cho controller và Web UI.
* Checksums.
* SBOM.
* Signed release artifacts.

## Task 27.9 — Upgrade and rollback

Document:

* Controller upgrade.
* Database migration.
* Agent rolling upgrade.
* Downgrade limitations.
* Policy compatibility.

## Task 27.10 — Final documentation

Hoàn thiện:

```text
Installation
Quick start
Firewall policies
Network profiles
Systemd policies
Change plans
Rollouts
RBAC
OIDC
Backup
Recovery
Troubleshooting
Security
```

## Production Final Completion Criteria

AegisNode được xem là Production Final khi:

* Controller chạy HA.
* Agent chạy non-root.
* Executor bị giới hạn đặc quyền.
* mTLS hoạt động.
* OIDC và RBAC hoạt động.
* Policy bundle có chữ ký.
* Approval workflow hoạt động.
* Backup và restore đã được test.
* Network change rollback hoạt động.
* Canary rollout hoạt động.
* Audit có integrity chain.
* Monitoring và alerts đầy đủ.
* Package cài được trên các distribution mục tiêu.
* Upgrade không làm mất firewall policy.
* Security review hoàn thành.

---

# 7. Tổng hợp phase theo stage

## Stage 1 — MVP

```text
Phase 0  Repository Bootstrap
Phase 1  Core Domain Models
Phase 2  Policy Validation Engine
Phase 3  nftables Compiler
Phase 4  nftables Runtime Backend
Phase 5  Safe Apply and Automatic Rollback
Phase 6  Local Agent and SQLite
Phase 7  CLI aegisctl
Phase 8  Docker, NAT and Router Mode
Phase 9  Blocker and Journald
Phase 10 MVP Web UI
Phase 11 Packaging and E2E Testing
```

## Stage 2 — Middle

```text
Phase 12 Controller Foundation
Phase 13 Agent Enrollment and mTLS
Phase 14 Node Inventory
Phase 15 Network Interface Management
Phase 16 Systemd and Journald Management
Phase 17 Combined Change Plan
Phase 18 Multi-Node Rollout
Phase 19 Middle Web UI and Observability
```

## Stage 3 — Production Final

```text
Phase 20 Privilege Separation
Phase 21 OIDC, RBAC and Approval Workflow
Phase 22 Signed Policy Bundles
Phase 23 Controller High Availability
Phase 24 Advanced Networking
Phase 25 Audit, Backup and Disaster Recovery
Phase 26 Production Observability and Operations
Phase 27 Security Hardening and Final Release
```

---

# 8. Thứ tự dependency bắt buộc

```text
Models
  ↓
Validation
  ↓
Compiler
  ↓
Runtime backend
  ↓
Safe apply
  ↓
Agent
  ↓
CLI
  ↓
Web UI
```

Middle:

```text
Controller
  ↓
Enrollment/mTLS
  ↓
Inventory
  ↓
Network + systemd
  ↓
Change plan
  ↓
Rollout
```

Production:

```text
Privilege separation
  ↓
RBAC/approval
  ↓
Signed bundles
  ↓
HA
  ↓
Recovery
  ↓
Hardening
```

Không được triển khai:

```text
Frontend trước API
Controller trước local agent ổn định
Network management trước rollback
Multi-node rollout trước mTLS
Production HA trước idempotency
```

---

# 9. Task execution template cho Gemini

Mỗi khi bắt đầu phase, Gemini phải trả về:

```text
1. Phase objective
2. Existing files affected
3. New files created
4. Dependencies added
5. Implementation steps
6. Tests added
7. Commands to verify
8. Known limitations
```

Sau khi code xong, phải chạy:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace --release
```

Nếu phase có frontend:

```bash
npm run lint
npm run typecheck
npm run test
npm run build
```

Nếu phase liên quan nftables hoặc network:

* Phải có unit test không cần root.
* Phải có integration test chạy trong VM hoặc network namespace.
* Không chạy test phá network của máy development trực tiếp.

---

# 10. Ưu tiên triển khai thực tế

Thứ tự ưu tiên kỹ thuật:

```text
1. Safety
2. Correctness
3. Rollback
4. Observability
5. Performance
6. User interface
```

Không ưu tiên UI đẹp trước khi safe apply hoạt động.

Không tối ưu performance sớm nếu compiler và rollback chưa đúng.

Không thêm eBPF, Kafka, Redis hoặc Kubernetes nếu chưa có nhu cầu rõ ràng.

---

# 11. Kết quả cuối cùng mong muốn

```text
AegisNode Controller
        │
        │ Signed desired state
        ▼
AegisNode Agent
        │
        │ Validated typed operations
        ▼
AegisNode Executor
        │
        ├── nftables
        ├── NetworkManager
        ├── systemd-networkd
        ├── systemd
        └── journald
```

AegisNode phải đạt được mục tiêu:

> Quản lý firewall, network interface và systemd service trên Linux một cách nhẹ, an toàn, có thể rollback, hoạt động độc lập tại node và mở rộng từ một máy đơn lẻ đến một hệ thống multi-node production.
