# End-to-End Safe Apply & Transactional Rollback Workflow Document

Tài liệu này mô tả chi tiết luồng **Safe Apply & Transactional Rollback (Tự động Hoàn tác An toàn)** trong hệ thống `AegisNode`. Cơ chế này giải quyết triệt để bài toán rủi ro lớn nhất khi quản trị firewall từ xa: **Áp dụng ruleset sai làm đứt kết nối mạng (Lock-out) và không thể truy cập lại server**.

---

## 1. Tổng quan Kiến trúc Safe Apply (Architecture Overview)

Luồng Safe Apply hoạt động theo mô hình **Atomic 2-Phase Commit (2PC) với Rollback Timer ngầm**:

```
+---------------------------------------------------------------------------------------+
|                                    SAFE APPLY MANAGER                                 |
|                                                                                       |
|  1. Capture Snapshot current nftables ruleset -> /var/lib/aegisnode/snapshots/         |
|  2. Compile & Validate new Firewall Policy                                            |
|  3. Atomic Apply candidate ruleset (`nft -f candidate.nft`)                          |
|  4. State -> `AppliedPendingConfirmation`                                             |
|  5. Spawn Automatic Rollback Timer task in background (Timeout: N seconds)            |
+---------------------------------------------------------------------------------------+
                                    |
          +-------------------------+-------------------------+
          |                                                   |
 [Admin xác nhận thành công]                         [Timeout hết hạn mà không Confirm]
          |                                                   |
          v                                                   v
+-----------------------------------+               +-----------------------------------+
|      confirm(execution_id)        |               |    automatic_rollback_timer()     |
|                                   |               |                                   |
| - Cancel Rollback Timer task      |               | - Restore ruleset from Snapshot   |
| - State -> `Committed`            |               | - State -> `RolledBack`           |
| - Commit ruleset vĩnh viễn        |               | - Khôi phục 100% mạng ban đầu     |
+-----------------------------------+               +-----------------------------------+
```

---

## 2. Luồng Trình tự End-to-End (Mermaid Sequence Diagram)

```mermaid
sequenceDiagram
    autonumber
    actor Admin as 👨‍💻 SRE Admin / Controller
    participant SAM as 🛡️ SafeApplyManager
    participant Snap as 📸 SnapshotManager
    participant Backend as ⚙️ NftablesRuntimeBackend
    participant System as 🖥️ Linux Host (nftables)
    participant Timer as ⏳ Background Rollback Timer

    note over Admin, System: Bước 1: Khởi tạo Safe Apply & Chụp Snapshot
    Admin->>SAM: execute_safe_apply(policy, timeout_seconds=30)
    SAM->>SAM: Acquire Concurrent Apply Mutex Lock (Ngăn chặn xung đột)
    SAM->>Snap: create_snapshot()
    Snap->>System: nft --json list ruleset
    System-->>Snap: Trả về JSON Ruleset hiện tại
    Snap->>Snap: Lưu snapshotfile: snap_<uuid>.json

    note over SAM, System: Bước 2: Nạp Candidate Ruleset & Kích hoạt Timer
    SAM->>Backend: apply_candidate_policy(policy)
    Backend->>System: nft -f /run/aegisnode/candidate.nft
    System-->>Backend: OK (Ruleset mới có hiệu lực tạm thời)
    SAM->>SAM: Transition state -> AppliedPendingConfirmation
    SAM->>Timer: tokio::spawn(async move { sleep(timeout); trigger_rollback_if_unconfirmed() })
    SAM-->>Admin: Trả về ExecutionResult { execution_id, state: AppliedPendingConfirmation }

    note over Admin, Timer: Kịch bản A: Confirm thành công (Happy Path)
    opt Admin xác nhận mạng vẫn thông suốt
        Admin->>SAM: confirm(execution_id)
        SAM->>Timer: Cancel/Abort Rollback Timer Task
        SAM->>SAM: Transition state -> Committed
        SAM-->>Admin: Trả về ExecutionResult { state: Committed }
    end

    note over Admin, Timer: Kịch bản B: Tự động Rollback do Timeout / Đứt mạng (Failure Path)
    opt Admin không thể Confirm (do đứt kết nối / quá timeout 30s)
        Timer->>Timer: Wake up sau N seconds sleep
        Timer->>Snap: restore_snapshot(snap_id)
        Snap->>System: nft -f snap_<uuid>.json
        System-->>Snap: OK (Khôi phục toàn bộ ruleset cũ)
        Timer->>SAM: Transition state -> RolledBack
        note over System: Kết nối SSH / REST API được khôi phục hoàn toàn
    end
```

---

## 3. Data Models & State Machine (Trạng thái giao dịch)

### 3.1. Các Trạng thái Giao dịch (`ExecutionState`)

| Trạng thái | Ý nghĩa | Hành động tiếp theo có thể có |
| :--- | :--- | :--- |
| `Pending` | Khởi tạo giao dịch, đang chụp snapshot | Apply candidate ruleset |
| `AppliedPendingConfirmation` | Ruleset mới đã nạp vào nftables, chờ Confirm | `Confirm` hoặc `Rollback` (Manual/Auto) |
| `Committed` | Admin đã confirm thành công, hủy timer hoàn tác | Giao dịch hoàn tất (Terminal state) |
| `RolledBack` | Hệ thống đã tự động hoàn tác về snapshot | Giao dịch đã rollback (Terminal state) |

### 3.2. Cấu trúc Bản ghi Snapshot (`SnapshotRecord`)
```json
{
  "snapshotId": "snap_f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "createdAt": "2026-08-02T13:30:00Z",
  "nftablesRulesetJson": "{\"nftables\":[{\"table\":{\"family\":\"inet\",\"name\":\"aegis_filter\"}}]}",
  "activeTables": ["inet:aegis_filter", "ip:aegis_nat"]
}
```

---

## 4. An toàn Bảo mật & Chống Race Condition

> [!IMPORTANT]
> **Concurrent Apply Mutex Lock:**
> Để tránh tình trạng 2 tiến trình Admin/Controller gửi 2 lệnh Safe Apply cùng lúc làm đè Snapshot của nhau, `SafeApplyManager` sử dụng `Arc<tokio::sync::Mutex<()>>`. Chỉ đúng 1 giao dịch Safe Apply được phép thực thi tại một thời điểm.

> [!TIP]
> **Tự động Dọn dẹp Snapshots cũ (Snapshot Rotation):**
> `SnapshotManager` duy trì tối đa $N$ bản snapshot gần nhất (mặc định $N=5$). Các snapshot cũ hơn sẽ tự động được xoá khỏi đĩa cứng để tránh cạn kiệt dung lượng `/var/lib/aegisnode`.
