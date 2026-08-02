# AegisNode — Stage Implementation Checklist

> Rust-based Linux firewall, network and service management platform.

---

# 0. Quy tắc thực hiện chung

## 0.1 Kiến trúc đã khóa

* [ ] Backend, CLI, controller, agent và executor đều viết bằng Rust.
* [ ] Frontend viết bằng React và TypeScript.
* [ ] Frontend được build thành static assets và nhúng vào Rust binary.
* [ ] Chỉ phân phối một binary chính: `aegisnode`.
* [ ] Binary hỗ trợ nhiều mode:

```bash
aegisnode local
aegisnode server
aegisnode agent
aegisnode execd
aegisnode ctl
```

* [ ] Rust không xử lý packet trong userspace.
* [ ] nftables trong Linux kernel xử lý packet.
* [ ] Không sử dụng Go.
* [ ] Không sử dụng C hoặc cgo.
* [ ] Không sử dụng assembly trong kiến trúc chính.
* [ ] Không sử dụng eBPF trong MVP.
* [ ] Không sử dụng Kafka, Redis hoặc Kubernetes nếu chưa có yêu cầu rõ ràng.
* [ ] Không cho frontend thực thi shell command.
* [ ] Không cho controller gửi raw shell command tới node.
* [ ] Không bao giờ chạy `nft flush ruleset`.
* [ ] Chỉ quản lý nftables table thuộc AegisNode.

```text
table inet aegis_filter
table ip aegis_nat
table ip6 aegis_nat
```

---

## 0.2 Definition of Done cho mọi phase

Một phase chỉ được hoàn thành khi:

* [ ] Code đã được format.

```bash
cargo fmt --all --check
```

* [ ] Clippy không có warning.

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

* [ ] Tất cả unit test pass.

```bash
cargo test --workspace
```

* [ ] Release build thành công.

```bash
cargo build --workspace --release
```

* [ ] Không dùng `unwrap()` hoặc `expect()` trong runtime code.
* [ ] Không còn `todo!()` trong luồng chính.
* [ ] Error có message và context rõ ràng.
* [ ] Public model và trait quan trọng có documentation.
* [ ] Có test cho happy path.
* [ ] Có test cho failure path.
* [ ] Có example configuration.
* [ ] Có structured logging.
* [ ] Tài liệu tương ứng đã được cập nhật.
* [ ] Không phá API hoặc config cũ nếu chưa có migration.
* [ ] Không chạy integration test network trực tiếp trên máy development.
* [ ] Test network nguy hiểm phải chạy trong VM hoặc Linux network namespace.

Nếu có frontend:

* [ ] Frontend lint pass.

```bash
npm run lint
```

* [ ] TypeScript type check pass.

```bash
npm run typecheck
```

* [ ] Frontend test pass.

```bash
npm run test
```

* [ ] Frontend production build thành công.

```bash
npm run build
```

---

# Stage 1 — MVP

## Mục tiêu

AegisNode chạy độc lập trên một Linux host, có CLI, Web UI, nftables firewall, Docker discovery, NAT, port forwarding, manual block, SSH auto-block và safe rollback.

## Topology

```text
Browser
   │
   ▼
Embedded React Web UI
   │
   ▼
aegisnode local
├── HTTP API
├── Unix socket API
├── Policy engine
├── nftables backend
├── Docker watcher
├── Blocker
├── SQLite
└── Safe apply
```

---

# MVP Phase 0 — Repository Skeleton

## Repository

* [x] Tạo repository `aegisnode`.
* [x] Tạo root `Cargo.toml`.
* [x] Tạo `Cargo.lock`.
* [x] Tạo `rust-toolchain.toml`.
* [x] Sử dụng Rust stable.
* [x] Sử dụng Rust edition 2024.
* [x] Thêm component `rustfmt`.
* [x] Thêm component `clippy`.
* [x] Tạo `rustfmt.toml`.
* [x] Tạo `.gitignore`.
* [x] Tạo `README.md`.
* [x] Tạo `ARCHITECTURE.md`.
* [x] Tạo `ROADMAP.md`.
* [x] Tạo `CHECKLIST.md`.
* [x] Tạo `LICENSE`.
* [x] Tạo `justfile` hoặc `Makefile`.

## Workspace skeleton

* [x] Tạo binary app:

```text
apps/aegisnode
```

* [x] Tạo các crate ban đầu:

```text
crates/aegis-core
crates/aegis-models
crates/aegis-config
crates/aegis-policy
crates/aegis-firewall
crates/aegis-storage
crates/aegis-api
crates/aegis-cli
crates/aegis-audit
crates/aegis-observability
```

* [x] Tạo frontend:

```text
web/
```

* [x] Tạo các thư mục:

```text
configs/
migrations/
packaging/
scripts/
tests/
docs/
```

## Dependencies nền tảng

* [x] Thêm `tokio`.
* [x] Thêm `axum`.
* [x] Thêm `tower`.
* [x] Thêm `tower-http`.
* [x] Thêm `clap`.
* [x] Thêm `serde`.
* [x] Thêm `serde_json`.
* [x] Thêm `serde_yaml`.
* [x] Thêm `thiserror`.
* [x] Thêm `anyhow` cho application boundary.
* [x] Thêm `tracing`.
* [x] Thêm `tracing-subscriber`.
* [x] Thêm `uuid`.
* [x] Thêm `chrono`.
* [x] Thêm `sqlx`.
* [x] Thêm `async-trait`.
* [x] Thêm crate nhúng frontend như `rust-embed`.

## CI

* [x] Tạo GitHub Actions.
* [x] Chạy format check.
* [x] Chạy clippy.
* [x] Chạy unit test.
* [x] Chạy release build.
* [x] Cache Cargo registry.
* [x] Cache Cargo target hợp lý.
* [x] Không lưu secret trong workflow.

## Gate Phase 0

* [ ] `cargo build --workspace` pass.
* [ ] `cargo test --workspace` pass.
* [ ] Binary chạy được:

```bash
aegisnode --help
```

* [ ] Có các subcommand placeholder:

```text
local
server
agent
execd
ctl
```

---

# MVP Phase 1 — Core Models and Errors

## Core identifiers

* [x] Tạo `NodeId`.
* [x] Tạo `PolicyId`.
* [x] Tạo `RuleId`.
* [x] Tạo `SnapshotId`.
* [x] Tạo `ExecutionId`.
* [x] Tạo `ChangePlanId`.
* [x] Tạo `BlockEntryId`.
* [x] Không dùng `String` trực tiếp cho toàn bộ identifier.

## Error model

* [x] Tạo `AegisError`.
* [x] Có error code ổn định.
* [x] Có user-facing message.
* [x] Có internal context.
* [x] Có source error.
* [x] Chuyển error thành HTTP response.
* [x] Chuyển error thành CLI exit code.

## Firewall models

* [x] Tạo `FirewallPolicy`.
* [x] Tạo `FirewallDefaults`.
* [x] Tạo `FirewallRule`.
* [x] Tạo `FirewallDirection`.
* [x] Tạo `FirewallAction`.
* [x] Tạo `TransportProtocol`.
* [x] Tạo `ConnectionState`.
* [x] Tạo `PortSpec`.
* [x] Tạo `CidrSpec`.
* [x] Tạo `InterfaceSelector`.
* [x] Tạo `RateLimit`.
* [x] Tạo `LogOptions`.

## NAT models

* [x] Tạo `NatPolicy`.
* [x] Tạo `MasqueradeRule`.
* [x] Tạo `SnatRule`.
* [x] Tạo `DnatRule`.
* [x] Tạo `PortForwardRule`.

## Blocklist models

* [x] Tạo `BlockEntry`.
* [x] Tạo `BlockReason`.
* [x] Tạo `BlockSource`.
* [x] Tạo `BlockDuration`.
* [x] Tạo `BlockStatus`.

## API metadata

* [x] Policy có `apiVersion`.
* [x] Policy có `kind`.
* [x] Policy có metadata.
* [x] Từ chối version không hỗ trợ.

```yaml
apiVersion: aegisnode.io/v1
kind: FirewallPolicy
```

## Fixtures

* [x] Tạo `minimal.yaml`.
* [x] Tạo `web-server.yaml`.
* [x] Tạo `docker-host.yaml`.
* [x] Tạo `router.yaml`.
* [x] Tạo `invalid-port.yaml`.
* [x] Tạo `invalid-cidr.yaml`.
* [x] Tạo `conflicting-rules.yaml`.

## Gate Phase 1

* [x] Parse YAML hợp lệ thành công.
* [x] Parse JSON hợp lệ thành công.
* [x] Reject port ngoài `1..65535`.
* [x] Reject CIDR không hợp lệ.
* [x] Round-trip serialization không mất field.
* [x] Schema version được kiểm tra.

---

# MVP Phase 2 — Policy Validation

## Schema validation

* [x] Rule ID không trống.
* [x] Rule ID không trùng.
* [x] Port hợp lệ.
* [x] CIDR hợp lệ.
* [x] Protocol và port tương thích.
* [x] Rate limit lớn hơn 0.
* [x] Interface name hợp lệ.
* [x] Timeout hợp lệ.
* [x] Action hợp lệ với rule type.

## Semantic validation

* [x] Phát hiện thiếu allow loopback.
* [x] Phát hiện thiếu established/related.
* [x] Cảnh báo SSH mở cho toàn Internet.
* [x] Cảnh báo PostgreSQL mở WAN.
* [x] Cảnh báo Redis mở WAN.
* [x] Phát hiện rule không thể match.
* [x] Phát hiện duplicate rule.
* [x] Phát hiện shadowed rule.
* [x] Phát hiện conflicting rule.
* [x] Phát hiện DNAT thiếu forward allow.
* [x] Phát hiện duplicate external port.

## Validation report

* [x] Có severity `ERROR`.
* [x] Có severity `WARNING`.
* [x] Có severity `INFO`.
* [x] Có field path.
* [x] Có rule ID.
* [x] Có machine-readable error code.

## Normalization

* [x] Loại port trùng.
* [x] Chuẩn hóa CIDR.
* [x] Chuẩn hóa IPv6.
* [x] Sắp xếp dữ liệu để output deterministic.
* [x] Tạo policy hash ổn định.
* [x] Hai policy tương đương sinh cùng hash.

## Gate Phase 2

* [x] Invalid policy không đi tiếp tới compiler.
* [x] Mỗi validation error có test.
* [x] Warning không tự động chặn apply nếu người dùng xác nhận.
* [x] Policy hash deterministic.

---

# MVP Phase 3 — nftables Compiler

## Base ruleset

* [x] Sinh table `inet aegis_filter`.
* [x] Sinh chain `input`.
* [x] Sinh chain `output`.
* [x] Sinh chain `forward`.
* [x] Áp dụng default policy.
* [x] Hỗ trợ IPv4.
* [x] Hỗ trợ IPv6.
* [x] Hỗ trợ TCP.
* [x] Hỗ trợ UDP.
* [x] Hỗ trợ ICMP.
* [x] Hỗ trợ ICMPv6.

## Rule matching

* [x] Source CIDR.
* [x] Destination CIDR.
* [x] Source port.
* [x] Destination port.
* [x] Port range.
* [x] Multiple ports.
* [x] Input interface.
* [x] Output interface.
* [x] Connection state.
* [x] Rate limiting.
* [x] Reject mode.
* [x] Logging có rate limit.

## Sets

* [x] Nhiều CIDR dùng nftables set.
* [x] Nhiều port dùng nftables set.
* [x] Blocklist dùng set.
* [x] Không tạo một rule riêng cho từng blocked IP.

## Rule identity

* [x] Mỗi rule có counter.
* [x] Mỗi rule có comment chứa rule ID.

```text
aegis:rule:<rule-id>
```

* [x] Có mapping từ policy rule sang nftables rule.

## NAT compiler

* [x] Sinh table NAT riêng.
* [x] Hỗ trợ masquerade.
* [x] Hỗ trợ SNAT.
* [x] Hỗ trợ DNAT.
* [x] Hỗ trợ port forward.
* [x] Hỗ trợ source CIDR restriction.
* [x] Hỗ trợ interface restriction.

## Compiler output

* [x] Output deterministic.
* [x] Có policy hash.
* [x] Có compile report.
* [x] Liệt kê auto-generated rule.
* [x] Không sinh `flush ruleset`.
* [x] Không sửa table Docker.

## Tests

* [x] Golden test cho web server.
* [x] Golden test cho router.
* [x] Golden test cho IPv6.
* [x] Golden test cho NAT.
* [x] Golden test cho blocklist.
* [x] Golden test cho logging.

## Gate Phase 3

* [x] Mọi generated ruleset pass:

```bash
nft --check --file candidate.nft
```

* [x] Compiler output ổn định qua nhiều lần chạy.
* [x] Không tác động table ngoài AegisNode.

---

# MVP Phase 4 — nftables Runtime Backend

## Process execution

* [x] Tạo `ProcessRunner` abstraction.
* [x] Không gọi `Command` rải rác.
* [x] Có timeout.
* [x] Capture stdout.
* [x] Capture stderr.
* [x] Không nối raw user input vào shell string.
* [x] Không dùng `/bin/sh -c`.

## Capability detection

* [x] Kiểm tra binary `nft`.
* [x] Kiểm tra nftables version.
* [x] Kiểm tra quyền thực thi.
* [x] Kiểm tra kernel support.
* [x] Kiểm tra IPv6 availability.
* [x] Sinh capability report.

## Runtime inspect

* [x] Đọc JSON ruleset.
* [x] Parse table.
* [x] Parse chain.
* [x] Parse rules.
* [x] Parse counters.
* [x] Parse handles.
* [x] Chỉ lấy state thuộc AegisNode.

## Snapshot

* [x] Snapshot trước mỗi apply.
* [x] Lưu ruleset.
* [x] Lưu metadata.
* [x] Lưu checksum.
* [x] Dùng atomic write.
* [x] Permission an toàn.
* [x] Có retention policy.

## Apply

* [x] Compile.
* [x] Syntax check.
* [x] Snapshot.
* [x] Apply.
* [x] Inspect lại.
* [x] Verify table.
* [x] Verify expected policy hash.
* [x] Audit kết quả.

## Restore

* [x] Restore snapshot gần nhất.
* [x] Không restore table ứng dụng khác.
* [x] Verify sau restore.
* [x] Audit rollback.

## Gate Phase 4

* [x] Apply policy hợp lệ thành công.
* [x] Policy syntax lỗi không làm thay đổi runtime.
* [x] Restore snapshot thành công.
* [x] Docker rules không bị mất.

---

# MVP Phase 5 — Safe Apply and Rollback

## State machine

* [x] `CREATED`.
* [x] `VALIDATED`.
* [x] `SNAPSHOTTED`.
* [x] `APPLIED_PENDING_CONFIRMATION`.
* [x] `COMMITTED`.
* [x] `ROLLING_BACK`.
* [x] `ROLLED_BACK`.
* [x] `FAILED`.

## Rollback timer

* [x] Timer chạy trong daemon.
* [x] Timer được persist vào SQLite.
* [x] Timer sống qua agent restart.
* [x] Không phụ thuộc Web UI.
* [x] Không phụ thuộc CLI.
* [x] Không phụ thuộc network connection.

## Confirmation

* [x] CLI confirm.
* [x] API confirm.
* [x] Web UI confirm.
* [x] Reject confirm cho execution hết hạn.
* [x] Reject confirm cho execution không tồn tại.
* [x] Audit confirm.

## Manual rollback

* [x] Rollback theo execution ID.
* [x] Rollback về snapshot cụ thể.
* [x] Verify rollback.
* [x] Audit rollback.

## Apply lock

* [x] Chỉ một apply chạy tại một thời điểm.
* [x] Có conflict response.
* [x] Lock được giải phóng khi process lỗi.
* [x] Khôi phục lock state sau restart.

## Health check MVP

* [x] Kiểm tra managed table.
* [x] Kiểm tra loopback.
* [x] Kiểm tra local management port.
* [x] Optional gateway probe.
* [x] Optional TCP target.
* [x] Có timeout riêng cho từng check.

## Gate Phase 5

* [x] Không confirm thì tự rollback.
* [x] Restart daemon giữa pending state vẫn rollback.
* [x] Hai apply đồng thời bị chặn.
* [x] Mất Web UI không ảnh hưởng rollback.

---

# MVP Phase 6 — Local Agent and Storage

## Agent modes

* [x] Implement `aegisnode local`.
* [x] Implement `aegisnode ctl`.
* [x] Placeholder cho `server`, `agent`, `execd`.

## Configuration

* [x] Tạo `/etc/aegisnode/agent.yaml`.
* [x] Validate path.
* [x] Validate address.
* [x] HTTP bind localhost mặc định.
* [x] Unix socket configurable.
* [x] Rollback timeout configurable.
* [x] Log format configurable.
* [x] Không log secret.

## SQLite

* [x] Migration framework.
* [x] Bảng `policy_versions`.
* [x] Bảng `apply_executions`.
* [x] Bảng `snapshots`.
* [x] Bảng `audit_events`.
* [x] Bảng `block_entries`.
* [x] Bảng `agent_state`.
* [x] Bảng `settings`.

## Repository layer

* [x] Handler không gọi SQL trực tiếp.
* [x] Có repository trait.
* [x] Có transaction.
* [x] Có migration test.
* [x] Có database recovery handling.

## Unix socket API

* [x] Tạo `/run/aegisnode/agent.sock`.
* [x] Set owner/group.
* [x] Set permission.
* [x] API status.
* [x] API validate.
* [x] API compile.
* [x] API apply.
* [x] API confirm.
* [x] API rollback.
* [x] API rules.
* [x] API audit.

## HTTP API

* [x] Bind `127.0.0.1` mặc định.
* [x] Request ID middleware.
* [x] Structured access log.
* [x] Timeout middleware.
* [x] Request body limit.
* [x] Consistent error response.
* [x] Health endpoint.
* [x] Readiness endpoint.

## Reconciliation

* [x] Đọc desired policy.
* [x] Đọc committed policy.
* [x] Inspect nftables.
* [x] So sánh hash.
* [x] Report drift.
* [x] Không tự apply policy không được duyệt.
* [x] Giữ firewall khi daemon dừng.

## Gate Phase 6

* [x] Restart daemon không làm mất firewall.
* [x] SQLite migration chạy tự động.
* [x] Unix socket API hoạt động.
* [x] HTTP không expose ra WAN mặc định.

---

# MVP Phase 7 — CLI

## CLI commands

* [x] `aegisnode ctl status`.
* [x] `aegisnode ctl firewall validate`.
* [x] `aegisnode ctl firewall compile`.
* [x] `aegisnode ctl firewall apply`.
* [x] `aegisnode ctl firewall confirm`.
* [x] `aegisnode ctl firewall rollback`.
* [x] `aegisnode ctl firewall rules`.
* [x] `aegisnode ctl firewall counters`.
* [x] `aegisnode ctl audit list`.
* [x] `aegisnode ctl version`.

## Output

* [x] Table.
* [x] JSON.
* [x] YAML.
* [x] JSON schema ổn định.
* [x] Không trộn log vào JSON stdout.
* [x] Error xuất stderr.

## Exit codes

* [x] `0`: success.
* [x] `1`: generic failure.
* [x] `2`: validation failure.
* [x] `3`: permission denied.
* [x] `4`: not found.
* [x] `5`: conflict.
* [x] `6`: timeout hoặc rollback.

## UX

* [x] Hiển thị warning trước apply.
* [x] Có `--yes`.
* [x] Có `--rollback-after`.
* [x] Có compile preview.
* [x] Có shell completion.
* [x] Có help text rõ ràng.

## Gate Phase 7

* [x] Toàn bộ MVP firewall operation thực hiện được bằng CLI.
* [x] JSON output dùng được trong shell automation.
* [x] CLI không yêu cầu root nếu socket permission cho phép.

---

# MVP Phase 8 — Docker and Router Mode

## Docker

* [x] Kết nối Docker socket.
* [x] Docker không tồn tại thì graceful degradation.
* [x] Container inventory.
* [x] Network inventory.
* [x] Published port inventory.
* [x] Container label inventory.
* [x] Container lifecycle watcher.
* [x] Network connect/disconnect watcher.

## Exposure analysis

* [x] Phát hiện bind `0.0.0.0`.
* [x] Phát hiện bind `[::]`.
* [x] Cảnh báo PostgreSQL public.
* [x] Cảnh báo Redis public.
* [x] Cảnh báo Docker API public.
* [x] Cảnh báo database port public.
* [x] Không tự block khi chưa có approval.

## Docker labels

* [x] Đọc `aegis.exposure`.
* [x] Đọc `aegis.allowed-cidrs`.
* [x] Validate label.
* [x] Cảnh báo label sai.
* [x] Không sửa Docker-owned chain.

## Router mode

* [x] Option bật IPv4 forwarding.
* [x] Option bật IPv6 forwarding.
* [x] Snapshot sysctl cũ.
* [x] Restore sysctl khi rollback.
* [x] Masquerade LAN → WAN.
* [x] Port forwarding.
* [x] Forward rules.
* [x] Validate incoming interface.
* [x] Validate destination IP.
* [x] Validate duplicate port.

## Gate Phase 8

* [x] Docker restart không làm mất policy.
* [x] Linux VM NAT được LAN ra Internet.
* [x] Port forwarding hoạt động.
* [x] Không phá Docker networking.

---

# MVP Phase 9 — Blocker and Journald

## nftables block set

* [x] `blocked_ipv4`.
* [x] `blocked_ipv6`.
* [x] Timeout support.
* [x] Counter support.
* [x] Single lookup rule.

## Manual block

* [x] Add IPv4.
* [x] Add IPv6.
* [x] Temporary duration.
* [x] Permanent block.
* [x] Reason.
* [x] Actor.
* [x] Remove block.
* [x] List blocks.

## Journald

* [x] Đọc journal theo cursor.
* [x] Filter theo unit.
* [x] Persist cursor.
* [x] Không đọc lại toàn bộ log sau restart.
* [x] Giới hạn batch.
* [x] Xử lý journal rotation.

## SSH detector

* [x] Parse failed login.
* [x] Parse invalid user.
* [x] Trích source IP.
* [x] Threshold.
* [x] Time window.
* [x] Block duration.
* [x] Memory cleanup.
* [x] IPv4.
* [x] IPv6.

## Protection

* [x] Không block loopback.
* [x] Không block management CIDR.
* [x] Có explicit allowlist.
* [x] Có maximum block duration.
* [x] Có maximum active block count.
* [x] Rate limit audit event.

## Gate Phase 9

* [x] IP vượt threshold bị block.
* [x] Block tự hết hạn.
* [x] Management CIDR không bị block.
* [x] Restart daemon đồng bộ được block state.

---

# MVP Phase 10 — Web UI

## Frontend bootstrap

* [x] React.
* [x] TypeScript.
* [x] Vite.
* [x] API client.
* [x] Shared types.
* [x] Error boundary.
* [x] Routing.
* [x] Responsive layout cơ bản.

## Pages

* [x] Dashboard.
* [x] Firewall rules.
* [x] Policy editor.
* [x] Docker.
* [x] NAT.
* [x] Blocked IPs.
* [x] Audit logs.
* [x] Settings.

## Dashboard

* [x] Firewall status.
* [x] Policy hash.
* [x] Rule count.
* [x] Allowed packet count.
* [x] Dropped packet count.
* [x] Active blocks.
* [x] Docker containers.
* [x] Public port warning.
* [x] Pending execution.
* [x] Recent audit events.

## Policy editor

* [x] Form mode.
* [x] YAML mode.
* [x] Validate.
* [x] Show errors.
* [x] Show warnings.
* [x] Compile preview.
* [x] Apply.
* [x] Confirm.
* [x] Rollback.

## Safe apply UI

* [x] Countdown.
* [x] Confirm button.
* [x] Rollback button.
* [x] Persist pending state qua refresh.
* [x] Hiển thị execution ID.
* [x] Hiển thị health-check result.

## Embedded assets

* [x] Production build.
* [x] Embed vào binary.
* [x] Serve `index.html`.
* [x] Serve hashed assets.
* [x] SPA fallback.
* [x] Cache headers cho assets.
* [x] Không cache API response sai cách.

## Gate Phase 10

* [x] Quản lý toàn bộ MVP qua Web UI.
* [x] Refresh không mất pending execution.
* [x] Frontend không cần Node.js trên production host.
* [x] Binary chứa frontend assets.

---

# MVP Phase 11 — Packaging and E2E

## systemd

* [x] Tạo `aegisnode-local.service`.
* [x] `Restart=on-failure`.
* [x] `RuntimeDirectory`.
* [x] `StateDirectory`.
* [x] `ConfigurationDirectory`.
* [x] Capability phù hợp.
* [x] File descriptor limit.
* [x] Log vào journald.
* [x] Graceful stop.

## Installation

* [x] Install script.
* [x] Uninstall script.
* [x] Không overwrite config hiện có.
* [x] Không purge database mặc định.
* [x] Check nftables.
* [x] Create user/group.
* [x] Set permission.
* [x] Enable systemd unit.

## Packages

* [x] Debian package.
* [x] Checksums.
* [x] Version command.
* [x] Upgrade script.
* [x] Migration strategy.

## Network namespace test lab

* [x] WAN namespace.
* [x] Firewall namespace.
* [x] LAN namespace.
* [x] Test input accept.
* [x] Test input drop.
* [x] Test output.
* [x] Test forwarding.
* [x] Test NAT.
* [x] Test DNAT.
* [x] Test rollback.
* [x] Test temporary block.
* [x] Test IPv6 nếu environment hỗ trợ.

## Reboot and upgrade tests

* [x] Firewall vẫn active sau reboot.
* [x] Agent reconcile đúng sau reboot.
* [x] Pending rollback xử lý đúng sau crash.
* [x] Upgrade không xóa policy.
* [x] Upgrade không xóa SQLite.
* [x] Migration thành công.

## MVP Stage Gate

MVP chỉ hoàn thành khi:

* [x] Cài được trên Ubuntu LTS.
* [x] Cài được trên Debian stable.
* [x] CLI hoạt động.
* [x] Web UI hoạt động.
* [x] Safe apply hoạt động.
* [x] Automatic rollback hoạt động.
* [x] Docker discovery hoạt động.
* [x] NAT hoạt động.
* [x] Port forwarding hoạt động.
* [x] Manual block hoạt động.
* [x] SSH auto-block hoạt động.
* [x] Audit hoạt động.
* [x] Reboot không mất firewall.
* [x] E2E tests pass.
* [x] Không cần controller để node hoạt động.

---

# Stage 2 — Middle

## Mục tiêu

AegisNode quản lý nhiều Linux node, có controller, agent, mTLS, network interface management, systemd management, combined change plan và multi-node rollout.

## Topology

```text
Web UI / Remote CLI
          │
          ▼
aegisnode server
          │
         mTLS
          │
          ▼
aegisnode agent
          │
          ▼
nftables / NetworkManager / systemd
```

---

# Middle Phase 12 — Controller Foundation

## Controller app

* [x] Implement `aegisnode server`.
* [x] Controller configuration.
* [x] HTTP API.
* [x] PostgreSQL.
* [x] Migration framework.
* [x] Authentication cơ bản.
* [x] API token.
* [x] Audit middleware.
* [x] Request ID.
* [x] Pagination.

## Database

* [x] `users`.
* [x] `api_tokens`.
* [x] `nodes`.
* [x] `node_groups`.
* [x] `node_labels`.
* [x] `firewall_policies`.
* [x] `policy_versions`.
* [x] `network_profiles`.
* [x] `service_policies`.
* [x] `change_plans`.
* [x] `rollouts`.
* [x] `rollout_targets`.
* [x] `audit_events`.

## Node management

* [ ] Create node.
* [ ] List nodes.
* [ ] Get node.
* [ ] Update labels.
* [ ] Assign group.
* [ ] Archive node.
* [ ] Node status model.

## Policy management

* [ ] Create policy.
* [ ] Version policy.
* [ ] Validate policy.
* [ ] Compare versions.
* [ ] Assign policy to node.
* [ ] Assign policy to group.

## Gate Phase 12

* [x] Controller chạy với PostgreSQL.
* [x] API authentication hoạt động.
* [x] Policy được version hóa.
* [x] Mọi write operation có audit.

---

# Middle Phase 13 — Enrollment and mTLS

## Enrollment

* [x] One-time enrollment token.
* [x] Token expiration.
* [x] Token revocation.
* [x] Token usage limit.
* [x] Enrollment audit.

## Agent identity

* [x] Node ID.
* [x] Hostname.
* [x] Machine ID.
* [x] Agent version.
* [x] Public key.
* [x] Basic system inventory.

## Certificates

* [x] Internal CA hoặc configurable CA.
* [x] Issue client certificate.
* [x] Store certificate an toàn.
* [x] Certificate renewal.
* [x] Certificate revocation.
* [x] Certificate expiry monitoring.

## mTLS connection

* [x] Agent xác minh controller.
* [x] Controller xác minh agent.
* [x] Reject unknown agent.
* [x] Reject revoked agent.
* [x] Retry với backoff.
* [x] Jitter.
* [x] Không retry loop tiêu tốn CPU.

## Heartbeat

* [x] Agent version.
* [x] Uptime.
* [x] Policy hash.
* [x] Health.
* [x] Last apply result.
* [x] Pending execution.
* [x] Timestamp.

## Gate Phase 13

* [x] Agent không certificate không kết nối được.
* [x] Token hết hạn bị từ chối.
* [x] Revoked certificate bị từ chối.
* [x] Controller restart không mất enrollment state.

---

# Middle Phase 14 — Node Inventory

## System inventory

* [x] Hostname.
* [x] OS name.
* [x] OS version.
* [x] Kernel.
* [x] CPU.
* [x] Memory.
* [x] Uptime.
* [x] Machine ID.
* [x] Agent version.

## Network inventory

* [x] Interface name.
* [x] Permanent MAC.
* [x] Current MAC.
* [x] Interface type.
* [x] Driver.
* [x] MTU.
* [x] Carrier.
* [x] Operational state.
* [x] Speed.
* [x] Duplex.
* [x] IPv4.
* [x] IPv6.
* [x] RX/TX counters.
* [x] Routes.
* [x] Gateway.
* [x] DNS.

## Runtime inventory

* [x] Current firewall hash.
* [x] Rule count.
* [x] Active blocks.
* [x] Docker inventory.
* [x] Systemd summary.
* [x] Network backend.
* [x] Pending change plan.

## Inventory transport

* [x] Payload version.
* [x] Full inventory.
* [x] Delta inventory.
* [x] Hash-based change detection.
* [x] Payload size limit.
* [x] Compression nếu cần.
* [x] Controller storage.

## Gate Phase 14

* [x] Controller hiển thị node state.
* [x] Node offline được phát hiện.
* [x] Docker không có thì agent vẫn healthy.
* [x] Inventory không gửi full payload liên tục khi không đổi.

---

# Middle Phase 15 — Network Management

## Backend detection

* [x] Detect NetworkManager.
* [x] Detect systemd-networkd.
* [x] Unsupported backend chuyển read-only.
* [x] Không trộn hai backend ghi cùng lúc.

## Network models

* [x] `NetworkProfile`.
* [x] `InterfaceProfile`.
* [x] `AddressConfig`.
* [x] `RouteConfig`.
* [x] `DnsConfig`.
* [x] `VlanConfig`.
* [x] `BridgeConfig`.
* [x] `InterfaceRole`.

## Interface roles

* [x] WAN.
* [x] LAN.
* [x] MANAGEMENT.
* [x] STORAGE.
* [x] UNSPECIFIED.

## NetworkManager adapter

* [x] D-Bus connection.
* [x] Inspect profiles.
* [x] DHCP.
* [x] Static IPv4.
* [x] Static IPv6.
* [x] DNS.
* [x] Gateway.
* [x] Static route.
* [x] MTU.
* [x] Stage profile.
* [x] Activate profile.
* [x] Rollback profile.

## systemd-networkd adapter

* [x] Inspect `.network`.
* [x] Inspect `.netdev`.
* [x] Staging directory.
* [x] Atomic replacement.
* [x] DHCP.
* [x] Static address.
* [x] DNS.
* [x] Gateway.
* [x] Route.
* [x] VLAN.
* [x] Bridge.
* [x] Reload networkd.
* [x] Rollback files.

## Safe network apply

* [x] Snapshot current state.
* [x] Stage candidate state.
* [x] Create local rollback timer.
* [x] Apply temporary management firewall rule.
* [x] Activate candidate.
* [x] Probe gateway.
* [x] Probe controller.
* [x] Probe DNS.
* [x] Confirm.
* [x] Automatic rollback.

## Management protection

* [x] Không disable active management interface.
* [x] Không remove current management IP nếu không có fallback.
* [x] Không xóa default route duy nhất không cảnh báo.
* [x] Critical changes yêu cầu explicit confirmation.
* [x] Local rollback không phụ thuộc controller.

## Gate Phase 15

* [x] Đổi static IP thành công.
* [x] Mất controller sau đổi IP thì tự rollback.
* [x] VLAN hoạt động.
* [x] Bridge hoạt động.
* [x] Unsupported backend chỉ read-only.

---

# Middle Phase 16 — Systemd Management

## Systemd integration

* [x] D-Bus client.
* [x] List units.
* [x] Inspect unit.
* [x] Start.
* [x] Stop.
* [x] Restart.
* [x] Reload.
* [x] Enable.
* [x] Disable.

## Typed operation

* [x] Không nhận command string.
* [x] Validate unit name.
* [x] Validate operation.
* [x] Operation timeout.
* [x] Capture result.
* [x] Audit operation.

## Allowlist

* [x] Configurable allowed units.
* [x] Deny unit ngoài allowlist.
* [x] Group-level service policy.
* [x] Node-level override.

## Protected units

* [x] `aegisnode-agent.service`.
* [x] `dbus.service`.
* [x] `systemd-logind.service`.
* [x] `NetworkManager.service`.
* [x] `systemd-networkd.service`.
* [x] SSH yêu cầu elevated confirmation.
* [x] Không cho self-stop qua remote API.

## Journald viewer

* [x] Filter unit.
* [x] Filter priority.
* [x] Filter time range.
* [x] Cursor pagination.
* [x] Result limit.
* [x] Redact sensitive fields nếu cần.

## Desired service state

* [x] Running.
* [x] Stopped.
* [x] Enabled.
* [x] Disabled.
* [x] Drift detection.
* [x] Optional reconcile.

## Gate Phase 16

* [x] Không thể chạy arbitrary shell.
* [x] Protected unit không bị stop.
* [x] Journal query có pagination.
* [x] Mọi service operation có audit.

---

# Middle Phase 17 — Combined Change Plan

## Models

* [x] `NodeChangePlan`.
* [x] `FirewallChange`.
* [x] `NetworkChange`.
* [x] `ServiceChange`.
* [x] `HealthCheckSpec`.
* [x] `RollbackPolicy`.
* [x] `ExecutionStep`.
* [x] `ExecutionResult`.

## Planner

* [x] Resolve dependency.
* [x] Detect circular dependency.
* [x] Generate execution order.
* [x] Generate rollback order.
* [x] Generate dry-run diff.
* [x] Calculate risk.

## Risk levels

* [x] LOW.
* [x] MEDIUM.
* [x] HIGH.
* [x] CRITICAL.

## Execution order

* [x] Snapshot firewall.
* [x] Snapshot network.
* [x] Snapshot service state.
* [x] Install temporary management allow.
* [x] Stage network.
* [x] Stage firewall.
* [x] Activate network.
* [x] Activate firewall.
* [x] Execute service operations.
* [x] Run health checks.
* [x] Confirm hoặc rollback.

## Idempotency

* [x] Mỗi plan có idempotency key.
* [x] Retry không execute hai lần.
* [x] Persist execution state.
* [x] Resume sau agent restart.
* [x] Reject conflicting execution.

## Gate Phase 17

* [x] Network và firewall thay đổi đồng bộ.
* [x] Mất controller thì node tự rollback.
* [x] Failure ở giữa plan rollback đúng reverse order.
* [x] Duplicate request không chạy hai lần.

---

# Middle Phase 17.5 — Transport Refactor: gRPC & IPC

## Mục tiêu

Thay thế toàn bộ HTTP REST nội bộ bằng gRPC (tonic) over mTLS cho giao tiếp xuyên mạng và Unix Domain Socket IPC cho giao tiếp nội bộ cùng máy. REST chỉ còn tồn tại ở lớp Controller HTTP API phục vụ Web UI.

## Proto definitions

* [ ] Định nghĩa `agent.proto` cho AgentService.
* [ ] Định nghĩa `controller.proto` cho ControllerService.
* [ ] `build.rs` tự động gen code từ `.proto` bằng `tonic-build`.
* [ ] Các message types protobuf ánh xạ 1-1 với domain models hiện tại.

## AgentService (gRPC server chạy trên Agent)

* [ ] `ApplyFirewallPolicy(FirewallPolicyRequest) → ApplyResult`.
* [ ] `GetInventory(Empty) → NodeInventoryPayload`.
* [ ] `ExecuteServiceOp(ServiceOpRequest) → ServiceOpResult`.
* [ ] `StreamRolloutCommands(Empty) → stream RolloutCommand` (bidirectional push từ Controller).
* [ ] `GetStatus(Empty) → AgentStatusResponse`.

## ControllerService (gRPC server chạy trên Controller)

* [ ] `ReportHeartbeat(HeartbeatRequest) → HeartbeatAck`.
* [ ] `ReportInventory(NodeInventoryPayload) → Empty`.
* [ ] `ReportRolloutStatus(RolloutStatusUpdate) → Empty`.

## Transport layer Agent

* [ ] gRPC server (tonic) over **Unix Domain Socket** nhận lệnh từ CLI (IPC cùng máy).
* [ ] gRPC client (tonic) over **TCP mTLS** gửi heartbeat và nhận lệnh từ Controller.
* [ ] Loại bỏ HTTP REST listener trên Agent.

## Transport layer Controller

* [ ] gRPC server (tonic) over **TCP mTLS** nhận data từ Agents.
* [ ] REST server (axum) over **HTTPS** giữ nguyên chỉ phục vụ Web UI và Admin API.
* [ ] gRPC client (tonic) over **TCP mTLS** push RolloutCommand xuống các Agents.

## CLI IPC refactor

* [ ] CLI dùng tonic gRPC client over **Unix Domain Socket** thay cho reqwest HTTP.
* [ ] Loại bỏ dependency `reqwest` khỏi `aegis-cli`.
* [ ] Cùng `.proto` types cho cả CLI và Controller client.

## mTLS integration

* [ ] tonic TLS config dùng cert đã sign từ PKI Manager (Phase 13).
* [ ] Server-side: load `ca.crt`, `server.crt`, `server.key`.
* [ ] Client-side: load `ca.crt`, `agent.crt`, `agent.key`.
* [ ] Reject kết nối không có cert hợp lệ.

## Backward compatibility

* [ ] REST Admin API (`/v1/*`) giữ nguyên 100%.
* [ ] Không thay đổi domain models (`aegis-models`).
* [ ] Không thay đổi business logic (`aegis-firewall`, `aegis-core`).

## Gate Phase 17.5

* [ ] CLI → Agent qua UDS gRPC hoạt động.
* [ ] Agent → Controller qua gRPC mTLS hoạt động.
* [ ] Controller push RolloutCommand → Agent qua streaming hoạt động.
* [ ] REST Web UI API không bị ảnh hưởng.
* [ ] Không còn HTTP endpoint nào trên Agent process.

---

# Middle Phase 18 — Multi-Node Rollout


## Rollout models

* [ ] Rollout ID.
* [ ] Strategy.
* [ ] Targets.
* [ ] Current batch.
* [ ] Node status.
* [ ] Failure threshold.
* [ ] Previous version.

## Strategies

* [ ] Manual.
* [ ] All-at-once.
* [ ] Canary.
* [ ] Batch.

## Canary

* [ ] Select canary node.
* [ ] Apply canary.
* [ ] Wait health gate.
* [ ] Continue hoặc stop.
* [ ] Manual approval option.

## Batch

* [ ] Configurable batch size.
* [ ] Maximum unavailable.
* [ ] Pause duration.
* [ ] Health gate.
* [ ] Failure threshold.

## Control

* [ ] Pause.
* [ ] Resume.
* [ ] Cancel.
* [ ] Rollback.
* [ ] Retry failed node.
* [ ] Skip node với audit.

## Coordination

* [ ] Không rollout conflict trên cùng node.
* [ ] Controller restart phục hồi rollout.
* [ ] Agent offline được xử lý.
* [ ] Node quay lại online được reconcile.
* [ ] Rollback về version trước.

## Gate Phase 18

* [ ] Canary fail thì không rollout node còn lại.
* [ ] Batch tuân thủ maximum unavailable.
* [ ] Controller restart không duplicate rollout.
* [ ] Rollback fleet hoạt động.

---

# Middle Phase 19 — Central Web UI and Observability

## Authentication UI

* [ ] Login.
* [ ] Logout.
* [ ] Session expiration.
* [ ] Token refresh.
* [ ] Viewer role.
* [ ] Admin role.

## Fleet UI

* [ ] Node list.
* [ ] Online/offline.
* [ ] Group.
* [ ] Labels.
* [ ] OS.
* [ ] Policy version.
* [ ] Health.
* [ ] Pending changes.

## Node detail

* [ ] Overview.
* [ ] Firewall.
* [ ] Network.
* [ ] Routes.
* [ ] Services.
* [ ] Docker.
* [ ] Blocks.
* [ ] Audit.

## Change plan UI

* [ ] Create.
* [ ] Validate.
* [ ] Diff.
* [ ] Risk.
* [ ] Apply.
* [ ] Confirm.
* [ ] Rollback.
* [ ] Execution timeline.

## Rollout UI

* [ ] Target selector.
* [ ] Strategy.
* [ ] Canary progress.
* [ ] Batch progress.
* [ ] Failure details.
* [ ] Pause.
* [ ] Resume.
* [ ] Rollback.

## Metrics

* [ ] Controller connected agents.
* [ ] Agent health.
* [ ] Apply total.
* [ ] Rollback total.
* [ ] Rollout duration.
* [ ] Rollout failure.
* [ ] Firewall drops.
* [ ] Blocklist size.
* [ ] Interface RX/TX.
* [ ] Service operation failure.

## Grafana

* [ ] Fleet health dashboard.
* [ ] Firewall activity dashboard.
* [ ] Rollout dashboard.
* [ ] Agent connectivity dashboard.
* [ ] Blocker dashboard.

## Middle Stage Gate

Middle chỉ hoàn thành khi:

* [ ] Controller quản lý nhiều node.
* [ ] mTLS hoạt động.
* [ ] Enrollment an toàn.
* [ ] Node offline giữ last-known-good state.
* [ ] NetworkManager được hỗ trợ.
* [ ] systemd-networkd được hỗ trợ.
* [ ] Static IP và DHCP hoạt động.
* [ ] VLAN hoạt động.
* [ ] Bridge hoạt động.
* [ ] Systemd service management hoạt động.
* [ ] Journald viewer hoạt động.
* [ ] Combined change plan hoạt động.
* [ ] Network rollback hoạt động.
* [ ] Canary rollout hoạt động.
* [ ] Central Web UI hoạt động.
* [ ] Central audit hoạt động.
* [ ] Prometheus metrics hoạt động.

---

# Stage 3 — Prod Final

## Mục tiêu

AegisNode đạt mức production với privilege separation, OIDC, RBAC, approval, signed bundles, HA controller, disaster recovery, advanced networking, observability và hardening.

---

# Prod Phase 20 — Privilege Separation

## Process separation

* [ ] `aegisnode server` chạy non-root.
* [ ] `aegisnode agent` chạy non-root.
* [ ] `aegisnode execd` chạy với quyền tối thiểu.
* [ ] `execd` không mở TCP listener.
* [ ] `execd` chỉ dùng Unix socket.

## Executor protocol

* [ ] Typed request enum.
* [ ] Typed response enum.
* [ ] Protocol version.
* [ ] Payload size limit.
* [ ] Request timeout.
* [ ] Peer credential validation.
* [ ] Không nhận raw shell.
* [ ] Không nhận arbitrary path.
* [ ] Không nhận arbitrary environment variable.

## Executor operations

* [ ] Firewall inspect.
* [ ] Firewall check.
* [ ] Firewall apply.
* [ ] Firewall snapshot.
* [ ] Firewall rollback.
* [ ] Network inspect.
* [ ] Network stage.
* [ ] Network activate.
* [ ] Network rollback.
* [ ] Service inspect.
* [ ] Service start.
* [ ] Service stop.
* [ ] Service restart.
* [ ] Service reload.

## Hardening

* [ ] `NoNewPrivileges`.
* [ ] `ProtectSystem`.
* [ ] `ProtectHome`.
* [ ] `PrivateTmp`.
* [ ] `CapabilityBoundingSet`.
* [ ] `RestrictAddressFamilies`.
* [ ] `SystemCallFilter` nếu phù hợp.
* [ ] Socket permission chặt.
* [ ] Local executor audit.

## Gate Phase 20

* [ ] Agent non-root vẫn hoạt động.
* [ ] Executor không thể chạy arbitrary command.
* [ ] Executor không truy cập network bên ngoài.
* [ ] Compromise agent không trực tiếp có shell root.

---

# Prod Phase 21 — OIDC, RBAC and Approval

## OIDC

* [ ] Authorization Code Flow.
* [ ] PKCE.
* [ ] Issuer validation.
* [ ] Audience validation.
* [ ] JWKS validation.
* [ ] Token expiry.
* [ ] Clock skew handling.
* [ ] Logout flow.

## Roles

* [ ] Viewer.
* [ ] Operator.
* [ ] SecurityAdmin.
* [ ] PlatformAdmin.
* [ ] Auditor.

## Permissions

* [ ] `nodes.read`.
* [ ] `firewall.read`.
* [ ] `firewall.write`.
* [ ] `firewall.apply`.
* [ ] `network.read`.
* [ ] `network.write`.
* [ ] `service.read`.
* [ ] `service.restart`.
* [ ] `change_plan.approve`.
* [ ] `rollout.manage`.
* [ ] `audit.read`.
* [ ] `admin.manage`.

## Scope

* [ ] Global.
* [ ] Node group.
* [ ] Specific node.
* [ ] Environment.
* [ ] Label selector.

## Approval workflow

* [ ] LOW không cần approval.
* [ ] MEDIUM cần operator.
* [ ] HIGH cần security admin.
* [ ] CRITICAL cần hai người.
* [ ] Creator không tự approve critical plan.
* [ ] Approval expiration.
* [ ] Re-approval nếu plan thay đổi.

## Break-glass

* [ ] Disabled mặc định.
* [ ] Time-limited.
* [ ] Strong audit.
* [ ] Notification.
* [ ] Rotation procedure.

## Gate Phase 21

* [ ] Viewer không apply.
* [ ] Operator không sửa RBAC.
* [ ] Critical plan cần đủ approval.
* [ ] Mọi deny/allow decision có audit.

---

# Prod Phase 22 — Signed Policy Bundles

## Bundle contents

* [ ] Metadata.
* [ ] Firewall policy.
* [ ] Network profile.
* [ ] Service policy.
* [ ] Execution plan.
* [ ] Checksums.
* [ ] Signature.

## Metadata

* [ ] Bundle ID.
* [ ] Target node ID.
* [ ] Policy version.
* [ ] Issued time.
* [ ] Expiry.
* [ ] Sequence.
* [ ] Controller identity.

## Signing

* [ ] Controller signing key.
* [ ] Private key không nằm trong source.
* [ ] Public key distribution.
* [ ] Secure key permission.
* [ ] Optional external secret manager.

## Agent verification

* [ ] Verify signature.
* [ ] Verify checksum.
* [ ] Verify target node.
* [ ] Verify expiry.
* [ ] Verify sequence.
* [ ] Replay protection.
* [ ] Reject unknown key.

## Rotation

* [ ] Multiple trusted public keys.
* [ ] Grace period.
* [ ] Key revocation.
* [ ] Rotation audit.

## Gate Phase 22

* [ ] Bundle sửa một byte bị reject.
* [ ] Bundle node khác bị reject.
* [ ] Bundle hết hạn bị reject.
* [ ] Replay bundle cũ bị reject.
* [ ] Key rotation không gián đoạn rollout.

---

# Prod Phase 23 — Controller High Availability

## Statelessness

* [ ] API stateless.
* [ ] Session externalized hoặc signed.
* [ ] Không giữ rollout state chỉ trong memory.
* [ ] Không giữ certificate state chỉ trong memory.

## Multiple replicas

* [ ] Chạy nhiều controller.
* [ ] Reverse proxy hoặc load balancer.
* [ ] Health check.
* [ ] Readiness check.
* [ ] Graceful shutdown.
* [ ] Rolling upgrade.

## Database

* [ ] PostgreSQL HA deployment guide.
* [ ] Connection pooling.
* [ ] Retry policy.
* [ ] Transaction timeout.
* [ ] Read-only degraded mode.

## Distributed coordination

* [ ] Distributed lock.
* [ ] Rollout leader.
* [ ] Certificate issuance lock.
* [ ] Background job leader election.
* [ ] Prevent duplicate execution.

## Agent failover

* [ ] Multiple controller endpoints.
* [ ] Endpoint rotation.
* [ ] Backoff.
* [ ] Jitter.
* [ ] Certificate pinning hoặc CA verification.
* [ ] Agent hoạt động khi toàn controller offline.

## Gate Phase 23

* [ ] Kill một controller không mất control plane.
* [ ] Rollout không chạy hai lần.
* [ ] Agent reconnect controller khác.
* [ ] Database tạm lỗi không làm agent mất policy.

---

# Prod Phase 24 — Advanced Networking

## Bonding

* [ ] Active-backup.
* [ ] 802.3ad nếu backend hỗ trợ.
* [ ] Member validation.
* [ ] Rollback.
* [ ] Management protection.

## VRF

* [ ] Inventory.
* [ ] Create.
* [ ] Attach interface.
* [ ] Route table association.
* [ ] Delete validation.
* [ ] Rollback.

## Policy routing

* [ ] Multiple routing tables.
* [ ] Source-based rule.
* [ ] Interface-based rule.
* [ ] Priority validation.
* [ ] Conflict detection.
* [ ] Rollback.

## Multiple gateways

* [ ] Route metric.
* [ ] Default route conflict detection.
* [ ] Gateway reachability.
* [ ] Failover design documentation.

## WireGuard

* [ ] Read-only inventory.
* [ ] Peer status.
* [ ] Traffic counters.
* [ ] Optional write support sau feature gate.
* [ ] Không log private key.

## HA gateway integration

* [ ] Keepalived status.
* [ ] VRRP state.
* [ ] Virtual IP inventory.
* [ ] Safe service reload.
* [ ] Không tự implement VRRP.

## Gate Phase 24

* [ ] Backend unsupported thì reject trước apply.
* [ ] Advanced network change rollback được.
* [ ] Management protection vẫn hoạt động.
* [ ] UI topology hiển thị đúng.

---

# Prod Phase 25 — Audit, Backup and Recovery

## Audit

* [ ] Actor.
* [ ] Action.
* [ ] Resource.
* [ ] Before hash.
* [ ] After hash.
* [ ] Request ID.
* [ ] Source IP.
* [ ] Timestamp.
* [ ] Result.
* [ ] Node ID.
* [ ] Execution ID.

## Integrity chain

* [ ] Event hash.
* [ ] Previous event hash.
* [ ] Chain verification.
* [ ] Detect modified event.
* [ ] Detect missing event.
* [ ] Verification CLI.

## Audit export

* [ ] JSON.
* [ ] Syslog.
* [ ] Log pipeline integration.
* [ ] Export retry.
* [ ] Local buffer.
* [ ] Buffer size limit.

## Controller backup

* [ ] PostgreSQL full backup.
* [ ] Point-in-time recovery guide.
* [ ] Config backup.
* [ ] Policy bundle archive.
* [ ] CA metadata.
* [ ] Signing public keys.
* [ ] Node enrollment records.

## Restore

* [ ] Restore test environment.
* [ ] Restore database.
* [ ] Restore controller config.
* [ ] Restore policy history.
* [ ] Verify agent reconnect.
* [ ] Verify audit integrity.

## Local node recovery

* [ ] `aegisnode ctl recovery status`.
* [ ] `rollback-last`.
* [ ] `restore-snapshot`.
* [ ] Offline emergency mode.
* [ ] Local actor audit.
* [ ] Sync recovery event về controller sau reconnect.

## Gate Phase 25

* [ ] Restore controller từ backup thành công.
* [ ] Agent giữ firewall trong thời gian disaster.
* [ ] Audit tampering được phát hiện.
* [ ] Local emergency rollback hoạt động.

---

# Prod Phase 26 — Production Observability

## Metrics

* [ ] HTTP request rate.
* [ ] HTTP latency.
* [ ] Database latency.
* [ ] Connected agents.
* [ ] Offline agents.
* [ ] Apply duration.
* [ ] Apply failure.
* [ ] Rollback total.
* [ ] Rollout duration.
* [ ] Rollout failure.
* [ ] Certificate renewal failure.
* [ ] Blocklist size.
* [ ] Firewall counter.
* [ ] Network apply failure.
* [ ] Service operation failure.

## Tracing

* [ ] Web request trace.
* [ ] Controller trace.
* [ ] Rollout trace.
* [ ] Agent request trace.
* [ ] Executor operation trace.
* [ ] Correlation ID xuyên suốt.
* [ ] Không trace packet.

## Alerts

* [ ] Agent offline.
* [ ] Controller degraded.
* [ ] Database unavailable.
* [ ] Rollback occurred.
* [ ] Policy apply failed.
* [ ] Rollout halted.
* [ ] Certificate expiring.
* [ ] Audit export failed.
* [ ] Blocklist unusually large.
* [ ] Drop count anomaly.

## Runbooks

* [ ] Agent offline.
* [ ] Controller unavailable.
* [ ] Database unavailable.
* [ ] Policy rollout stuck.
* [ ] Network rollback.
* [ ] Certificate renewal failure.
* [ ] Emergency local access.
* [ ] Audit chain failure.

## Capacity tests

* [ ] Maximum connected agents.
* [ ] Heartbeat throughput.
* [ ] Inventory throughput.
* [ ] Concurrent apply.
* [ ] Concurrent rollout.
* [ ] Audit throughput.
* [ ] Database connection saturation.

## Soak tests

* [ ] Controller restart loop.
* [ ] Agent reconnect.
* [ ] Network flapping.
* [ ] Docker churn.
* [ ] Repeated policy apply.
* [ ] Long-running blocker.
* [ ] Database failover.

## Gate Phase 26

* [ ] Dashboard đầy đủ.
* [ ] Alerts có runbook.
* [ ] Capacity limit được ghi lại.
* [ ] Soak test không có memory leak nghiêm trọng.

---

# Prod Phase 27 — Security Hardening and Release

## Threat model

* [ ] Compromised user account.
* [ ] Compromised Web session.
* [ ] Compromised controller.
* [ ] Compromised agent.
* [ ] Compromised executor socket.
* [ ] Malicious policy.
* [ ] MITM.
* [ ] Replay.
* [ ] Self-lockout.
* [ ] Audit tampering.
* [ ] Denial of service.

## Dependency security

* [ ] `cargo audit`.
* [ ] `cargo deny`.
* [ ] `npm audit`.
* [ ] License check.
* [ ] Vulnerability exception process.
* [ ] Dependency update policy.

## Fuzzing

* [ ] YAML parser.
* [ ] JSON parser.
* [ ] CIDR parser.
* [ ] Port parser.
* [ ] Policy validator.
* [ ] Bundle parser.
* [ ] Executor protocol.
* [ ] Change-plan parser.

## API hardening

* [ ] Request size limit.
* [ ] Rule count limit.
* [ ] Policy size limit.
* [ ] Rate limit.
* [ ] Timeout.
* [ ] Pagination.
* [ ] Concurrent request limit.
* [ ] CORS policy.
* [ ] Security headers.

## Runtime limits

* [ ] Memory limits.
* [ ] Open file limits.
* [ ] Audit buffer limits.
* [ ] Journald detector limits.
* [ ] Maximum active block count.
* [ ] Maximum pending execution count.
* [ ] Maximum snapshot storage.

## Compatibility matrix

* [ ] Ubuntu LTS.
* [ ] Debian stable.
* [ ] RHEL-compatible distro.
* [ ] NetworkManager.
* [ ] systemd-networkd.
* [ ] Multiple nftables versions.
* [ ] Multiple Docker versions.
* [ ] amd64.
* [ ] arm64.

## Release artifacts

* [ ] Single `aegisnode` binary.
* [ ] `.deb`.
* [ ] `.rpm`.
* [ ] Container image cho controller.
* [ ] Checksums.
* [ ] SBOM.
* [ ] Signed artifacts.
* [ ] Release notes.
* [ ] Upgrade guide.
* [ ] Downgrade limitations.

## Documentation

* [ ] Installation.
* [ ] Quick start.
* [ ] CLI reference.
* [ ] API reference.
* [ ] Firewall policy.
* [ ] NAT.
* [ ] Docker.
* [ ] Blocker.
* [ ] Network profiles.
* [ ] Systemd management.
* [ ] Change plans.
* [ ] Rollouts.
* [ ] OIDC.
* [ ] RBAC.
* [ ] Backup.
* [ ] Recovery.
* [ ] Troubleshooting.
* [ ] Security model.

## Prod Final Stage Gate

AegisNode chỉ được đánh dấu Production Final khi:

* [ ] Controller HA hoạt động.
* [ ] Agent chạy non-root.
* [ ] Executor được giới hạn đặc quyền.
* [ ] Executor không nhận raw command.
* [ ] mTLS hoạt động.
* [ ] OIDC hoạt động.
* [ ] RBAC hoạt động.
* [ ] Approval workflow hoạt động.
* [ ] Signed bundles hoạt động.
* [ ] Replay protection hoạt động.
* [ ] Canary rollout hoạt động.
* [ ] Fleet rollback hoạt động.
* [ ] Network rollback hoạt động.
* [ ] Backup và restore đã được kiểm thử.
* [ ] Offline node recovery hoạt động.
* [ ] Audit integrity chain hoạt động.
* [ ] Monitoring đầy đủ.
* [ ] Alerts có runbook.
* [ ] Security review hoàn thành.
* [ ] Compatibility tests pass.
* [ ] Upgrade không làm mất firewall policy.
* [ ] Release artifacts được ký.
* [ ] Documentation hoàn chỉnh.

---

# Tổng hợp tiến độ

## Stage MVP

* [x] Phase 0 — Repository Skeleton.
* [x] Phase 1 — Core Models and Errors.
* [x] Phase 2 — Policy Validation.
* [x] Phase 3 — nftables Compiler.
* [x] Phase 4 — nftables Runtime Backend.
* [x] Phase 5 — Safe Apply and Rollback.
* [x] Phase 6 — Local Agent and Storage.
* [x] Phase 7 — CLI.
* [x] Phase 8 — Docker and Router Mode.
* [x] Phase 9 — Blocker and Journald.
* [ ] Phase 10 — Web UI.
* [ ] Phase 11 — Packaging and E2E.
* [ ] MVP Stage Gate passed.

## Stage Middle

* [ ] Phase 12 — Controller Foundation.
* [ ] Phase 13 — Enrollment and mTLS.
* [ ] Phase 14 — Node Inventory.
* [ ] Phase 15 — Network Management.
* [ ] Phase 16 — Systemd Management.
* [ ] Phase 17 — Combined Change Plan.
* [ ] Phase 18 — Multi-Node Rollout.
* [ ] Phase 19 — Central Web UI and Observability.
* [ ] Middle Stage Gate passed.

## Stage Prod Final

* [ ] Phase 20 — Privilege Separation.
* [ ] Phase 21 — OIDC, RBAC and Approval.
* [ ] Phase 22 — Signed Policy Bundles.
* [ ] Phase 23 — Controller High Availability.
* [ ] Phase 24 — Advanced Networking.
* [ ] Phase 25 — Audit, Backup and Recovery.
* [ ] Phase 26 — Production Observability.
* [ ] Phase 27 — Security Hardening and Release.
* [ ] Prod Final Stage Gate passed.

---

# Quy tắc chuyển stage

## MVP → Middle

Không bắt đầu Middle nếu:

* [ ] Safe apply chưa ổn định.
* [ ] Rollback chưa sống qua daemon restart.
* [ ] nftables compiler chưa deterministic.
* [ ] Docker rules vẫn có nguy cơ bị flush.
* [ ] NAT chưa có E2E test.
* [ ] CLI chưa quản lý được toàn bộ local node.
* [ ] Web UI chưa quản lý được pending execution.
* [ ] Reboot test chưa pass.

## Middle → Prod Final

Không bắt đầu Prod Final nếu:

* [ ] mTLS chưa ổn định.
* [ ] Agent offline chưa giữ được last-known-good state.
* [ ] Network safe apply chưa hoạt động.
* [ ] Management interface protection chưa hoạt động.
* [ ] Combined change plan chưa rollback đúng.
* [ ] Rollout chưa có idempotency.
* [ ] Canary failure chưa tự dừng rollout.
* [ ] Controller restart còn làm duplicate operation.

---

# Thứ tự ưu tiên

```text
1. Safety
2. Correctness
3. Rollback
4. Security
5. Observability
6. Performance
7. User interface
```

* [ ] Không ưu tiên giao diện đẹp trước safe apply.
* [ ] Không tối ưu compiler trước khi output đúng.
* [ ] Không thêm advanced networking trước khi network rollback ổn định.
* [ ] Không thêm HA trước khi operation idempotent.
* [ ] Không thêm plugin hoặc WASM trước Production Final.
* [ ] Không thêm eBPF nếu nftables đã đáp ứng yêu cầu.
