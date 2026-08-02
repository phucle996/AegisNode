# AegisNode

> Lightweight Linux Firewall, Network and Service Management Platform written in Rust.

AegisNode là nền tảng quản lý node Linux tập trung vào 3 nhóm chức năng chính:
- **Firewall & NAT**: Dựa trên Linux `nftables` kernel subsystem.
- **Network Interface**: Quản lý IP, Route, VLAN, Bridge qua NetworkManager / systemd-networkd.
- **Systemd & Journald**: Quản lý dịch vụ hệ thống và xem nhật ký log.

## Kiến trúc cốt lõi (Core Principles)
1. **Không nằm trên data path**: Network packet được xử lý trực tiếp bởi Linux Kernel và nftables. AegisNode chỉ quản lý cấu hình và runtime state.
2. **Safe Apply & Rollback**: Mọi thay đổi nguy hiểm đều qua validation, snapshot, dry-run và rollback timer tự động.
3. **Local Autonomy**: Node tiếp tục hoạt động độc lập ngay cả khi mất kết nối tới Central Controller.
4. **Không chạy arbitrary shell**: Mọi tác vụ được định nghĩa bằng Schema an toàn nghiêm ngặt.

## Cấu trúc Workspace
- `apps/aegisnode`: Binary ứng dụng chính (hỗ trợ subcommands: `local`, `server`, `agent`, `execd`, `ctl`).
- `crates/`: Tập hợp các crate modul hóa domain logic (firewall, policy, storage, api, audit,...).
- `web/`: Giao diện quản trị Web UI (React + TypeScript).

## Hướng dẫn phát triển (Development)
```bash
# Kiểm tra build toàn bộ workspace
cargo build --workspace

# Chạy unit tests
cargo test --workspace

# Kiểm tra linting
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
