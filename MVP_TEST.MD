# AegisNode Phase 11 — End-to-End Qualification Checklist

> Mục tiêu: kiểm thử binary release, package, systemd service và toàn bộ luồng vận hành MVP trên môi trường Linux thật trước khi chuyển sang Stage Middle.

---

# 1. Phạm vi Phase 11

Phase 11 không chỉ kiểm tra “chạy được”.

Nó phải chứng minh:

* Binary release có thể cài đặt.
* Package không phá hệ thống hiện có.
* Systemd khởi động, dừng và restart đúng.
* AegisNode không rơi vào restart loop.
* Firewall vẫn tồn tại khi daemon crash.
* Safe apply và rollback hoạt động khi mất kết nối.
* Reboot không làm mất desired state.
* Docker networking không bị phá.
* NAT và port forwarding hoạt động.
* SQLite, snapshot và audit có khả năng phục hồi.
* Input xấu không làm process panic.
* Tải cao không làm memory hoặc file descriptor tăng vô hạn.
* Upgrade không làm mất policy.
* Uninstall không phá ruleset không thuộc AegisNode.
* Binary tương thích với các distribution mục tiêu.

---

# 2. Nguyên tắc môi trường test

## 2.1 Phải dùng VM thật

Không dùng container làm môi trường E2E chính cho systemd và network.

Mỗi test image phải boot bằng systemd thật:

```text
QEMU/KVM hoặc cloud VM
        ↓
Linux kernel thật
        ↓
systemd PID 1
        ↓
nftables thật
        ↓
AegisNode package thật
```

Container chỉ được dùng cho:

* Unit test.
* API test.
* Compiler test.
* Frontend test.
* PostgreSQL/Docker fixture phụ trợ.

## 2.2 Mỗi test suite bắt đầu từ clean image

Mỗi run phải:

* Clone image sạch.
* Không giữ database từ run trước.
* Không giữ nftables rules từ run trước.
* Không giữ journal cursor từ run trước.
* Không reuse `/var/lib/aegisnode`.
* Không reuse `/etc/aegisnode`.
* Snapshot VM trước khi cài package.
* Destroy VM sau khi thu thập artifact.

## 2.3 Không chạy destructive test trên máy developer

Các test sau chỉ được chạy trong VM hoặc network namespace riêng:

* Thay default input policy thành `drop`.
* Bật IP forwarding.
* DNAT/SNAT.
* Kill daemon.
* Làm đầy disk.
* Corrupt SQLite.
* Reboot.
* Xóa nftables table.
* Restart Docker.
* Thay đổi system clock.
* OOM injection.

---

# 3. Compatibility Matrix

Tính đến tháng 8 năm 2026, Ubuntu 26.04 là LTS mới nhất, Ubuntu 24.04 vẫn nằm trong vòng hỗ trợ; Debian 13 là stable hiện tại. RHEL 10 cung cấp cả firewalld và nftables, vì vậy nhánh Red Hat phải kiểm thử cả phát hiện xung đột và chế độ coexistence.

## 3.1 Tier 1 — Release blocker

Mọi test bắt buộc phải pass:

* [ ] Ubuntu Server 24.04 LTS — amd64.
* [ ] Ubuntu Server 26.04 LTS — amd64.
* [ ] Debian 13 — amd64.
* [ ] Ubuntu Server 24.04 LTS — arm64.
* [ ] Debian 13 — arm64.

## 3.2 Tier 2 — Compatibility

Không nhất thiết chạy trên mọi pull request nhưng bắt buộc trước release:

* [ ] Ubuntu Server 22.04 LTS — amd64.
* [ ] Debian 12 — amd64.
* [ ] RHEL 9 hoặc distribution tương thích — amd64.
* [ ] RHEL 10 hoặc distribution tương thích — amd64.
* [ ] Ubuntu 24.04 với kernel HWE.
* [ ] VM chỉ có IPv4.
* [ ] VM dual-stack IPv4/IPv6.
* [ ] VM không cài Docker.
* [ ] VM có Docker dùng backend iptables-nft.
* [ ] VM có Docker dùng backend nftables nếu phiên bản Docker hỗ trợ.

## 3.3 Thông tin phải lưu cho mỗi test run

```bash
uname -a
cat /etc/os-release
systemd --version
nft --version
docker version
ip -Version
aegisnode version
```

Artifact metadata phải chứa:

* OS name và version.
* Kernel version.
* Architecture.
* systemd version.
* nftables version.
* Docker version và firewall backend.
* AegisNode version.
* Git commit.
* Package checksum.
* Thời điểm chạy test.

---

# 4. Test Topology

## 4.1 Firewall server topology

```text
Client Namespace
       │
       │ veth
       ▼
AegisNode VM
├── SSH test service
├── HTTP test service
├── UDP echo service
└── nftables
```

## 4.2 Router topology

```text
WAN namespace
      │
    veth-wan
      │
      ▼
AegisNode router VM
      │
    veth-lan
      │
      ▼
LAN namespace
```

## 4.3 DNAT topology

```text
WAN client
    │
    ▼
AegisNode WAN:8443
    │ DNAT
    ▼
LAN server:443
```

## 4.4 Docker topology

```text
External namespace
        │
        ▼
AegisNode host
├── Docker bridge
├── nginx container
├── PostgreSQL container
└── Redis container
```

---

# 5. Global Pass Criteria

Mỗi test case phải kiểm tra các invariant sau.

## 5.1 Process invariant

Sau test bình thường:

```bash
systemctl show aegisnode-local \
  -p ActiveState \
  -p SubState \
  -p ExecMainStatus \
  -p Result \
  -p NRestarts
```

Kết quả yêu cầu:

* [ ] `ActiveState=active`.
* [ ] `SubState=running`.
* [ ] `ExecMainStatus=0`.
* [ ] `Result=success`.
* [ ] `NRestarts=0` nếu test không chủ động crash process.
* [ ] Không có zombie process.
* [ ] Chỉ có một instance daemon.
* [ ] Unix socket thuộc đúng user/group.
* [ ] HTTP listener chỉ bind vào địa chỉ được cấu hình.

## 5.2 Crash invariant

```bash
coredumpctl list aegisnode
```

* [ ] Không có coredump ngoài test cố tình tạo crash.
* [ ] Không có panic stack trace trong journal.
* [ ] Không có `segmentation fault`.
* [ ] Không có `double free`.
* [ ] Không có `stack overflow`.
* [ ] Không có abort không được kiểm soát.

## 5.3 Journal invariant

```bash
journalctl \
  -u aegisnode-local.service \
  -b \
  --priority=warning..alert
```

* [ ] Không có lỗi lặp vô hạn.
* [ ] Không log secret.
* [ ] Không log password/token.
* [ ] Không log raw private configuration không cần thiết.
* [ ] Mỗi operation có request hoặc execution ID.
* [ ] Error có context.
* [ ] Không spam log khi dependency không tồn tại.

## 5.4 Firewall invariant

Trước và sau mỗi test:

```bash
nft -j list ruleset > ruleset.json
```

* [ ] Chỉ table AegisNode được thay đổi.
* [ ] Rule không thuộc AegisNode giữ nguyên.
* [ ] Docker/firewalld rules không biến mất.
* [ ] Ruleset luôn pass `nft --check`.
* [ ] Không xuất hiện table AegisNode trùng.
* [ ] Không xuất hiện chain mồ côi.
* [ ] Không xuất hiện temporary rule sau khi execution kết thúc.
* [ ] Policy hash trong runtime khớp desired state sau commit.

## 5.5 Storage invariant

* [ ] SQLite `PRAGMA integrity_check` trả về `ok`.
* [ ] Không có transaction pending.
* [ ] Không có snapshot thiếu metadata.
* [ ] Không có candidate file mồ côi.
* [ ] Không có world-readable snapshot.
* [ ] Audit event tồn tại cho mọi operation thay đổi state.
* [ ] Disk usage nằm trong giới hạn retention.

---

# 6. Build Artifact Tests

## E2E-BUILD-001 — Clean release build

**Action**

```bash
cargo clean
cargo build --workspace --locked --release
```

**Expected**

* [ ] Build thành công.
* [ ] Không tự sửa `Cargo.lock`.
* [ ] Chỉ tạo một binary chính `aegisnode`.
* [ ] Binary chạy được.
* [ ] `aegisnode version` đúng với release metadata.

## E2E-BUILD-002 — Reproducible source state

* [ ] Build từ clean clone.
* [ ] Không phụ thuộc file ngoài repository.
* [ ] Không phụ thuộc frontend development server.
* [ ] Frontend assets có trong binary.
* [ ] Database migrations có trong package hoặc binary.
* [ ] Default config có trong package.

## E2E-BUILD-003 — Binary dependency inspection

```bash
file /usr/bin/aegisnode
ldd /usr/bin/aegisnode || true
readelf -h /usr/bin/aegisnode
```

* [ ] Architecture đúng.
* [ ] Không phụ thuộc library không được package khai báo.
* [ ] Không chứa debug path tuyệt đối ngoài ý muốn.
* [ ] Binary không cần Node.js.
* [ ] Binary không cần Rust toolchain trên target host.

## E2E-BUILD-004 — Frontend embedded

* [ ] `GET /` trả về `index.html`.
* [ ] JS/CSS assets load được.
* [ ] Reload trực tiếp một SPA route không trả 404.
* [ ] API route không bị SPA fallback nuốt.
* [ ] Production host không cần thư mục `web/dist`.

## E2E-BUILD-005 — Package checksum

* [ ] Sinh SHA-256.
* [ ] Cài package có checksum hợp lệ.
* [ ] Sửa một byte package làm verification thất bại.
* [ ] Package version khớp binary version.

---

# 7. Package Installation Tests

## E2E-PKG-001 — Cài mới trên clean VM

**Action**

```bash
sudo dpkg -i ./aegisnode_<version>_<arch>.deb
```

hoặc package manager tương ứng.

**Expected**

* [ ] User hệ thống được tạo.
* [ ] Group hệ thống được tạo.
* [ ] `/etc/aegisnode` tồn tại.
* [ ] `/var/lib/aegisnode` tồn tại.
* [ ] `/run/aegisnode` được systemd tạo.
* [ ] Permission chính xác.
* [ ] Unit file được cài.
* [ ] Không tự sửa firewall trước khi service khởi động.
* [ ] Không overwrite file cấu hình có sẵn.
* [ ] Package install exit code bằng 0.

## E2E-PKG-002 — Cài lại cùng version

* [ ] Không tạo user trùng.
* [ ] Không reset database.
* [ ] Không reset policy.
* [ ] Không sinh unit file duplicate.
* [ ] Service vẫn hoạt động.
* [ ] Package manager không báo broken state.

## E2E-PKG-003 — Cài khi `nft` chưa tồn tại

* [ ] Package có thể từ chối cài với lỗi rõ ràng, hoặc dependency manager tự cài `nftables`.
* [ ] Không để package half-configured.
* [ ] Không start service trong trạng thái không thể hoạt động.
* [ ] Error hướng dẫn cách khắc phục.

## E2E-PKG-004 — Cài khi config directory đã tồn tại

* [ ] File người dùng không bị ghi đè.
* [ ] Default config được lưu dưới tên rõ ràng nếu cần.
* [ ] Service báo config lỗi rõ ràng.
* [ ] Không panic.

## E2E-PKG-005 — Cài khi state directory permission sai

* [ ] Service fail-fast.
* [ ] Không restart loop.
* [ ] Journal chỉ rõ permission problem.
* [ ] Không tự chmod file tùy tiện ngoài phạm vi quản lý.

## E2E-PKG-006 — Cài package sai architecture

* [ ] Package manager từ chối.
* [ ] Hệ thống không bị thay đổi một phần.
* [ ] Unit không được enable.

---

# 8. Systemd Unit Tests

Systemd hỗ trợ nhiều cơ chế giới hạn môi trường thực thi và hardening trong unit file; các directive phải được kiểm tra trên từng version systemd mục tiêu thay vì giả định mọi distribution hỗ trợ giống nhau.

## E2E-SD-001 — Unit syntax

```bash
systemd-analyze verify \
  /usr/lib/systemd/system/aegisnode-local.service
```

* [ ] Không có syntax error.
* [ ] Không có unknown directive trên Tier 1.
* [ ] Không có dependency cycle.
* [ ] `ExecStart` trỏ đúng binary.
* [ ] User/group tồn tại.

## E2E-SD-002 — Enable service

```bash
sudo systemctl enable aegisnode-local
systemctl is-enabled aegisnode-local
```

* [ ] Trả về `enabled`.
* [ ] Symlink đúng.
* [ ] Enable hai lần vẫn idempotent.

## E2E-SD-003 — Start service

```bash
sudo systemctl start aegisnode-local
systemctl is-active aegisnode-local
```

* [ ] Service active.
* [ ] Startup không vượt budget.
* [ ] Unix socket được tạo.
* [ ] HTTP listener được tạo.
* [ ] SQLite migration hoàn thành.
* [ ] Không thay đổi policy ngoài dự kiến.

## E2E-SD-004 — Stop service

```bash
sudo systemctl stop aegisnode-local
```

* [ ] Process thoát trong `TimeoutStopSec`.
* [ ] Exit code bằng 0.
* [ ] Không để process con.
* [ ] Unix socket được dọn.
* [ ] nftables committed policy vẫn tồn tại.
* [ ] SQLite không corrupt.
* [ ] Không rollback policy đã commit.

## E2E-SD-005 — Restart service

```bash
sudo systemctl restart aegisnode-local
```

* [ ] Service quay lại active.
* [ ] API trở lại.
* [ ] Policy không mất.
* [ ] Blocklist runtime được reconcile.
* [ ] Không tạo table duplicate.
* [ ] Không tăng snapshot vô lý.

## E2E-SD-006 — Reload không được hỗ trợ

Nếu service chưa hỗ trợ reload:

```bash
sudo systemctl reload aegisnode-local
```

* [ ] Trả lỗi rõ ràng.
* [ ] Không crash.
* [ ] Service vẫn active.

Nếu hỗ trợ reload:

* [ ] Config hợp lệ được reload.
* [ ] Config lỗi giữ state cũ.
* [ ] Không restart process không cần thiết.

## E2E-SD-007 — SIGTERM

```bash
sudo systemctl kill \
  --signal=SIGTERM \
  aegisnode-local
```

* [ ] Graceful shutdown.
* [ ] Không corrupt database.
* [ ] Systemd restart theo policy.
* [ ] Process quay lại healthy.
* [ ] Pending execution được phục hồi.

## E2E-SD-008 — SIGKILL

```bash
sudo systemctl kill \
  --signal=SIGKILL \
  aegisnode-local
```

* [ ] Systemd phát hiện failure.
* [ ] Service restart đúng.
* [ ] Không restart storm.
* [ ] Committed firewall vẫn tồn tại.
* [ ] Pending rollback được tiếp tục sau restart.
* [ ] SQLite integrity vẫn `ok`.

## E2E-SD-009 — Nhiều crash liên tục

* [ ] Kill process nhiều lần liên tiếp.
* [ ] Systemd áp dụng start limit.
* [ ] Không fork bomb.
* [ ] Không tiêu tốn CPU 100% vô hạn.
* [ ] Journal không tăng vô hạn trong vài giây.
* [ ] Operator nhận được state `failed`.

## E2E-SD-010 — Start khi port đã bị chiếm

* [ ] Service fail với lỗi bind rõ ràng.
* [ ] Không panic.
* [ ] Không sửa nftables.
* [ ] Không tạo database transaction dang dở.
* [ ] Không restart loop vô hạn.

## E2E-SD-011 — Start khi Unix socket path bị chiếm

Các biến thể:

* File thường tồn tại.
* Socket stale tồn tại.
* Directory không writable.
* Symlink trỏ ra ngoài.

Expected:

* [ ] Stale socket hợp lệ được xử lý an toàn.
* [ ] Không xóa file tùy ý.
* [ ] Không follow symlink nguy hiểm.
* [ ] Service fail-fast khi cần.

## E2E-SD-012 — Boot ordering

* [ ] Service khởi động khi network chưa online.
* [ ] Không crash.
* [ ] Local API vẫn hoạt động nếu có thể.
* [ ] Retry dependency theo backoff.
* [ ] Không restart loop.
* [ ] Khi network xuất hiện, health trở lại healthy.

## E2E-SD-013 — Systemd hardening inspection

```bash
systemd-analyze security aegisnode-local.service
```

* [ ] Báo cáo được lưu làm artifact.
* [ ] Không vô tình cấp quyền vượt yêu cầu.
* [ ] Hardening không chặn directory cần ghi.
* [ ] Service vẫn hoạt động trên mọi Tier 1 distro.

---

# 9. First Boot and Configuration Tests

## E2E-CFG-001 — Default config

* [ ] Service start bằng default config.
* [ ] HTTP chỉ bind `127.0.0.1`.
* [ ] Unix socket permission đúng.
* [ ] Không có default rule nguy hiểm.
* [ ] Không tự bật router mode.
* [ ] Không tự expose Web UI ra WAN.

## E2E-CFG-002 — YAML syntax lỗi

* [ ] Service fail-fast hoặc giữ last-known-good config.
* [ ] Có line/column trong error.
* [ ] Không panic.
* [ ] Không thay đổi firewall.
* [ ] Không overwrite config lỗi.

## E2E-CFG-003 — Unknown config field

* [ ] Hành vi được khóa rõ: reject hoặc warning.
* [ ] Không âm thầm bỏ qua field bảo mật quan trọng.
* [ ] Error ghi rõ field path.

## E2E-CFG-004 — Config quá lớn

* [ ] Từ chối vượt giới hạn.
* [ ] Không OOM.
* [ ] Không block event loop quá lâu.
* [ ] API vẫn responsive.

## E2E-CFG-005 — Config permission quá rộng

* [ ] Có warning hoặc từ chối nếu chứa secret.
* [ ] Không log nội dung secret.
* [ ] Service không tự làm hỏng ownership.

## E2E-CFG-006 — Config thay đổi trong lúc daemon chạy

* [ ] Không tự apply nếu không có reload/reconcile.
* [ ] Operation hiện tại không đọc file nửa chừng.
* [ ] Atomic rename được xử lý.
* [ ] Không panic khi file tạm biến mất.

---

# 10. CLI, API and Web UI E2E

## E2E-API-001 — Health endpoints

* [ ] `/health` trả process health.
* [ ] `/ready` chỉ ready sau migration và subsystem initialization.
* [ ] HTTP status đúng.
* [ ] Response có version.
* [ ] Không lộ internal path.

## E2E-API-002 — CLI qua Unix socket

```bash
aegisnode ctl status --output json
```

* [ ] Exit code 0.
* [ ] JSON hợp lệ.
* [ ] Log không trộn vào stdout.
* [ ] CLI hoạt động khi HTTP disabled.

## E2E-API-003 — Unix socket permission

* [ ] User thuộc group được truy cập.
* [ ] User ngoài group bị từ chối.
* [ ] Không cần root nếu permission đúng.
* [ ] Error CLI rõ ràng.

## E2E-API-004 — Request malformed JSON

* [ ] HTTP 400.
* [ ] Process không panic.
* [ ] Không tạo execution.
* [ ] Audit nếu operation có ý định thay đổi state.

## E2E-API-005 — Request body quá lớn

* [ ] HTTP 413.
* [ ] Connection được xử lý đúng.
* [ ] Memory không tăng vô hạn.
* [ ] Service vẫn responsive.

## E2E-API-006 — Concurrent apply requests

* [ ] Chỉ một execution được nhận.
* [ ] Request còn lại nhận conflict.
* [ ] Không tạo hai snapshot cạnh tranh.
* [ ] Không deadlock.

## E2E-API-007 — Client disconnect giữa request

* [ ] Operation đã nhận có trạng thái xác định.
* [ ] Không tạo execution “mồ côi”.
* [ ] Safe rollback vẫn hoạt động.
* [ ] Không giữ connection/resource leak.

## E2E-API-008 — Web UI load

* [ ] Dashboard load.
* [ ] API errors được hiển thị.
* [ ] Không có uncaught JavaScript error.
* [ ] Refresh giữ pending execution.
* [ ] Countdown rollback dựa trên server deadline, không chỉ browser timer.

## E2E-API-009 — API path traversal

Các payload:

```text
../../etc/shadow
%2e%2e/%2e%2e/etc/passwd
/assets/../../../etc/passwd
```

* [ ] Không đọc file ngoài embedded assets.
* [ ] HTTP 404 hoặc 400.
* [ ] Không panic.

---

# 11. Firewall Functional Tests

## E2E-FW-001 — Default input accept

* [ ] Traffic hợp lệ đi vào.
* [ ] Existing connection không bị reset.
* [ ] Counter cập nhật.

## E2E-FW-002 — Default input drop

* [ ] Port không allow bị drop.
* [ ] Port allow vẫn truy cập được.
* [ ] Local loopback vẫn hoạt động.
* [ ] Established connection xử lý đúng theo policy.

## E2E-FW-003 — TCP allow single port

* [ ] Port mục tiêu mở.
* [ ] Port lân cận bị chặn.
* [ ] IPv4 đúng.
* [ ] IPv6 đúng nếu bật.

## E2E-FW-004 — TCP multiple ports

* [ ] Tất cả port trong set được allow.
* [ ] Port ngoài set bị chặn.
* [ ] Compiler dùng set như mong đợi.

## E2E-FW-005 — Port range

* [ ] Đầu range match.
* [ ] Cuối range match.
* [ ] Port ngoài range không match.

## E2E-FW-006 — UDP

* [ ] UDP echo được phép.
* [ ] UDP port khác bị chặn.
* [ ] Không áp dụng TCP state sai cho UDP.

## E2E-FW-007 — ICMP IPv4

* [ ] Echo request theo policy.
* [ ] Destination unreachable cần thiết không bị chặn sai.
* [ ] Rate limit hoạt động.

## E2E-FW-008 — ICMPv6

* [ ] Neighbor discovery không bị phá.
* [ ] Router advertisement xử lý theo policy.
* [ ] IPv6 connectivity không hỏng do chặn ICMPv6 cần thiết.

## E2E-FW-009 — Source CIDR

* [ ] IP trong CIDR được allow.
* [ ] IP ngoài CIDR bị chặn.
* [ ] Boundary address đúng.
* [ ] IPv6 prefix đúng.

## E2E-FW-010 — Destination CIDR

* [ ] Output traffic đúng destination được match.
* [ ] Destination khác không match.

## E2E-FW-011 — Interface selector

* [ ] Rule chỉ match interface WAN.
* [ ] Không match interface LAN.
* [ ] Interface không tồn tại bị validation error.

## E2E-FW-012 — Established/related

* [ ] Connection đang hoạt động tiếp tục.
* [ ] Return traffic được phép.
* [ ] Invalid state bị drop nếu policy yêu cầu.

## E2E-FW-013 — Reject versus drop

* [ ] Reject trả response phù hợp.
* [ ] Drop timeout như mong đợi.
* [ ] Counter phân biệt đúng.

## E2E-FW-014 — Rule order

* [ ] First-match semantics đúng.
* [ ] Shadow warning xuất hiện.
* [ ] Runtime order khớp compile preview.

## E2E-FW-015 — Logging rate limit

* [ ] Log được tạo cho match.
* [ ] Flood không làm journal tăng vô hạn.
* [ ] Prefix chứa rule ID.
* [ ] Packet vẫn xử lý đúng khi log bị rate-limit.

## E2E-FW-016 — Rule counters

* [ ] Counter bắt đầu ở giá trị hợp lệ.
* [ ] Traffic làm counter tăng.
* [ ] Counter map đúng rule ID.
* [ ] Missing handle không làm API crash.

## E2E-FW-017 — Large ruleset

Test tối thiểu:

* 100 rules.
* 1.000 rules.
* Giới hạn tối đa được cấu hình.

Expected:

* [ ] Compile thành công trong budget.
* [ ] Apply thành công trong budget.
* [ ] API không timeout ngoài thiết kế.
* [ ] Memory trở về gần baseline sau operation.
* [ ] Vượt limit bị từ chối trước compile.

---

# 12. Ruleset Isolation Tests

## E2E-ISO-001 — Existing custom nftables table

Tạo trước:

```nft
table inet third_party_test {
    chain input_test {
        counter
    }
}
```

Sau AegisNode apply:

* [ ] Table vẫn tồn tại.
* [ ] Rule vẫn tồn tại.
* [ ] Counter vẫn hoạt động.
* [ ] Snapshot/rollback không sửa table này.

## E2E-ISO-002 — Existing iptables-nft rules

* [ ] AegisNode phát hiện môi trường.
* [ ] Không flush rule.
* [ ] Existing service vẫn reachable.
* [ ] Rollback không xóa rule iptables-nft.

## E2E-ISO-003 — Existing firewalld

Trên RHEL-family:

* [ ] Phát hiện firewalld active.
* [ ] Không âm thầm tranh quyền quản lý.
* [ ] Chế độ unsupported trả lỗi rõ.
* [ ] Chế độ coexistence nếu được hỗ trợ không phá zone hiện tại.
* [ ] Restart firewalld không làm AegisNode crash.

## E2E-ISO-004 — Aegis table bị sửa bên ngoài

* [ ] Drift được phát hiện.
* [ ] Daemon không panic.
* [ ] Không tự overwrite nếu reconcile auto bị tắt.
* [ ] Audit drift.
* [ ] Operator có thể re-apply desired state.

## E2E-ISO-005 — Aegis table bị xóa bên ngoài

* [ ] Health chuyển degraded.
* [ ] API vẫn hoạt động.
* [ ] Reconcile khôi phục được nếu được yêu cầu.
* [ ] Không tạo duplicate snapshot.

---

# 13. Safe Apply and Rollback Tests

## E2E-RB-001 — Apply và confirm

* [ ] Validate.
* [ ] Snapshot.
* [ ] Apply.
* [ ] State thành pending confirmation.
* [ ] Confirm trước deadline.
* [ ] State thành committed.
* [ ] Rollback timer bị hủy.

## E2E-RB-002 — Không confirm

* [ ] Policy được apply tạm thời.
* [ ] Deadline hết hạn.
* [ ] Snapshot cũ được restore.
* [ ] State thành rolled back.
* [ ] Audit chứa lý do timeout.

## E2E-RB-003 — CLI process chết sau apply

* [ ] Daemon giữ execution.
* [ ] Timer tiếp tục.
* [ ] Không confirm thì rollback.
* [ ] Không phụ thuộc client process.

## E2E-RB-004 — Browser đóng sau apply

* [ ] Timer tiếp tục phía server.
* [ ] Mở lại browser thấy pending state.
* [ ] Deadline đúng.
* [ ] Rollback đúng nếu không confirm.

## E2E-RB-005 — Daemon SIGKILL trong pending state

* [ ] Systemd restart daemon.
* [ ] Pending execution được đọc từ SQLite.
* [ ] Deadline cũ được giữ.
* [ ] Hết hạn thì rollback.
* [ ] Không reset lại toàn bộ timeout từ đầu.

## E2E-RB-006 — Host reboot trong pending state

* [ ] Reboot VM.
* [ ] Service start.
* [ ] Pending state được phục hồi.
* [ ] Nếu deadline đã hết thì rollback ngay.
* [ ] Nếu chưa hết thì tiếp tục remaining duration.

## E2E-RB-007 — Snapshot bị thiếu

* [ ] Rollback không panic.
* [ ] Execution chuyển failed rõ ràng.
* [ ] Firewall hiện tại không bị flush.
* [ ] Có critical audit.
* [ ] Health chuyển degraded.

## E2E-RB-008 — Snapshot checksum sai

* [ ] Snapshot bị từ chối.
* [ ] Không apply dữ liệu corrupt.
* [ ] Không chạy `flush ruleset`.
* [ ] Error có snapshot ID.

## E2E-RB-009 — `nft --check` fail

* [ ] Runtime không thay đổi.
* [ ] Không tạo pending execution.
* [ ] Snapshot không cần thiết được dọn.
* [ ] Error trả về compiler/nft stderr đã sanitize.

## E2E-RB-010 — `nft apply` fail sau snapshot

* [ ] Runtime được inspect.
* [ ] Restore được thử nếu state đã thay đổi một phần.
* [ ] Không leave pending lock.
* [ ] Audit đầy đủ.

## E2E-RB-011 — Confirm sau deadline

* [ ] Bị từ chối.
* [ ] Policy cũ vẫn active.
* [ ] Không resurrect policy đã rollback.

## E2E-RB-012 — Hai confirm đồng thời

* [ ] Một operation thành công.
* [ ] Operation còn lại idempotent hoặc conflict.
* [ ] State cuối cùng nhất quán.

---

# 14. Crash and Fault Injection Tests

## E2E-FAULT-001 — Panic có chủ đích trong request worker

Chỉ trong test build có fault hook:

* [ ] Request lỗi trả failure.
* [ ] Toàn daemon không chết nếu architecture cho phép isolation.
* [ ] Nếu daemon chết, systemd restart.
* [ ] Firewall committed vẫn tồn tại.
* [ ] Có crash artifact.

## E2E-FAULT-002 — OOM kill

Giới hạn memory của service ở test environment:

* [ ] Process bị OOM kill có kiểm soát.
* [ ] Systemd restart theo policy.
* [ ] SQLite không corrupt.
* [ ] Firewall vẫn active.
* [ ] Không restart storm.

## E2E-FAULT-003 — Disk full trước apply

* [ ] Snapshot write thất bại.
* [ ] Policy không được apply.
* [ ] Error `insufficient storage`.
* [ ] Không panic.
* [ ] Apply lock được giải phóng.

## E2E-FAULT-004 — Disk full sau snapshot

* [ ] Candidate write fail.
* [ ] Runtime không đổi.
* [ ] Snapshot không corrupt hoặc được đánh dấu invalid.
* [ ] Audit có failure.

## E2E-FAULT-005 — Read-only state directory

* [ ] Service fail hoặc chuyển read-only/degraded.
* [ ] Không apply thay đổi không thể persist.
* [ ] Status endpoint vẫn giải thích state.
* [ ] Không restart loop.

## E2E-FAULT-006 — SQLite locked

* [ ] Operation retry có giới hạn.
* [ ] Timeout rõ ràng.
* [ ] Không block toàn bộ API vô hạn.
* [ ] Không deadlock.

## E2E-FAULT-007 — SQLite corrupt

* [ ] Service phát hiện integrity failure.
* [ ] Không tự xóa database.
* [ ] Firewall hiện tại giữ nguyên.
* [ ] Health degraded.
* [ ] Có hướng dẫn recovery.

## E2E-FAULT-008 — System clock nhảy về quá khứ

* [ ] Rollback deadline không bị kéo dài vô hạn.
* [ ] TTL block không trở thành permanent ngoài ý muốn.
* [ ] Audit timestamp vẫn xử lý được.
* [ ] Nên dùng monotonic time cho deadline runtime.

## E2E-FAULT-009 — System clock nhảy về tương lai

* [ ] Pending execution rollback an toàn.
* [ ] Không panic vì duration âm.
* [ ] Block expiry được reconcile.

## E2E-FAULT-010 — `nft` command treo

Dùng fake wrapper hoặc fault injection:

* [ ] Process runner timeout.
* [ ] Child process bị terminate.
* [ ] Apply lock được giải phóng.
* [ ] API không treo vô hạn.
* [ ] Service vẫn healthy/degraded có kiểm soát.

## E2E-FAULT-011 — `nft` command biến mất sau startup

* [ ] Operation fail rõ ràng.
* [ ] Daemon không crash.
* [ ] Existing firewall giữ nguyên.
* [ ] Health capability chuyển degraded.

## E2E-FAULT-012 — Journal unavailable

* [ ] Blocker chuyển degraded.
* [ ] Firewall management vẫn hoạt động.
* [ ] Không retry tight loop.
* [ ] Có backoff.

---

# 15. Reboot and Boot Recovery Tests

## E2E-BOOT-001 — Reboot với committed policy

* [ ] Reboot VM.
* [ ] Service auto-start.
* [ ] Desired policy được reconcile.
* [ ] Management port vẫn reachable.
* [ ] Policy hash khớp.
* [ ] Không duplicate table.

## E2E-BOOT-002 — Power-loss simulation

* [ ] Force-stop VM không graceful.
* [ ] Boot lại.
* [ ] SQLite integrity `ok`.
* [ ] Snapshot state được nhận diện.
* [ ] Không coi incomplete apply là committed.
* [ ] Firewall đi tới state an toàn đã định nghĩa.

## E2E-BOOT-003 — Network interface xuất hiện chậm

* [ ] Service không crash.
* [ ] Rule tham chiếu interface được xử lý theo semantics đã khóa.
* [ ] Khi interface xuất hiện, health cập nhật.
* [ ] Không cần restart daemon nếu có thể reconcile.

## E2E-BOOT-004 — Docker start sau AegisNode

* [ ] AegisNode start bình thường.
* [ ] Docker start sau đó.
* [ ] Container watcher kết nối lại.
* [ ] Docker rules không bị phá.
* [ ] Inventory cập nhật.

## E2E-BOOT-005 — Docker start trước AegisNode

* [ ] Inventory ban đầu đầy đủ.
* [ ] Published ports được phát hiện.
* [ ] Không apply duplicate rule.

## E2E-BOOT-006 — nftables service cũng enabled

* [ ] Boot order không tạo race phá ruleset.
* [ ] AegisNode phát hiện runtime state sau boot.
* [ ] Không tự flush.
* [ ] Desired state cuối cùng xác định.

---

# 16. Docker Compatibility Tests

Docker tạo firewall rules để phục vụ bridge networking và port publishing; các phiên bản hiện đại có thể sử dụng backend iptables hoặc nftables. Vì vậy Phase 11 phải test coexistence thay vì giả định Docker không chạm firewall.

## E2E-DKR-001 — Docker không được cài

* [ ] Daemon start.
* [ ] Docker module disabled/degraded.
* [ ] Không spam journal.
* [ ] Firewall API vẫn hoạt động.

## E2E-DKR-002 — Docker socket không accessible

* [ ] Permission error rõ.
* [ ] Không crash daemon.
* [ ] Retry có backoff.
* [ ] Status hiển thị Docker unavailable.

## E2E-DKR-003 — Container start/stop

* [ ] Inventory thêm container.
* [ ] Inventory xóa hoặc đánh dấu stopped.
* [ ] Không leak watcher task.
* [ ] Không tạo duplicate event.

## E2E-DKR-004 — Docker daemon restart

```bash
sudo systemctl restart docker
```

* [ ] AegisNode không crash.
* [ ] Watcher reconnect.
* [ ] Container inventory resync.
* [ ] Docker firewall rules còn nguyên.
* [ ] AegisNode tables còn nguyên.

## E2E-DKR-005 — Nginx public port

* [ ] Container port publish `0.0.0.0:8080`.
* [ ] Exposure analyzer phát hiện public.
* [ ] Warning đúng.
* [ ] Traffic hoạt động theo policy.

## E2E-DKR-006 — PostgreSQL public port warning

* [ ] Port 5432 public bị cảnh báo.
* [ ] Không tự block ngoài ý muốn.
* [ ] Warning có container ID/name.

## E2E-DKR-007 — Redis private bind

* [ ] Bind localhost/private IP không bị báo public sai.
* [ ] Inventory hiển thị đúng.

## E2E-DKR-008 — IPv6 published port

* [ ] `[::]:port` được nhận diện là public.
* [ ] IPv6 policy hoạt động.
* [ ] Không crash khi host chưa bật Docker IPv6.

## E2E-DKR-009 — Container churn

* [ ] Start/stop 100 container vòng lặp.
* [ ] Không memory leak đáng kể.
* [ ] Không task leak.
* [ ] Inventory cuối cùng chính xác.
* [ ] Event queue không tăng vô hạn.

## E2E-DKR-010 — Docker backend iptables-nft

* [ ] Container networking hoạt động.
* [ ] Port publishing hoạt động.
* [ ] AegisNode apply không xóa Docker chain.
* [ ] Rollback không xóa Docker chain.

## E2E-DKR-011 — Docker backend nftables

* [ ] Chỉ chạy khi Docker version hỗ trợ.
* [ ] Container networking hoạt động.
* [ ] AegisNode table namespace không conflict.
* [ ] Restart Docker không phá AegisNode.

## E2E-DKR-012 — Rootless Docker

* [ ] Phát hiện hoặc cấu hình socket riêng.
* [ ] Nếu chưa hỗ trợ, trả trạng thái `unsupported`.
* [ ] Không crash.
* [ ] Không cố thay đổi rootless networking ngoài phạm vi.

---

# 17. Router and NAT Tests

## E2E-NAT-001 — Router mode disabled mặc định

* [ ] IP forwarding không bị thay đổi.
* [ ] Không sinh NAT table không cần thiết.
* [ ] Forward policy không mở traffic.

## E2E-NAT-002 — Enable IPv4 forwarding

* [ ] Sysctl được bật.
* [ ] Giá trị cũ được snapshot.
* [ ] Commit giữ desired state.
* [ ] Rollback khôi phục giá trị cũ.

## E2E-NAT-003 — LAN to WAN masquerade

* [ ] LAN client truy cập WAN.
* [ ] Source được NAT.
* [ ] Return traffic hoạt động.
* [ ] Host không thuộc LAN rule không được NAT.

## E2E-NAT-004 — WAN không vào LAN mặc định

* [ ] Connection không được DNAT bị chặn.
* [ ] Không expose LAN ngoài ý muốn.

## E2E-NAT-005 — TCP port forwarding

* [ ] WAN external port tới đúng LAN server.
* [ ] Port khác không forward.
* [ ] Counter tăng.
* [ ] Source restriction hoạt động.

## E2E-NAT-006 — UDP port forwarding

* [ ] UDP packet tới đúng backend.
* [ ] Return traffic hoạt động.
* [ ] Timeout conntrack không làm daemon crash.

## E2E-NAT-007 — Duplicate external port

* [ ] Validation từ chối.
* [ ] Runtime không thay đổi.
* [ ] Error nêu rule conflict.

## E2E-NAT-008 — Destination không reachable

* [ ] Apply có thể thành công nếu semantics cho phép.
* [ ] Health warning rõ.
* [ ] Daemon không crash.
* [ ] Traffic failure không làm busy loop.

## E2E-NAT-009 — Interface down

* [ ] Daemon vẫn hoạt động.
* [ ] Health degraded.
* [ ] Khi interface up lại, NAT hoạt động.
* [ ] Không cần tạo duplicate rules.

## E2E-NAT-010 — IPv6 forwarding

Nếu MVP hỗ trợ:

* [ ] Forwarding hoạt động.
* [ ] ICMPv6 không bị phá.
* [ ] Không áp dụng IPv4 NAT semantics sai.

Nếu chưa hỗ trợ NAT66:

* [ ] Validation từ chối rõ ràng.
* [ ] Không crash.

---

# 18. Blocker and Journald Tests

## E2E-BLK-001 — Manual temporary IPv4 block

* [ ] IP bị block.
* [ ] Counter tăng.
* [ ] Entry hiển thị trong API.
* [ ] Entry tự hết hạn.

## E2E-BLK-002 — Manual temporary IPv6 block

* [ ] IPv6 bị block.
* [ ] Hết hạn đúng.
* [ ] Không ảnh hưởng IPv4 cùng host.

## E2E-BLK-003 — Permanent block

* [ ] Có warning.
* [ ] Entry tồn tại qua daemon restart.
* [ ] Có thể unblock.

## E2E-BLK-004 — Block loopback

* [ ] Bị validation từ chối.
* [ ] Không mất local API.

## E2E-BLK-005 — Block management CIDR

* [ ] Bị từ chối hoặc yêu cầu override đặc biệt.
* [ ] Không tự khóa operator trong default flow.

## E2E-BLK-006 — SSH threshold

* [ ] Tạo đủ failed login events.
* [ ] Trước threshold chưa block.
* [ ] Đạt threshold thì block.
* [ ] Audit chứa detector và count.

## E2E-BLK-007 — Window expiry

* [ ] Event cũ hết window.
* [ ] Không block sai.
* [ ] Detector state được cleanup.

## E2E-BLK-008 — Malformed journal entry

* [ ] Parser bỏ qua an toàn.
* [ ] Không panic.
* [ ] Có sampled warning, không spam.

## E2E-BLK-009 — Journal rotation

* [ ] Cursor cũ được xử lý.
* [ ] Không đọc lại toàn bộ lịch sử.
* [ ] Không bỏ lỡ vô hạn event mới.
* [ ] Daemon không crash.

## E2E-BLK-010 — Blocklist lớn

Test:

* 1.000 entries.
* 10.000 entries hoặc limit được khóa.

Expected:

* [ ] API pagination hoạt động.
* [ ] nftables set lookup hoạt động.
* [ ] Memory trong budget.
* [ ] Vượt limit bị từ chối rõ ràng.

## E2E-BLK-011 — Restart daemon với active blocks

* [ ] Runtime set được inspect/reconcile.
* [ ] Expiry không reset sai.
* [ ] Permanent block vẫn tồn tại.
* [ ] Expired block được dọn.

---

# 19. Persistence and Snapshot Tests

## E2E-DB-001 — Fresh migration

* [ ] Database mới được tạo.
* [ ] Tất cả migration chạy.
* [ ] Schema version đúng.
* [ ] Không chạy migration hai lần.

## E2E-DB-002 — Upgrade migration

* [ ] Tạo database từ version trước.
* [ ] Cài version mới.
* [ ] Migration thành công.
* [ ] Policy cũ đọc được.
* [ ] Audit cũ còn.
* [ ] Block entry cũ còn.

## E2E-DB-003 — Migration fail

* [ ] Transaction rollback.
* [ ] Database cũ không bị hỏng.
* [ ] Service không tiếp tục với schema nửa mới nửa cũ.
* [ ] Error rõ.

## E2E-DB-004 — Snapshot retention

* [ ] Tạo nhiều snapshot.
* [ ] Snapshot vượt retention được dọn.
* [ ] Snapshot đang được execution dùng không bị xóa.
* [ ] Last-known-good luôn được giữ.

## E2E-DB-005 — Atomic snapshot write

* [ ] Kill daemon giữa lúc ghi snapshot.
* [ ] File partial không được coi là valid.
* [ ] Checksum phát hiện incomplete file.
* [ ] Recovery không panic.

## E2E-DB-006 — Audit ordering

* [ ] Event có timestamp.
* [ ] Event có execution ID.
* [ ] Apply/confirm/rollback có trình tự hợp lý.
* [ ] Concurrent request không làm mất event.

---

# 20. Security and Abuse Tests

## E2E-SEC-001 — Non-root CLI access

* [ ] Authorized group dùng được CLI.
* [ ] Unauthorized user bị từ chối.
* [ ] Không cần setuid binary.

## E2E-SEC-002 — Symlink attack vào snapshot directory

* [ ] Không ghi đè file ngoài state directory.
* [ ] Dùng safe open/atomic write.
* [ ] Operation thất bại an toàn.

## E2E-SEC-003 — Path traversal trong policy ID

Payload:

```text
../../etc/passwd
policy/../../../root
```

* [ ] ID bị validation từ chối.
* [ ] Không tạo file ngoài directory.
* [ ] Không panic.

## E2E-SEC-004 — Command injection

Các field chứa:

```text
"; rm -rf /"
$(touch /tmp/pwned)
`id`
```

* [ ] Không thực thi shell.
* [ ] Không tạo `/tmp/pwned`.
* [ ] Input được xem là data.
* [ ] Validation xử lý đúng.

## E2E-SEC-005 — Malformed UTF-8

* [ ] API trả lỗi.
* [ ] Không panic.
* [ ] Journal không bị hỏng encoding.

## E2E-SEC-006 — Extremely deep YAML

* [ ] Parser có giới hạn.
* [ ] Không stack overflow.
* [ ] Không OOM.
* [ ] Operation timeout hợp lý.

## E2E-SEC-007 — YAML alias bomb

* [ ] Bị chặn hoặc giới hạn.
* [ ] Không tăng memory mất kiểm soát.
* [ ] Daemon vẫn responsive.

## E2E-SEC-008 — HTTP slow client

* [ ] Header/body timeout hoạt động.
* [ ] Connection bị đóng.
* [ ] Không exhaust worker/task.

## E2E-SEC-009 — High connection count

* [ ] Listener có giới hạn.
* [ ] Service không crash.
* [ ] Sau tải, resource trở về baseline.

## E2E-SEC-010 — Secret redaction

* [ ] Config secret không có trong journal.
* [ ] API error không trả secret.
* [ ] Audit không chứa raw token.
* [ ] Debug log vẫn redact.

---

# 21. Resource and Stability Tests

## Recommended MVP budget

Các con số dưới đây là release budget ban đầu; được phép điều chỉnh sau khi có benchmark baseline, nhưng mọi thay đổi phải được ghi lại.

* Startup đến readiness: tối đa 3 giây trên VM 2 vCPU.
* Idle RSS sau warm-up: tối đa 80 MB.
* Idle CPU: trung bình dưới 1%.
* File descriptor ổn định: không tăng liên tục.
* Không có memory growth vượt 15% sau warm-up trong soak test.
* Không có restart ngoài fault-injection test.
* API status p95 dưới 100 ms trong điều kiện local idle.
* Apply policy thông thường dưới 2 giây, không tính thời gian chờ confirm.

## E2E-RES-001 — Idle stability 1 giờ

* [ ] CPU ổn định.
* [ ] RSS ổn định.
* [ ] FD count ổn định.
* [ ] Task count ổn định.
* [ ] Journal không tăng vô lý.

## E2E-RES-002 — Soak 24 giờ

Traffic:

* HTTP requests liên tục.
* CLI status định kỳ.
* Docker events định kỳ.
* Block/unblock định kỳ.
* Policy validate định kỳ.

Expected:

* [ ] Không crash.
* [ ] Không restart.
* [ ] Không memory leak nghiêm trọng.
* [ ] Không FD leak.
* [ ] SQLite integrity `ok`.
* [ ] Final inventory chính xác.

## E2E-RES-003 — Concurrent reads

* [ ] 100 concurrent status/rules/audit requests.
* [ ] Không deadlock.
* [ ] Không starvation apply operation.
* [ ] Error rate trong ngưỡng.

## E2E-RES-004 — Concurrent validation

* [ ] Nhiều policy validate song song.
* [ ] Hash deterministic.
* [ ] Không shared-state corruption.

## E2E-RES-005 — Apply under read load

* [ ] Apply vẫn hoàn thành.
* [ ] Read request trả state nhất quán.
* [ ] Không thấy half-applied state.
* [ ] Không deadlock database.

## E2E-RES-006 — Log flood

* [ ] Gửi traffic bị drop với tốc độ cao.
* [ ] nft logging rate limit hoạt động.
* [ ] Journald không làm đầy disk nhanh.
* [ ] Daemon không phải xử lý từng packet log không giới hạn.

## E2E-RES-007 — Snapshot stress

* [ ] Apply/rollback lặp 100 lần.
* [ ] Không corrupt state.
* [ ] Retention hoạt động.
* [ ] Không để lock stale.
* [ ] Policy cuối đúng.

---

# 22. Upgrade and Downgrade Tests

## E2E-UPG-001 — Upgrade service đang active

* [ ] Package upgrade dừng/start đúng.
* [ ] Firewall committed vẫn tồn tại trong thời gian binary thay thế.
* [ ] Database migration chạy.
* [ ] Service trở lại healthy.
* [ ] Policy hash không đổi ngoài dự kiến.

## E2E-UPG-002 — Upgrade có active temporary block

* [ ] Block vẫn có hiệu lực.
* [ ] Expiry không reset sai.
* [ ] Inventory đồng bộ sau upgrade.

## E2E-UPG-003 — Upgrade có pending apply

Hành vi phải được khóa rõ:

* Hoặc package manager từ chối upgrade.
* Hoặc upgrade bảo toàn pending execution.
* Hoặc chủ động rollback trước upgrade.

Expected:

* [ ] Không để state mơ hồ.
* [ ] Không tự commit policy pending.

## E2E-UPG-004 — Upgrade config cũ

* [ ] Config cũ vẫn parse hoặc migration rõ.
* [ ] Unknown deprecated field có warning.
* [ ] Không âm thầm thay security default.

## E2E-UPG-005 — Downgrade schema không tương thích

* [ ] Downgrade bị từ chối rõ ràng.
* [ ] Database không bị sửa.
* [ ] Firewall vẫn hoạt động.
* [ ] Có hướng dẫn restore backup.

## E2E-UPG-006 — Binary replacement bị gián đoạn

* [ ] Package manager rollback hoặc để trạng thái dễ phục hồi.
* [ ] Firewall runtime không bị flush.
* [ ] Không để binary zero-byte.

---

# 23. Uninstall Tests

## E2E-UN-001 — Uninstall không purge

* [ ] Binary bị xóa.
* [ ] Unit bị disable.
* [ ] Config được giữ.
* [ ] Database được giữ.
* [ ] Snapshot được giữ.
* [ ] Hành vi đối với AegisNode nftables tables được document rõ.

## E2E-UN-002 — Purge

Chỉ khi user dùng flag/package purge:

* [ ] Config được xóa.
* [ ] State được xóa.
* [ ] AegisNode-owned tables được xóa.
* [ ] Third-party tables còn nguyên.
* [ ] Docker/firewalld rules còn nguyên.
* [ ] User/group được xử lý theo policy package.

## E2E-UN-003 — Uninstall khi service failed

* [ ] Package vẫn gỡ được.
* [ ] Không chạy cleanup nguy hiểm.
* [ ] Không flush ruleset.

## E2E-UN-004 — Reinstall sau uninstall

* [ ] Service cài lại được.
* [ ] State cũ được nhận diện nếu chưa purge.
* [ ] Migration chạy đúng.
* [ ] Không duplicate user/unit.

---

# 24. Compatibility-Specific Tests

## E2E-COMP-001 — systemd directive matrix

Trên từng distro:

* [ ] Unit load không warning.
* [ ] Unsupported hardening directive không làm service fail ngoài ý muốn.
* [ ] Runtime/State/ConfigurationDirectory hoạt động.
* [ ] Restart behavior nhất quán.
* [ ] Journal logging hoạt động.

## E2E-COMP-002 — nftables JSON output matrix

* [ ] Parse `nft -j list ruleset`.
* [ ] Missing optional field không crash.
* [ ] Unknown field được bỏ qua an toàn.
* [ ] Handle/counter/comment parse đúng.

## E2E-COMP-003 — Kernel module availability

* [ ] nftables module loaded hoặc loadable.
* [ ] Conntrack available khi cần.
* [ ] NAT module available khi router mode bật.
* [ ] Thiếu capability trả lỗi rõ.

## E2E-COMP-004 — IPv6 disabled kernel/sysctl

* [ ] Daemon không crash.
* [ ] IPv4 vẫn hoạt động.
* [ ] IPv6 policy bị validation/capability warning.
* [ ] Không cố apply ip6 table không hỗ trợ.

## E2E-COMP-005 — SELinux enforcing

Trên RHEL-family:

* [ ] Service start.
* [ ] SQLite write.
* [ ] Unix socket hoạt động.
* [ ] nft operation hoạt động theo policy.
* [ ] Không có AVC denial chưa giải quyết.

## E2E-COMP-006 — AppArmor enforcing

Trên Ubuntu:

* [ ] Service hoạt động với profile nếu cung cấp.
* [ ] Không bị chặn file/socket cần thiết.
* [ ] Không cấp quyền quá rộng chỉ để test pass.

## E2E-COMP-007 — amd64 versus arm64

* [ ] Cùng fixture tạo cùng policy hash.
* [ ] Compiler output tương đương.
* [ ] Serialization không phụ thuộc endian.
* [ ] Timer và integer không overflow.
* [ ] Package install pass.

---

# 25. Test Cadence

## Pull Request — Smoke Suite

Chạy trên Ubuntu 24.04 amd64:

* [ ] Clean install.
* [ ] Unit verify.
* [ ] Start/stop/restart.
* [ ] Default config.
* [ ] CLI status.
* [ ] API health.
* [ ] Basic input allow/drop.
* [ ] Apply/confirm.
* [ ] Timeout rollback.
* [ ] SIGKILL recovery.
* [ ] Existing nft table isolation.
* [ ] SQLite integrity.
* [ ] Uninstall không purge.
* [ ] Frontend load.
* [ ] No coredump.

## Nightly Suite

* [ ] Toàn bộ Tier 1 amd64.
* [ ] arm64 build và core E2E.
* [ ] Docker backend matrix.
* [ ] Router/NAT.
* [ ] Reboot.
* [ ] Fault injection.
* [ ] Concurrent API.
* [ ] Repeated apply/rollback.
* [ ] One-hour stability.

## Release Candidate Suite

* [ ] Toàn bộ Tier 1.
* [ ] Toàn bộ Tier 2.
* [ ] Upgrade từ release trước.
* [ ] Package reinstall.
* [ ] Uninstall/purge.
* [ ] SELinux.
* [ ] AppArmor.
* [ ] Docker iptables-nft.
* [ ] Docker nftables nếu hỗ trợ.
* [ ] 24-hour soak.
* [ ] Power-loss recovery.
* [ ] Disk-full recovery.
* [ ] SQLite corruption behavior.
* [ ] arm64 package.
* [ ] Final security abuse tests.

---

# 26. Artifact Collection

Mỗi failed test phải upload:

```text
artifacts/
├── metadata.json
├── test-result.json
├── systemctl-status.txt
├── systemctl-show.txt
├── journal.txt
├── coredump-list.txt
├── ruleset-before.json
├── ruleset-after.json
├── ruleset-diff.txt
├── sqlite-integrity.txt
├── process-metrics.csv
├── network-state.txt
├── docker-state.txt
└── screenshots/
```

## Bắt buộc thu thập

```bash
systemctl status aegisnode-local --no-pager
systemctl show aegisnode-local
journalctl -u aegisnode-local -b --no-pager
coredumpctl list aegisnode --no-pager
nft -j list ruleset
ip -details address
ip route show table all
ss -lntup
ps -eo pid,ppid,user,rss,vsz,stat,cmd
```

Không upload:

* Password.
* API token.
* Private key.
* User data thật.
* Production IP không được sanitize.

---

# 27. Failure Classification

## P0 — Release blocker nghiêm trọng

* Tự khóa management access mà rollback không cứu được.
* Flush hoặc xóa third-party rules.
* Mất Docker networking.
* SQLite corruption sau normal restart.
* Policy pending bị tự commit.
* Rollback restore sai snapshot.
* Privilege escalation.
* Command injection.
* Crash loop sau boot.
* Upgrade làm mất policy.

## P1 — Release blocker

* Daemon crash với input hợp lệ.
* Memory hoặc FD leak rõ ràng.
* Reboot không restore desired state.
* NAT hoạt động sai.
* Block management CIDR ngoài ý muốn.
* Tier 1 distro không chạy.
* systemd service không dừng đúng.
* Package không uninstall được.

## P2 — Phải sửa hoặc có accepted limitation

* Error message thiếu rõ ràng.
* Warning sai.
* Metrics thiếu.
* UI state không đồng bộ nhưng backend đúng.
* Unsupported environment chưa báo đẹp.

## P3 — Không chặn release

* Lỗi hiển thị nhỏ.
* Documentation typo.
* Cosmetic frontend issue không ảnh hưởng operation.

---

# 28. Phase 11 Release Gate

Phase 11 chỉ được đánh dấu hoàn thành khi:

* [ ] 100% smoke tests pass.
* [ ] 100% Tier 1 release tests pass.
* [ ] Không có P0.
* [ ] Không có P1 chưa xử lý.
* [ ] Tier 2 failure đã được phân loại và document.
* [ ] Không có coredump ngoài fault-injection test.
* [ ] Không có restart loop.
* [ ] 24-hour soak không crash.
* [ ] RSS không tăng liên tục.
* [ ] File descriptor không tăng liên tục.
* [ ] SQLite integrity pass sau mọi recovery test.
* [ ] Existing nftables rules không bị thay đổi.
* [ ] Docker networking không bị phá.
* [ ] Safe apply hoạt động.
* [ ] Rollback qua daemon restart hoạt động.
* [ ] Rollback qua host reboot hoạt động.
* [ ] Upgrade không làm mất policy.
* [ ] Uninstall không flush ruleset.
* [ ] Ubuntu 24.04 pass.
* [ ] Ubuntu 26.04 pass.
* [ ] Debian 13 pass.
* [ ] amd64 pass.
* [ ] arm64 core qualification pass.
* [ ] Package checksum được sinh.
* [ ] Test artifact được lưu.
* [ ] Compatibility report được xuất bản.

---

# 29. Final Qualification Report

Sau mỗi release candidate phải sinh file:

```text
docs/releases/<version>/PHASE_11_QUALIFICATION.md
```

Nội dung:

```text
Release version
Git commit
Build date
Test matrix
Passed cases
Failed cases
Skipped cases
Known limitations
Resource measurements
Soak-test result
Upgrade result
Compatibility result
Security result
Final release decision
```

Quyết định cuối:

```text
PASS
PASS WITH DOCUMENTED LIMITATIONS
FAIL
```

Không được đánh dấu `PASS` nếu còn lỗi P0 hoặc P1.
