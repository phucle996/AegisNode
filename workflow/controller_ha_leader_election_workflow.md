# End-to-End Controller HA Leader Election & Cluster State Sync Workflow Document

Tài liệu này mô tả chi tiết luồng **Bầu chọn Leader phân tán (Distributed Leader Election)** và **Đồng bộ Trạng thái Cluster trong Môi trường High Availability (HA Multi-Replica Deployment)** của `AegisNode Controller`.

---

## 1. Tổng quan Kiến trúc Controller High Availability (HA Architecture)

Khi triển khai `AegisNode Controller` trên môi trường Kubernetes hoặc Multi-Node Cluster, nhiều Controller Replicas (Pods/Instances) được khởi tạo đồng thời để đạt tính sẵn sàng cao (High Availability). 

Để tránh hiện tượng **Split-Brain** và **Duplicate Job Execution** (nhiều Controller cùng thực thi đè công việc), hệ thống sử dụng **PostgreSQL Transactional Advisory Locks (`pg_try_advisory_lock`)**:

```
+---------------------------------------------------------------------------------------+
|                                  CONTROLLER HA REPLICAS                               |
|                                                                                       |
|  +----------------------------+                 +------------------------------+  |
|  |    Controller Replica 1    |                 |     Controller Replica 2     |  |
|  |  Role: ACTIVE LEADER       |                 |     Role: STANDBY FOLLOWER   |  |
|  |  is_leader = true          |                 |     is_leader = false        |  |
|  +--------------+-------------+                 +--------------+---------------+  |
+-----------------|----------------------------------------------|------------------+
                  |                                              |
                  | Try Acquire Lock (Key ID: 88888888)          | Try Acquire Lock (Denied)
                  v                                              v
                      +--------------------------------------+
                      |      PostgreSQL HA Storage           |
                      |                                      |
                      |  - SELECT pg_try_advisory_lock(88888888)
                      |  - Single Active Advisory Lock       |
                      +--------------------------------------+
```

---

## 2. Luồng Trình tự Bầu chọn Leader (Mermaid Sequence Diagram)

```mermaid
sequenceDiagram
    autonumber
    participant R1 as ⚙️ Controller Replica 1
    participant R2 as ⚙️ Controller Replica 2
    participant DB as 🐘 PostgreSQL (Advisory Lock Key 88888888)
    participant K8s as ☸️ K8s / LB Readiness Probe

    note over R1, R2: Bước 1: Khởi động Replicas & Vòng lặp Elector (Mỗi 5s)
    par Replica 1 Try Lock
        R1->>DB: SELECT pg_try_advisory_lock(88888888)
        DB-->>R1: Trả về true (Acquired Successfully!)
        R1->>R1: Set is_leader = true -> Role: ACTIVE LEADER
    and Replica 2 Try Lock
        R2->>DB: SELECT pg_try_advisory_lock(88888888)
        DB-->>R2: Trả về false (Lock Already Held by Replica 1)
        R2->>R2: Set is_leader = false -> Role: STANDBY FOLLOWER
    end

    note over K8s, R2: Bước 2: Load Balancer Health / Readiness Probes
    K8s->>R1: GET /readiness
    R1-->>K8s: 200 OK { status: "READY", role: "LEADER", leaderElection: "advisory_lock" }

    K8s->>R2: GET /readiness
    R2-->>K8s: 200 OK { status: "STANDBY", role: "FOLLOWER", leaderElection: "advisory_lock" }

    note over R1, R2: Bước 3: Failover Tự động khi Leader Gặp Sự cố (Crash / Restart)
    R1->>R1: Replica 1 (Leader) bị Crash / Network Cut
    DB->>DB: PostgreSQL tự động giải phóng Session Advisory Lock!

    note over R2, DB: Bước 4: Replica 2 Chiếm Lock & Trở thành LEADER mới
    R2->>DB: SELECT pg_try_advisory_lock(88888888) (Vòng lặp 5s tiếp theo)
    DB-->>R2: Trả về true (Lock Acquired!)
    R2->>R2: Set is_leader = true -> Promoted to ACTIVE LEADER 🎉
    K8s->>R2: GET /readiness
    R2-->>K8s: 200 OK { status: "READY", role: "LEADER" }
```

---

## 3. Ma trận Quyền hạn & Phân chia Nhiệm vụ (Leader vs Follower Matrix)

| Chức năng / Công việc trong System | ACTIVE LEADER (`is_leader = true`) | STANDBY FOLLOWER (`is_leader = false`) |
| :--- | :--- | :--- |
| **Phục vụ REST API Read Only** (`GET /v1/nodes`) | ✅ Sẵn sàng phục vụ | ✅ Sẵn sàng phục vụ (Load Balanced) |
| **Nhận Node Heartbeats & Inventory** | ✅ Xử lý và ghi vào DB | ✅ Xử lý và ghi vào DB |
| **Phát hành Certificate & Ký CSR** | ✅ Thực thi ký X.509 | ❌ Từ chối / Chuyển hướng lên Leader |
| **Điều phối Job Orchestrator & Rollouts** | ✅ Lên lịch & Phân phối | ❌ Chờ ở trạng thái Standby |
| **Tự động Clean Up Offline Nodes** | ✅ Thực thi background worker | ❌ Bị vô hiệu hóa background worker |

---

## 4. Đặc tính Kỹ thuật của PostgreSQL Advisory Locks

> [!IMPORTANT]
> **Session-Level Lock (`pg_try_advisory_lock`):**
> PostgreSQL Advisory Locks liên kết trực tiếp với **Database Connection Session** của Controller. Khi một Leader Controller Replica bị crash hoặc sập mạng, kết nối TCP tới PostgreSQL ngắt -> PostgreSQL ngay lập tức giải phóng Advisory Lock `88888888` mà không cần chờ timeout phức tạp.

> [!TIP]
> **Khả năng Phục hồi Tự động (Zero Manual Intervention):**
> Vòng lặp `LeaderElector` chạy ngầm mỗi 5 giây (`tokio::time::sleep(5s)`). Ngay khi Leader cũ ngắt kết nối, Follower Replica tiếp theo sẽ chiếm được lock và nâng cấp vai trò thành Leader trong vòng dưới 5 giây.
