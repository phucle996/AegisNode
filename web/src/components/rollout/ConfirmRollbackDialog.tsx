// Confirm Rollback Dialog Component
// Modal Dialog xác nhận thao tác nguy hiểm (Rollback / Cancel Rollout) thay thế cho window.confirm()

import React from 'react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { AlertTriangleIcon } from 'lucide-react'

interface ConfirmRollbackDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onConfirm: () => void
  rolloutId: string
  actionType: 'PAUSE' | 'RESUME' | 'CANCEL' | 'ROLLBACK'
  loading?: boolean
}

export const ConfirmRollbackDialog: React.FC<ConfirmRollbackDialogProps> = ({
  open,
  onOpenChange,
  onConfirm,
  rolloutId,
  actionType,
  loading = false,
}) => {
  // Chuẩn bị nhãn nội dung theo từng loại thao tác
  const getActionDetails = () => {
    switch (actionType) {
      case 'PAUSE':
        return {
          title: 'Tạm dừng đợt Triển khai Rollout',
          desc: 'Quá trình áp dụng cấu hình trên các Node tiếp theo sẽ bị tạm dừng cho đến khi được Resume.',
          confirmText: 'Xác nhận Tạm dừng',
          btnColor: 'bg-amber-600 hover:bg-amber-500 shadow-amber-600/20',
        }
      case 'RESUME':
        return {
          title: 'Tiếp tục đợt Triển khai Rollout',
          desc: 'Hệ thống sẽ tiếp tục áp dụng cấu hình sang đợt Node (Batch) kế tiếp.',
          confirmText: 'Xác nhận Tiếp tục',
          btnColor: 'bg-indigo-600 hover:bg-indigo-500 shadow-indigo-600/20',
        }
      case 'CANCEL':
        return {
          title: 'Hủy bỏ đợt Triển khai Rollout',
          desc: 'Hệ thống sẽ ngắt ngay đợt Rollout này. Các Node chưa áp dụng sẽ giữ nguyên cấu hình cũ.',
          confirmText: 'Xác nhận Hủy bỏ',
          btnColor: 'bg-rose-600 hover:bg-rose-500 shadow-rose-600/20',
        }
      case 'ROLLBACK':
      default:
        return {
          title: 'Khôi phục khẩn cấp (Emergency Fleet Rollback)',
          desc: 'Toàn bộ các Node đã nhận cấu hình mới sẽ lập tức đảo ngược (LIFO) về phiên bản an toàn trước đó.',
          confirmText: 'Xác nhận Khôi phục Rollback',
          btnColor: 'bg-rose-600 hover:bg-rose-500 shadow-rose-600/20',
        }
    }
  }

  const details = getActionDetails()

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[450px] bg-slate-900 border-slate-800 text-slate-100">
        <DialogHeader>
          <div className="flex items-center gap-3">
            <div className="p-2.5 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400">
              <AlertTriangleIcon className="w-5 h-5" />
            </div>
            <div>
              <DialogTitle className="text-slate-100 text-base font-bold">
                {details.title}
              </DialogTitle>
              <p className="text-[11px] font-mono text-slate-400 mt-0.5">
                Rollout ID: {rolloutId.substring(0, 18)}...
              </p>
            </div>
          </div>
          <DialogDescription className="text-slate-300 text-xs mt-3 bg-slate-950/60 p-3 rounded-lg border border-slate-800">
            {details.desc}
          </DialogDescription>
        </DialogHeader>

        <DialogFooter className="pt-3 border-t border-slate-800">
          <button
            type="button"
            onClick={() => onOpenChange(false)}
            className="px-4 py-2 bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-medium rounded-lg transition-colors"
          >
            Bỏ qua
          </button>
          <button
            type="button"
            disabled={loading}
            onClick={() => {
              onConfirm()
              onOpenChange(false)
            }}
            className={`px-4 py-2 text-white text-xs font-medium rounded-lg transition-colors shadow-lg disabled:opacity-50 ${details.btnColor}`}
          >
            {loading ? 'Đang xử lý...' : details.confirmText}
          </button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
