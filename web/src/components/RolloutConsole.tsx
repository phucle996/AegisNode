// Multi-Node Rollout Console Component
// Giám sát và điều hướng tiến trình áp dụng Policy (Canary / Batch / Rolling) trên toàn Fleet kết nối API Controller thực tế

import React, { useEffect, useState } from 'react'
import { rolloutApi } from '../api/rolloutClient'
import { MultiNodeRollout } from '../types/fleet'
import { CreateRolloutDialog } from './rollout/CreateRolloutDialog'
import { ConfirmRollbackDialog } from './rollout/ConfirmRollbackDialog'
import { PlusIcon, PauseIcon, PlayIcon, RotateCcwIcon, XCircleIcon, LayersIcon } from 'lucide-react'

export const RolloutConsole: React.FC = () => {
  // State quản lý danh sách các đợt Rollout thực tế từ PostgreSQL
  const [rollouts, setRollouts] = useState<MultiNodeRollout[]>([])
  // State điều khiển mở Dialog Tạo Rollout mới
  const [createDialogOpen, setCreateDialogOpen] = useState<boolean>(false)
  // State điều khiển mở Dialog Xác nhận Thao tác khẩn cấp (Pause, Resume, Rollback, Cancel)
  const [confirmDialogOpen, setConfirmDialogOpen] = useState<boolean>(false)
  // Target Rollout ID đang chọn để thao tác
  const [targetRolloutId, setTargetRolloutId] = useState<string>('')
  // Loại thao tác đang chọn
  const [actionType, setActionType] = useState<'PAUSE' | 'RESUME' | 'CANCEL' | 'ROLLBACK'>('PAUSE')
  // Trạng thái đang gửi HTTP Request
  const [actionLoading, setActionLoading] = useState<boolean>(false)

  // Fetch danh sách Rollouts từ Controller REST API (GET /v1/rollouts)
  const loadRollouts = async () => {
    const data = await rolloutApi.getRollouts()
    setRollouts(data)
  }

  useEffect(() => {
    loadRollouts()
    // Tự động poll tiến độ Rollout mỗi 3 giây
    const timer = setInterval(loadRollouts, 3000)
    return () => clearInterval(timer)
  }, [])

  // Mở Dialog xác nhận cho thao tác người dùng
  const triggerActionDialog = (id: string, type: 'PAUSE' | 'RESUME' | 'CANCEL' | 'ROLLBACK') => {
    setTargetRolloutId(id)
    setActionType(type)
    setConfirmDialogOpen(true)
  }

  // Thực thi thao tác đã xác nhận từ React Component Dialog
  const handleExecuteAction = async () => {
    if (!targetRolloutId) return
    setActionLoading(true)
    try {
      switch (actionType) {
        case 'PAUSE':
          await rolloutApi.pauseRollout(targetRolloutId)
          break
        case 'RESUME':
          await rolloutApi.resumeRollout(targetRolloutId)
          break
        case 'CANCEL':
          await rolloutApi.cancelRollout(targetRolloutId)
          break
        case 'ROLLBACK':
          await rolloutApi.rollbackFleet(targetRolloutId)
          break
      }
      await loadRollouts()
    } catch (error) {
      console.error(`Lỗi thực thi ${actionType}:`, error)
    } finally {
      setActionLoading(false)
    }
  }

  // Tính toán số liệu tổng quan Summary Metrics
  const activeCount = rollouts.filter((r) => r.status === 'IN_PROGRESS' || r.status === 'RUNNING').length
  const completedCount = rollouts.filter((r) => r.status === 'COMPLETED' || r.status === 'SUCCESS').length
  const totalNodesCount = rollouts.reduce((acc, r) => acc + (r.totalNodes || 0), 0)

  return (
    <div className="space-y-6">
      {/* Header điều khiển Rollout Console & Nút Tạo mới */}
      <div className="flex items-center justify-between bg-slate-900/60 backdrop-blur border border-slate-800 p-6 rounded-2xl">
        <div>
          <div className="flex items-center gap-2">
            <span className="p-2 bg-indigo-500/10 border border-indigo-500/20 text-indigo-400 rounded-lg">
              <LayersIcon className="w-5 h-5" />
            </span>
            <h2 className="text-xl font-bold text-slate-100">Multi-Node Rollout Console</h2>
          </div>
          <p className="text-xs text-slate-400 mt-1">
            Điều phối và giám sát tiến trình áp dụng Change Plan (Canary / Batch) trên toàn bộ Linux Nodes
          </p>
        </div>
        <button
          onClick={() => setCreateDialogOpen(true)}
          className="px-4 py-2.5 bg-indigo-600 hover:bg-indigo-500 text-white font-medium text-xs rounded-xl transition-all shadow-lg shadow-indigo-600/25 flex items-center gap-2"
        >
          <PlusIcon className="w-4 h-4" />
          Phát hành Change Plan Mới
        </button>
      </div>

      {/* Thẻ Chỉ số Tổng quan Summary Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="bg-slate-900/60 border border-slate-800 p-4 rounded-xl">
          <p className="text-xs font-mono text-slate-400">TỔNG ĐỢT ROLLOUT</p>
          <p className="text-2xl font-bold text-slate-100 font-mono mt-1">{rollouts.length}</p>
        </div>
        <div className="bg-slate-900/60 border border-slate-800 p-4 rounded-xl">
          <p className="text-xs font-mono text-slate-400">ĐANG CHẠY (ACTIVE)</p>
          <p className="text-2xl font-bold text-sky-400 font-mono mt-1">{activeCount}</p>
        </div>
        <div className="bg-slate-900/60 border border-slate-800 p-4 rounded-xl">
          <p className="text-xs font-mono text-slate-400">HOÀN THÀNH (COMPLETED)</p>
          <p className="text-2xl font-bold text-emerald-400 font-mono mt-1">{completedCount}</p>
        </div>
        <div className="bg-slate-900/60 border border-slate-800 p-4 rounded-xl">
          <p className="text-xs font-mono text-slate-400">TỔNG TARGET NODES</p>
          <p className="text-2xl font-bold text-indigo-400 font-mono mt-1">{totalNodesCount}</p>
        </div>
      </div>

      {/* Danh sách các đợt Rollout */}
      <div className="space-y-4">
        {rollouts.length === 0 ? (
          <div className="bg-slate-900/40 border border-slate-800/80 rounded-xl p-12 text-center text-slate-400">
            Chưa có đợt Change Plan Rollout nào được ghi nhận trong PostgreSQL CSDL.
          </div>
        ) : (
          rollouts.map((rollout) => (
            <div key={rollout.id} className="bg-slate-900/60 border border-slate-800 rounded-xl p-6 space-y-4">
              <div className="flex items-start justify-between">
                <div>
                  <div className="flex items-center gap-2">
                    <span className="px-2.5 py-0.5 bg-indigo-500/10 text-indigo-400 border border-indigo-500/20 text-[11px] font-mono rounded-md">
                      {rollout.strategy} STRATEGY
                    </span>
                    <span className="text-xs text-slate-500 font-mono">ID: {rollout.id}</span>
                  </div>
                  <h3 className="text-base font-semibold text-slate-100 mt-2">{rollout.planName}</h3>
                </div>

                {/* Badge Trạng thái Rollout */}
                <span
                  className={`px-3 py-1 text-xs font-semibold rounded-full border flex items-center gap-1.5 ${
                    rollout.status === 'IN_PROGRESS' || rollout.status === 'RUNNING'
                      ? 'bg-sky-500/10 text-sky-400 border-sky-500/20 animate-pulse'
                      : rollout.status === 'PAUSED'
                      ? 'bg-amber-500/10 text-amber-400 border-amber-500/20'
                      : rollout.status === 'CANCELLED' || rollout.status === 'FAILED'
                      ? 'bg-rose-500/10 text-rose-400 border-rose-500/20'
                      : 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
                  }`}
                >
                  <span className="w-1.5 h-1.5 rounded-full bg-current" />
                  {rollout.status}
                </span>
              </div>

              {/* Thanh Progress bar hiển thị tiến độ phần trăm % */}
              <div className="space-y-2">
                <div className="flex justify-between text-xs font-mono">
                  <span className="text-slate-400">Rollout Progress ({rollout.progressPercentage}%)</span>
                  <span className="text-slate-300">
                    {rollout.completedNodes} / {rollout.totalNodes} Nodes Succeeded
                  </span>
                </div>
                <div className="w-full bg-slate-950 h-2.5 rounded-full overflow-hidden border border-slate-800">
                  <div
                    className="bg-indigo-500 h-full transition-all duration-500 rounded-full"
                    style={{ width: `${Math.max(rollout.progressPercentage, 5)}%` }}
                  />
                </div>
              </div>

              {/* Nút hành động tương tác (Sử dụng React Component Dialog) */}
              <div className="flex items-center justify-end gap-3 pt-3 border-t border-slate-800/80">
                {rollout.status === 'IN_PROGRESS' || rollout.status === 'RUNNING' ? (
                  <button
                    onClick={() => triggerActionDialog(rollout.id, 'PAUSE')}
                    className="px-3 py-1.5 bg-amber-500/10 hover:bg-amber-500/20 text-amber-300 border border-amber-500/30 text-xs font-medium rounded-lg transition-colors flex items-center gap-1.5"
                  >
                    <PauseIcon className="w-3.5 h-3.5" />
                    Tạm dừng
                  </button>
                ) : rollout.status === 'PAUSED' ? (
                  <button
                    onClick={() => triggerActionDialog(rollout.id, 'RESUME')}
                    className="px-3 py-1.5 bg-emerald-500/10 hover:bg-emerald-500/20 text-emerald-300 border border-emerald-500/30 text-xs font-medium rounded-lg transition-colors flex items-center gap-1.5"
                  >
                    <PlayIcon className="w-3.5 h-3.5" />
                    Tiếp tục
                  </button>
                ) : null}

                <button
                  onClick={() => triggerActionDialog(rollout.id, 'CANCEL')}
                  className="px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-300 border border-slate-700 text-xs font-medium rounded-lg transition-colors flex items-center gap-1.5"
                >
                  <XCircleIcon className="w-3.5 h-3.5" />
                  Hủy đợt
                </button>

                <button
                  onClick={() => triggerActionDialog(rollout.id, 'ROLLBACK')}
                  className="px-3 py-1.5 bg-rose-500/10 hover:bg-rose-500/20 text-rose-300 border border-rose-500/30 text-xs font-medium rounded-lg transition-colors flex items-center gap-1.5"
                >
                  <RotateCcwIcon className="w-3.5 h-3.5" />
                  Khôi phục Rollback
                </button>
              </div>
            </div>
          ))
        )}
      </div>

      {/* Modal Dialog Tạo Change Plan Mới */}
      <CreateRolloutDialog
        open={createDialogOpen}
        onOpenChange={setCreateDialogOpen}
        onSuccess={loadRollouts}
      />

      {/* Modal Dialog Xác nhận Thao tác Nguy hiểm */}
      <ConfirmRollbackDialog
        open={confirmDialogOpen}
        onOpenChange={setConfirmDialogOpen}
        onConfirm={handleExecuteAction}
        rolloutId={targetRolloutId}
        actionType={actionType}
        loading={actionLoading}
      />
    </div>
  )
}
