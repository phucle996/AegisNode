// Fleet & Multi-Node Controller Type Definitions
// Định nghĩa kiểu dữ liệu TypeScript cho Quản lý Fleet, Rollout, Change Plan và Metrics

/** Trạng thái kết nối của Node trong Fleet */
export type NodeStatusType = 'ONLINE' | 'OFFLINE' | 'DEGRADED' | 'ENROLLING'

/** Thông tin chi tiết một Node trong hệ thống Fleet */
export interface FleetNode {
  /** Định danh duy nhất của Node (UUID) */
  id: string
  /** Tên Hostname của máy khách */
  hostname: string
  /** Địa chỉ IP quản lý mTLS */
  ipAddress: string
  /** Phiên bản OS của máy khách */
  osVersion: string
  /** Trạng thái hoạt động hiện tại */
  status: NodeStatusType
  /** Nhóm phân loại Node */
  group: string
  /** Các nhãn nhãn phân loại (Key-Value) */
  labels: Record<string, string>
  /** Phiên bản Firewall Policy đang chạy */
  policyVersion: string
  /** Thời điểm nhận Heartbeat lần cuối */
  lastHeartbeat: string
}

/** Chiến lược triển khai Rollout trên danh sách Node */
export type RolloutStrategyType = 'CANARY' | 'BATCH' | 'ALL_AT_ONCE' | 'MANUAL'

/** Trạng thái tiến trình của đợt Rollout */
export type RolloutStatusType = 'PENDING' | 'IN_PROGRESS' | 'RUNNING' | 'PAUSED' | 'SUCCESS' | 'COMPLETED' | 'FAILED' | 'CANCELLED' | 'ROLLED_BACK'

/** Chi tiết Đợt triển khai Multi-Node Rollout */
export interface MultiNodeRollout {
  /** ID duy nhất của đợt Rollout */
  id: string
  /** Tên mô tả của Change Plan */
  planName: string
  /** Chiến lược áp dụng (Canary / Batch) */
  strategy: RolloutStrategyType
  /** Trạng thái tiến trình hiện tại */
  status: RolloutStatusType
  /** Tổng số Node tham gia Rollout */
  totalNodes: number
  /** Số Node đã áp dụng thành công */
  completedNodes: number
  /** Số Node bị thất bại */
  failedNodes: number
  /** Tỷ lệ phần trăm tiến độ (0 - 100) */
  progressPercentage: number
  /** Thời gian bắt đầu triển khai */
  startedAt: string
}

/** Chỉ số Giám sát Prometheus Metrics */
export interface SystemMetrics {
  /** Tổng số HTTP requests đã xử lý */
  httpRequestsTotal: number
  /** Số lượng Agent đang kết nối mTLS */
  connectedAgents: number
  /** Tổng số Rollout thất bại */
  rolloutFailuresTotal: number
  /** Tổng số gói tin bị Firewall chặn */
  firewallDropsTotal: number
  /** Số lượng IP hiện đang bị khóa */
  activeBlocksTotal: number
}
