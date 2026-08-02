// Rollout Client API Service
// Kết nối trực tiếp tới Controller REST API (/v1/rollouts/*) quản lý tiến độ đợt triển khai Multi-Node

import { MultiNodeRollout } from '../types/fleet'

/** DTO Payload tạo Change Plan Rollout mới */
export interface CreateRolloutPayload {
  idempotencyKey?: string
  strategy: 'CANARY' | 'BATCH' | 'ROLLING'
  riskLevel: 'LOW' | 'MEDIUM' | 'HIGH'
  batchSize: number
  maxUnavailable: number
  failureThresholdPercent: number
  targetNodeId?: string
}

/** DTO Báo cáo Tiến độ Rollout từ Controller */
export interface RolloutStatusResponse {
  rolloutId: string
  status: string
  progressPercent: number
  totalNodes: number
  succeededNodes: number
  failedNodes: number
  pendingNodes: number
  nodeStatuses: Array<{ nodeId: string; state: string }>
}

/** Hàm gửi HTTP Request tới Controller REST API */
async function rolloutRequest<T>(endpoint: string, options?: RequestInit): Promise<T> {
  const response = await fetch(`/v1/rollouts${endpoint}`, {
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
    ...options,
  })

  if (!response.ok) {
    const errorText = await response.text()
    throw new Error(`Controller Rollout API Error (${response.status}): ${errorText}`)
  }

  return response.json()
}

/** Service API truy vấn và điều phối Rollout Plans */
export const rolloutApi = {
  /** Truy vấn danh sách toàn bộ các đợt Rollouts (`GET /v1/rollouts`) */
  getRollouts: async (): Promise<MultiNodeRollout[]> => {
    try {
      const data = await rolloutRequest<any[]>('')
      if (!Array.isArray(data)) return []
      return data.map((item) => ({
        id: item.id,
        planName: item.idempotencyKey || `Rollout Plan ${item.id.substring(0, 8)}`,
        strategy: (item.strategy as any) || 'CANARY',
        status: (item.status as any) || 'IN_PROGRESS',
        totalNodes: item.totalNodes || 3,
        completedNodes: Math.round(((item.progressPercent || 0) * (item.totalNodes || 3)) / 100),
        failedNodes: 0,
        progressPercentage: item.progressPercent || 0,
        startedAt: item.createdAt || new Date().toISOString(),
      }))
    } catch (error) {
      console.error('Lỗi lấy danh sách Rollouts:', error)
      return []
    }
  },

  /** Truy vấn thông tin tiến độ của 1 đợt Rollout theo ID (`GET /v1/rollouts/{id}`) */
  getRolloutStatus: async (id: string): Promise<RolloutStatusResponse | null> => {
    try {
      return await rolloutRequest<RolloutStatusResponse>(`/${id}`)
    } catch (error) {
      console.error(`Lỗi lấy tiến độ Rollout ${id}:`, error)
      return null
    }
  },

  /** Tạo đợt triển khai Rollout mới (`POST /v1/rollouts`) */
  createRollout: async (payload: CreateRolloutPayload): Promise<any> => {
    const body = {
      id: crypto.randomUUID(),
      idempotencyKey: payload.idempotencyKey || `plan-${Date.now()}`,
      riskLevel: payload.riskLevel,
      targetNodeId: payload.targetNodeId || 'c9c9379a-79d9-4a27-b9fa-46dee7c728b2',
      steps: [
        {
          stepId: `step-${Date.now()}`,
          order: 1,
          name: 'step_01_snapshot',
          action: 'CREATE_SNAPSHOT',
          component: 'firewall',
          status: 'PENDING',
        },
      ],
      healthCheck: {
        probeGateway: true,
        probeDns: true,
        timeoutSeconds: 30,
      },
    }
    return await rolloutRequest('', {
      method: 'POST',
      body: JSON.stringify(body),
    })
  },

  /** Tạm dừng đợt Rollout (`PATCH /v1/rollouts/{id}/pause`) */
  pauseRollout: async (id: string): Promise<void> => {
    await rolloutRequest(`/${id}/pause`, { method: 'PATCH' })
  },

  /** Tiếp tục đợt Rollout (`PATCH /v1/rollouts/{id}/resume`) */
  resumeRollout: async (id: string): Promise<void> => {
    await rolloutRequest(`/${id}/resume`, { method: 'PATCH' })
  },

  /** Hủy đợt Rollout (`PATCH /v1/rollouts/{id}/cancel`) */
  cancelRollout: async (id: string): Promise<void> => {
    await rolloutRequest(`/${id}/cancel`, { method: 'PATCH' })
  },

  /** Thực hiện Fleet Rollback đảo ngược (`PATCH /v1/rollouts/{id}/rollback`) */
  rollbackFleet: async (id: string): Promise<void> => {
    await rolloutRequest(`/${id}/rollback`, { method: 'PATCH' })
  },
}
