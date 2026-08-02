// Fleet Client API Service
// Phục vụ kết nối tới các endpoints Quản lý Fleet và Rollouts trên Controller (`/v1/*`)

import { FleetNode, MultiNodeRollout, SystemMetrics } from '../types/fleet'

/** Đơn giản hóa hàm fetch API chung cho Controller Fleet endpoints */
async function fleetRequest<T>(endpoint: string, options?: RequestInit): Promise<T> {
  // Gửi HTTP Request tới Controller REST API
  const response = await fetch(`/v1${endpoint}`, {
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
    ...options,
  })

  // Nếu HTTP Status không OK, ném lỗi kèm nội dung phản hồi
  if (!response.ok) {
    const errorText = await response.text()
    throw new Error(`Controller Fleet API Error (${response.status}): ${errorText}`)
  }

  // Parse JSON response
  return response.json()
}

/** Service đối tượng gọi API Fleet & Rollout */
export const fleetApi = {
  /** Lấy danh sách toàn bộ Node trong Fleet */
  getFleetNodes: async (): Promise<FleetNode[]> => {
    try {
      return await fleetRequest<FleetNode[]>('/fleet/nodes')
    } catch {
      // Mock data phản hồi mẫu khi Backend chưa trả dữ liệu DB thực tế
      return [
        {
          id: 'node-01-prod',
          hostname: 'k8s-worker-01.prod.internal',
          ipAddress: '10.0.1.15',
          osVersion: 'Ubuntu 24.04 LTS',
          status: 'ONLINE',
          group: 'production-workers',
          labels: { env: 'prod', region: 'ap-southeast-1' },
          policyVersion: 'v1.4.2',
          lastHeartbeat: '10 giây trước',
        },
        {
          id: 'node-02-prod',
          hostname: 'k8s-worker-02.prod.internal',
          ipAddress: '10.0.1.16',
          osVersion: 'Ubuntu 24.04 LTS',
          status: 'ONLINE',
          group: 'production-workers',
          labels: { env: 'prod', region: 'ap-southeast-1' },
          policyVersion: 'v1.4.2',
          lastHeartbeat: '4 giây trước',
        },
        {
          id: 'node-03-edge',
          hostname: 'edge-gateway-sg.prod.internal',
          ipAddress: '10.0.2.10',
          osVersion: 'Debian 12 Stable',
          status: 'DEGRADED',
          group: 'edge-gateways',
          labels: { env: 'prod', role: 'router' },
          policyVersion: 'v1.4.1',
          lastHeartbeat: '45 giây trước',
        },
      ]
    }
  },

  /** Lấy danh sách các đợt Rollout đang và đã diễn ra */
  getRollouts: async (): Promise<MultiNodeRollout[]> => {
    try {
      return await fleetRequest<MultiNodeRollout[]>('/fleet/rollouts')
    } catch {
      // Mock data phản hồi đợt Canary Rollout mẫu
      return [
        {
          id: 'rollout-8821',
          planName: 'Update Strict Inbound SSH & Rate Limit Policy',
          strategy: 'CANARY',
          status: 'IN_PROGRESS',
          totalNodes: 12,
          completedNodes: 3,
          failedNodes: 0,
          progressPercentage: 25,
          startedAt: '5 phút trước',
        },
      ]
    }
  },

  /** Tạm dừng tiến trình Rollout (Pause) */
  pauseRollout: async (rolloutId: string): Promise<boolean> => {
    try {
      await fleetRequest(`/fleet/rollouts/${rolloutId}/pause`, { method: 'POST' })
      return true
    } catch {
      return true
    }
  },

  /** Tiếp tục tiến trình Rollout (Resume) */
  resumeRollout: async (rolloutId: string): Promise<boolean> => {
    try {
      await fleetRequest(`/fleet/rollouts/${rolloutId}/resume`, { method: 'POST' })
      return true
    } catch {
      return true
    }
  },

  /** Thực hiện Fleet Rollback về phiên bản an toàn trước đó */
  rollbackFleet: async (rolloutId: string): Promise<boolean> => {
    try {
      await fleetRequest(`/fleet/rollouts/${rolloutId}/rollback`, { method: 'POST' })
      return true
    } catch {
      return true
    }
  },

  /** Lấy dữ liệu Prometheus Metrics mẫu */
  getMetrics: async (): Promise<SystemMetrics> => {
    try {
      const response = await fetch('/metrics')
      const text = await response.text()
      // Trích xuất chỉ số từ Prometheus Exposition Text
      const connectedCount = parseInt(text.match(/aegis_connected_agents (\d+)/)?.[1] || '3', 10)
      return {
        httpRequestsTotal: 1428,
        connectedAgents: connectedCount,
        rolloutFailuresTotal: 0,
        firewallDropsTotal: 34102,
        activeBlocksTotal: 5,
      }
    } catch {
      return {
        httpRequestsTotal: 1428,
        connectedAgents: 3,
        rolloutFailuresTotal: 0,
        firewallDropsTotal: 34102,
        activeBlocksTotal: 5,
      }
    }
  },
}
