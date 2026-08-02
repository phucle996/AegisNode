// Create Rollout Dialog Component
// Modal Dialog khởi tạo đợt triển khai Multi-Node Rollout Change Plan mới (Sử dụng React Radix Dialog Component)

import React, { useState } from 'react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { rolloutApi, CreateRolloutPayload } from '../../api/rolloutClient'

interface CreateRolloutDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSuccess: () => void
}

export const CreateRolloutDialog: React.FC<CreateRolloutDialogProps> = ({
  open,
  onOpenChange,
  onSuccess,
}) => {
  // State quản lý thông tin form tạo Rollout
  const [strategy, setStrategy] = useState<'CANARY' | 'BATCH' | 'ROLLING'>('CANARY')
  const [riskLevel, setRiskLevel] = useState<'LOW' | 'MEDIUM' | 'HIGH'>('LOW')
  const [batchSize, setBatchSize] = useState<number>(1)
  const [failureThreshold, setFailureThreshold] = useState<number>(20)
  const [loading, setLoading] = useState<boolean>(false)

  // Xử lý gửi Form phát hành Change Plan mới
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setLoading(true)
    try {
      const payload: CreateRolloutPayload = {
        idempotencyKey: `plan-${Date.now()}`,
        strategy,
        riskLevel,
        batchSize,
        maxUnavailable: 1,
        failureThresholdPercent: failureThreshold,
      }

      // Gọi Controller REST API POST /v1/rollouts
      await rolloutApi.createRollout(payload)
      onSuccess()
      onOpenChange(false)
    } catch (error) {
      console.error('Lỗi khởi tạo Rollout Plan:', error)
    } finally {
      setLoading(false)
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[500px] bg-slate-900 border-slate-800 text-slate-100">
        <DialogHeader>
          <DialogTitle className="text-slate-100 flex items-center gap-2">
            <span className="w-2 h-2 rounded-full bg-indigo-500" />
            Khởi tạo Multi-Node Change Plan mới
          </DialogTitle>
          <DialogDescription className="text-slate-400 text-xs">
            Cấu hình chiến lược triển khai Canary/Batch, số lượng Node/đợt và ngưỡng tự động khôi phục an toàn.
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4 py-2">
          {/* 1. Chọn chiến lược Rollout Strategy */}
          <div>
            <label className="block text-xs font-mono text-slate-300 mb-1.5">Triển khai Strategy</label>
            <select
              value={strategy}
              onChange={(e) => setStrategy(e.target.value as any)}
              className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-xs font-mono text-slate-200 focus:outline-none focus:border-indigo-500"
            >
              <option value="CANARY">CANARY STRATEGY (10% Test Batch trước)</option>
              <option value="BATCH">BATCH STRATEGY (Chia thành nhiều đợt nhỏ)</option>
              <option value="ROLLING">ROLLING UPDATE (Cập nhật cuốn chiếu)</option>
            </select>
          </div>

          {/* 2. Chọn Mức độ rủi ro Risk Level */}
          <div>
            <label className="block text-xs font-mono text-slate-300 mb-1.5">Mức độ Rủi ro (Risk Level)</label>
            <div className="grid grid-cols-3 gap-2">
              {(['LOW', 'MEDIUM', 'HIGH'] as const).map((level) => (
                <button
                  type="button"
                  key={level}
                  onClick={() => setRiskLevel(level)}
                  className={`py-2 px-3 text-xs font-mono rounded-lg border transition-colors ${
                    riskLevel === level
                      ? level === 'LOW'
                        ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30'
                        : level === 'MEDIUM'
                        ? 'bg-amber-500/10 text-amber-400 border-amber-500/30'
                        : 'bg-rose-500/10 text-rose-400 border-rose-500/30'
                      : 'bg-slate-950 text-slate-400 border-slate-800 hover:border-slate-700'
                  }`}
                >
                  {level} RISK
                </button>
              ))}
            </div>
          </div>

          {/* 3. Cấu hình Batch Size và Failure Threshold % */}
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-mono text-slate-300 mb-1.5">Batch Size (Số Node/Đợt)</label>
              <input
                type="number"
                min={1}
                max={50}
                value={batchSize}
                onChange={(e) => setBatchSize(parseInt(e.target.value, 10) || 1)}
                className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-xs font-mono text-slate-200 focus:outline-none focus:border-indigo-500"
              />
            </div>
            <div>
              <label className="block text-xs font-mono text-slate-300 mb-1.5">Failure Threshold (%)</label>
              <input
                type="number"
                min={1}
                max={100}
                value={failureThreshold}
                onChange={(e) => setFailureThreshold(parseInt(e.target.value, 10) || 20)}
                className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-xs font-mono text-slate-200 focus:outline-none focus:border-indigo-500"
              />
            </div>
          </div>

          <DialogFooter className="pt-4 border-t border-slate-800">
            <button
              type="button"
              onClick={() => onOpenChange(false)}
              className="px-4 py-2 bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-medium rounded-lg transition-colors"
            >
              Hủy bỏ
            </button>
            <button
              type="submit"
              disabled={loading}
              className="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-medium rounded-lg transition-colors shadow-lg shadow-indigo-600/20 disabled:opacity-50"
            >
              {loading ? 'Đang khởi tạo...' : 'Phát hành Change Plan'}
            </button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
