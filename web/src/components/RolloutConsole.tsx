// Rollout Console Component (Phase 19 Central Web UI)
// Hiển thị tiến trình Canary / Batch Rollouts trên toàn Fleet và các nút điều khiển Pause, Resume, Rollback

import React, { useEffect, useState } from 'react'
import { fleetApi } from '../api/fleetClient'
import { MultiNodeRollout } from '../types/fleet'

export const RolloutConsole: React.FC = () => {
  // State quản lý danh sách các đợt Rollout
  const [rollouts, setRollouts] = useState<MultiNodeRollout[]>([])
  // State theo dõi trạng thái đang bấm thao tác
  const [actionLoading, setActionLoading] = useState<string | null>(null)

  // Fetch danh sách Rollouts từ API Controller
  const loadRollouts = async () => {
    const data = await fleetApi.getRollouts()
    setRollouts(data)
  }

  useEffect(() => {
    loadRollouts()
    // Tự động poll tiến độ Rollout mỗi 3 giây
    const timer = setInterval(loadRollouts, 3000)
    return () => clearInterval(timer)
  }, [])

  // Xử lý tạm dừng Rollout (Pause)
  const handlePause = async (id: string) => {
    setActionLoading(`pause-${id}`)
    await fleetApi.pauseRollout(id)
    await loadRollouts()
    setActionLoading(null)
  }

  // Xử lý tiếp tục Rollout (Resume)
  const handleResume = async (id: string) => {
    setActionLoading(`resume-${id}`)
    await fleetApi.resumeRollout(id)
    await loadRollouts()
    setActionLoading(null)
  }

  // Xử lý hủy đợt Rollout và thực hiện Fleet Rollback
  const handleRollback = async (id: string) => {
    if (!window.confirm('Bạn có chắc chắn muốn hủy Rollout và khôi phục toàn bộ Fleet về version trước đó?')) {
      return
    }
    setActionLoading(`rollback-${id}`)
    await fleetApi.rollbackFleet(id)
    await loadRollouts()
    setActionLoading(null)
  }

  return (
    <div className="space-y-6">
      {/* Header điều khiển Rollout Console */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-bold text-slate-100">Multi-Node Rollout Console</h2>
          <p className="text-sm text-slate-400">
            Giám sát và điều hướng tiến trình áp dụng Policy (Canary / Batch) trên toàn mạng lưới
          </p>
        </div>
        <button
          onClick={() => alert('Vui lòng chọn Policy từ Policy Editor để tạo Change Plan mới')}
          className="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white font-medium text-xs rounded-lg transition-colors shadow-lg shadow-indigo-600/20"
        >
          + Create New Fleet Change Plan
        </button>
      </div>

      {/* Danh sách các đợt Rollout */}
      <div className="space-y-4">
        {rollouts.map((rollout) => (
          <div key={rollout.id} className="bg-slate-900/60 border border-slate-800 rounded-xl p-6 space-y-4">
            <div className="flex items-start justify-between">
              <div>
                <div className="flex items-center gap-2">
                  <span className="px-2.5 py-0.5 bg-indigo-500/10 text-indigo-400 border border-indigo-500/20 text-[11px] font-mono rounded">
                    {rollout.strategy} STRATEGY
                  </span>
                  <span className="text-xs text-slate-500 font-mono">ID: {rollout.id}</span>
                </div>
                <h3 className="text-lg font-semibold text-slate-100 mt-2">{rollout.planName}</h3>
              </div>

              {/* Trạng thái hiện tại của Rollout */}
              <span
                className={`px-3 py-1 text-xs font-semibold rounded-full border ${
                  rollout.status === 'IN_PROGRESS'
                    ? 'bg-sky-500/10 text-sky-400 border-sky-500/20 animate-pulse'
                    : rollout.status === 'PAUSED'
                    ? 'bg-amber-500/10 text-amber-400 border-amber-500/20'
                    : 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
                }`}
              >
                ● {rollout.status}
              </span>
            </div>

            {/* Thanh Progress bar hiển thị tiến độ % */}
            <div className="space-y-2">
              <div className="flex justify-between text-xs font-mono">
                <span className="text-slate-400">Rollout Progress ({rollout.progressPercentage}%)</span>
                <span className="text-slate-300">
                  {rollout.completedNodes} / {rollout.totalNodes} Nodes Completed ({rollout.failedNodes} Failed)
                </span>
              </div>
              <div className="w-full bg-slate-800 h-3 rounded-full overflow-hidden">
                <div
                  className="bg-indigo-500 h-full transition-all duration-500"
                  style={{ width: `${rollout.progressPercentage}%` }}
                />
              </div>
            </div>

            {/* Bộ các nút hành động điều khiển (Pause / Resume / Fleet Rollback) */}
            <div className="flex items-center justify-end gap-3 pt-2 border-t border-slate-800/60">
              {rollout.status === 'IN_PROGRESS' ? (
                <button
                  disabled={actionLoading === `pause-${rollout.id}`}
                  onClick={() => handlePause(rollout.id)}
                  className="px-3 py-1.5 bg-amber-600/20 hover:bg-amber-600/30 text-amber-300 border border-amber-500/30 font-medium text-xs rounded transition-colors"
                >
                  ⏸ Pause Rollout
                </button>
              ) : (
                <button
                  disabled={actionLoading === `resume-${rollout.id}`}
                  onClick={() => handleResume(rollout.id)}
                  className="px-3 py-1.5 bg-emerald-600/20 hover:bg-emerald-600/30 text-emerald-300 border border-emerald-500/30 font-medium text-xs rounded transition-colors"
                >
                  ▶ Resume Rollout
                </button>
              )}

              <button
                disabled={actionLoading === `rollback-${rollout.id}`}
                onClick={() => handleRollback(rollout.id)}
                className="px-3 py-1.5 bg-rose-600/20 hover:bg-rose-600/30 text-rose-300 border border-rose-500/30 font-medium text-xs rounded transition-colors"
              >
                ↺ Fleet Rollback
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
