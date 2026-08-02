// Approval Workflow & RBAC View Component (Phase 21)
// Hiển thị quy trình Phê duyệt Kép (2-Person Approval), kiểm tra chống tự duyệt và nút duyệt Change Plan

import React, { useState } from 'react'

export interface ApprovalViewProps {
  /** ID đợt Change Plan */
  planId: string
  /** Tên mô tả của Change Plan */
  planName: string
  /** ID người tạo Change Plan */
  creatorId: string
  /** Mức độ rủi ro */
  riskLevel: 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL'
  /** Danh sách chữ ký đã duyệt */
  approvals?: Array<{ approverId: string; approvedAt: string; comments?: string }>
  /** Callback khi thực hiện Approve */
  onApprove?: () => void
}

export const ApprovalWorkflowView: React.FC<ApprovalViewProps> = ({
  planId,
  planName,
  creatorId,
  riskLevel,
  approvals = [],
  onApprove,
}) => {
  // State giả lập ID tài khoản đang đăng nhập hiện tại
  const [currentUser] = useState<string>('user-sec-admin-01')
  // State hiển thị Break-Glass modal
  const [showBreakGlass, setShowBreakGlass] = useState<boolean>(false)

  // Kiểm tra xem user hiện tại có phải người tạo hay không (Anti Self-Approval check)
  const isCreator = currentUser === creatorId
  // Kiểm tra xem user hiện tại đã duyệt chưa
  const hasAlreadyApproved = approvals.some((a) => a.approverId === currentUser)

  return (
    <div className="bg-slate-900/60 border border-slate-800 rounded-xl p-6 space-y-4">
      {/* 1. Header hiển thị thông tin Plan & Risk Badge */}
      <div className="flex items-start justify-between border-b border-slate-800/80 pb-4">
        <div>
          <div className="flex items-center gap-2">
            <span className="text-xs font-mono text-slate-400">PLAN ID: {planId}</span>
            <span className="text-xs text-slate-500">• Created by {creatorId}</span>
          </div>
          <h3 className="text-lg font-semibold text-slate-100 mt-1">{planName}</h3>
        </div>

        {/* Mức rủi ro Risk Tier Badge */}
        <span
          className={`px-3 py-1 text-xs font-bold font-mono rounded border ${
            riskLevel === 'CRITICAL'
              ? 'bg-rose-500/10 text-rose-400 border-rose-500/30 animate-pulse'
              : riskLevel === 'HIGH'
              ? 'bg-amber-500/10 text-amber-400 border-amber-500/30'
              : 'bg-sky-500/10 text-sky-400 border-sky-500/30'
          }`}
        >
          RISK TIER: {riskLevel}
        </span>
      </div>

      {/* 2. Cảnh báo Quy tắc Duyệt (2-Person Approval & Anti Self-Approval) */}
      {riskLevel === 'CRITICAL' && (
        <div className="bg-amber-500/10 border border-amber-500/20 rounded-lg p-3 text-xs text-amber-300">
          ⚠️ <strong>Quy tắc 2-Person Approval</strong>: Thay đổi mức <strong>CRITICAL</strong> yêu cầu tối thiểu{' '}
          <strong>2 chữ ký phê duyệt</strong> từ 2 quản trị viên khác nhau. Người tạo (
          <code className="text-amber-200">{creatorId}</code>) tuyệt đối không được tự duyệt plan của mình.
        </div>
      )}

      {/* 3. Danh sách các chữ ký Phê duyệt đã thu thập được */}
      <div className="space-y-2">
        <h4 className="text-xs font-medium text-slate-400 uppercase tracking-wider">
          Approval Signatures ({approvals.length} / {riskLevel === 'CRITICAL' ? 2 : 1})
        </h4>

        {approvals.length === 0 ? (
          <p className="text-xs italic text-slate-500">Chưa có chữ ký phê duyệt nào.</p>
        ) : (
          <div className="space-y-1.5">
            {approvals.map((a, i) => (
              <div key={i} className="flex items-center justify-between bg-slate-950/40 px-3 py-2 rounded text-xs">
                <span className="font-mono text-emerald-400">✓ Approved by {a.approverId}</span>
                <span className="text-slate-500 font-mono">{a.approvedAt}</span>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* 4. Action Bar (Approve / Reject / Break-Glass Emergency) */}
      <div className="flex items-center justify-between pt-4 border-t border-slate-800/80">
        <button
          onClick={() => setShowBreakGlass(true)}
          className="text-xs text-rose-400 hover:text-rose-300 underline font-mono"
        >
          🚨 Trigger Break-Glass Emergency Mode
        </button>

        <div className="flex items-center gap-3">
          {isCreator ? (
            <span className="text-xs text-amber-400 italic">
              ⛔ Bạn là người tạo plan này. Anti Self-Approval cấm bạn tự duyệt.
            </span>
          ) : hasAlreadyApproved ? (
            <span className="text-xs text-emerald-400 font-medium">✓ Bạn đã phê duyệt Change Plan này</span>
          ) : (
            <button
              onClick={() => onApprove?.()}
              className="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 text-white font-medium text-xs rounded-lg transition-colors shadow-lg shadow-emerald-600/20"
            >
              Sign & Approve Plan
            </button>
          )}
        </div>
      </div>

      {/* Modal Cảnh báo Break-Glass Emergency */}
      {showBreakGlass && (
        <div className="fixed inset-0 bg-black/80 flex items-center justify-center p-4 z-50">
          <div className="bg-slate-900 border border-rose-500/40 rounded-xl p-6 max-w-md w-full space-y-4 shadow-2xl">
            <h3 className="text-lg font-bold text-rose-400">🚨 Trigger Break-Glass Emergency Override</h3>
            <p className="text-xs text-slate-300">
              Chế độ Khẩn cấp cho phép bypass kiểm tra phê duyệt trong tình huống sự cố nghiêm trọng. Mọi thao tác sẽ bị
              ghi log Audit Trail vĩnh viễn và gửi cảnh báo tới toàn bộ đội ngũ Security.
            </p>
            <div className="space-y-1">
              <label className="text-[11px] text-slate-400">Lý do kích hoạt Break-Glass (bắt buộc):</label>
              <textarea
                className="w-full bg-slate-950 border border-slate-800 rounded p-2 text-xs text-slate-200 focus:outline-none focus:border-rose-500"
                placeholder="VD: Outage khẩn cấp trên Production cluster..."
                rows={3}
              />
            </div>
            <div className="flex justify-end gap-2">
              <button
                onClick={() => setShowBreakGlass(false)}
                className="px-3 py-1.5 bg-slate-800 text-slate-300 text-xs rounded"
              >
                Hủy
              </button>
              <button
                onClick={() => {
                  alert('Đã kích hoạt Break-Glass Emergency Mode (Thời hạn: 1 giờ)')
                  setShowBreakGlass(false)
                }}
                className="px-3 py-1.5 bg-rose-600 hover:bg-rose-500 text-white text-xs font-bold rounded"
              >
                Kích hoạt Break-Glass
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
