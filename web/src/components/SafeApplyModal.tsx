// SafeApplyModal Component - Hiển thị đếm ngược Confirm / Rollback trong quá trình Safe Apply
// Lưu trữ trạng thái PENDING bền vững qua F5 refresh bằng localStorage

import { useState, useEffect } from 'react'
import { AlertTriangleIcon, CheckCircle2Icon, RotateCcwIcon, ShieldCheckIcon } from 'lucide-react'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter } from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import { Badge } from '@/components/ui/badge'
import { api, type ApplyExecution } from '@/api/client'

interface SafeApplyModalProps {
  execution: ApplyExecution | null
  onClose: () => void
  onSuccess?: () => void
}

export function SafeApplyModal({ execution, onClose, onSuccess }: SafeApplyModalProps) {
  const [currentExec, setCurrentExec] = useState<ApplyExecution | null>(execution)
  const [timeLeft, setTimeLeft] = useState<number>(30)
  const [loading, setLoading] = useState<boolean>(false)
  const [statusMsg, setStatusMsg] = useState<string>('')

  // Khôi phục pending execution từ localStorage nếu F5 refresh trang
  useEffect(() => {
    if (execution) {
      setCurrentExec(execution)
      localStorage.setItem('aegis_pending_execution', JSON.stringify(execution))
    } else {
      const saved = localStorage.getItem('aegis_pending_execution')
      if (saved) {
        try {
          const parsed = JSON.parse(saved)
          setCurrentExec(parsed)
        } catch {
          localStorage.removeItem('aegis_pending_execution')
        }
      }
    }
  }, [execution])

  // Đếm ngược Rollback Timer
  useEffect(() => {
    if (!currentExec || currentExec.state !== 'APPLIED_PENDING_CONFIRMATION') return

    const initialTimeout = currentExec.timeoutSeconds || 30
    setTimeLeft(initialTimeout)

    const interval = setInterval(() => {
      setTimeLeft((prev) => {
        if (prev <= 1) {
          clearInterval(interval)
          handleAutoRollbackExpired()
          return 0
        }
        return prev - 1
      })
    }, 1000)

    return () => clearInterval(interval)
  }, [currentExec])

  const handleAutoRollbackExpired = () => {
    setStatusMsg('Time expired! Policy automatically rolled back.')
    localStorage.removeItem('aegis_pending_execution')
  }

  const handleConfirm = async () => {
    if (!currentExec) return
    setLoading(true)
    try {
      const updated = await api.confirmApply(currentExec.executionId)
      setCurrentExec(updated)
      localStorage.removeItem('aegis_pending_execution')
      setStatusMsg('Policy successfully committed!')
      if (onSuccess) onSuccess()
    } catch (err: any) {
      setStatusMsg(`Confirm failed: ${err.message}`)
    } finally {
      setLoading(false)
    }
  }

  const handleRollback = async () => {
    if (!currentExec) return
    setLoading(true)
    try {
      const updated = await api.rollbackApply(currentExec.executionId)
      setCurrentExec(updated)
      localStorage.removeItem('aegis_pending_execution')
      setStatusMsg('Policy rolled back to previous snapshot!')
    } catch (err: any) {
      setStatusMsg(`Rollback failed: ${err.message}`)
    } finally {
      setLoading(false)
    }
  }

  if (!currentExec) return null

  const isPending = currentExec.state === 'APPLIED_PENDING_CONFIRMATION'
  const isCommitted = currentExec.state === 'COMMITTED'
  const isRolledBack = currentExec.state === 'ROLLED_BACK'
  const progressPercent = ((timeLeft / (currentExec.timeoutSeconds || 30)) * 100)

  return (
    <Dialog open={!!currentExec} onOpenChange={(open) => !open && isPending ? null : onClose()}>
      <DialogContent className="sm:max-w-md border-primary/30 glass-card">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            {isPending && <AlertTriangleIcon className="h-5 w-5 text-amber-400 animate-pulse" />}
            {isCommitted && <CheckCircle2Icon className="h-5 w-5 text-green-400" />}
            {isRolledBack && <RotateCcwIcon className="h-5 w-5 text-destructive" />}
            Safe Apply Execution
          </DialogTitle>
          <DialogDescription className="text-xs font-mono text-muted-foreground truncate">
            ID: {currentExec.executionId}
          </DialogDescription>
        </DialogHeader>

        {isPending && (
          <div className="space-y-4 py-3">
            <div className="flex items-center justify-between text-sm">
              <span className="text-muted-foreground flex items-center gap-1.5">
                <ShieldCheckIcon className="h-4 w-4 text-cyan-400" />
                Automatic Rollback Timer:
              </span>
              <span className="font-mono text-lg font-bold text-amber-400">{timeLeft}s</span>
            </div>

            <Progress value={progressPercent} className="h-2 bg-muted" />

            <div className="rounded-md bg-muted/40 p-3 text-xs space-y-1 border border-border">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Snapshot ID:</span>
                <span className="font-mono">{currentExec.snapshotId.slice(0, 8)}...</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Health Check:</span>
                <Badge variant="success" className="text-[10px]">PASSED</Badge>
              </div>
            </div>

            <p className="text-xs text-muted-foreground bg-amber-500/10 border border-amber-500/20 p-2.5 rounded text-amber-300">
              Chờ xác nhận từ người quản trị. Nếu mất kết nối hoặc hết {timeLeft} giây, kịch bản tự động Rollback về trạng thái cũ.
            </p>
          </div>
        )}

        {statusMsg && (
          <div className={`p-3 rounded text-xs font-medium ${isCommitted ? 'bg-green-500/10 text-green-400 border border-green-500/30' : 'bg-destructive/10 text-destructive border border-destructive/30'}`}>
            {statusMsg}
          </div>
        )}

        <DialogFooter className="gap-2 sm:gap-0">
          {isPending ? (
            <>
              <Button variant="destructive" onClick={handleRollback} disabled={loading} className="gap-1.5">
                <RotateCcwIcon className="h-4 w-4" />
                Rollback Now
              </Button>
              <Button variant="success" onClick={handleConfirm} disabled={loading} className="gap-1.5">
                <CheckCircle2Icon className="h-4 w-4" />
                Confirm & Commit
              </Button>
            </>
          ) : (
            <Button variant="outline" onClick={onClose}>
              Close Window
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
