// Plan Setup Studio Component
// Phân hệ soạn thảo và thiết lập Change Plan trực quan (Target, Strategy, Step Chain Builder & Health Check Probes)

import React, { useState } from 'react'
import { rolloutApi } from '../../api/rolloutClient'
import { PlusIcon, Trash2Icon, RocketIcon } from 'lucide-react'

export interface StepItem {
  id: string
  name: string
  component: 'firewall' | 'network' | 'systemd'
  action: string
  details: string
}

interface PlanSetupStudioProps {
  onLaunchSuccess: () => void
}

export const PlanSetupStudio: React.FC<PlanSetupStudioProps> = ({ onLaunchSuccess }) => {
  // State quản lý thông tin chung của Change Plan
  const [planName, setPlanName] = useState<string>(`plan-release-v1.${Date.now().toString().slice(-4)}`)
  const [targetNodeId, setTargetNodeId] = useState<string>('c9c9379a-79d9-4a27-b9fa-46dee7c728b2')
  const [strategy, setStrategy] = useState<'CANARY' | 'BATCH' | 'ROLLING'>('CANARY')
  const [riskLevel, setRiskLevel] = useState<'LOW' | 'MEDIUM' | 'HIGH'>('LOW')
  const [batchSize, setBatchSize] = useState<number>(1)
  const [failureThreshold, setFailureThreshold] = useState<number>(20)

  // State danh sách các bước thực thi (Execution Step Chain)
  const [steps, setSteps] = useState<StepItem[]>([
    {
      id: 'step-1',
      name: '01_create_system_snapshot',
      component: 'firewall',
      action: 'CREATE_SNAPSHOT',
      details: 'Sao lưu trạng thái cấu hình hiện tại trước khi thay đổi',
    },
    {
      id: 'step-2',
      name: '02_apply_firewall_policy_v1.1.5',
      component: 'firewall',
      action: 'APPLY_FIREWALL_POLICY',
      details: 'Áp dụng bộ luật Firewall Rules: Allow SSH (22), HTTPS (443)',
    },
    {
      id: 'step-3',
      name: '03_restart_nftables_service',
      component: 'systemd',
      action: 'RESTART_SERVICE',
      details: 'Khởi động lại dịch vụ nftables daemon',
    },
  ])

  // State cấu hình Health Check Safety Probes
  const [probeGateway, setProbeGateway] = useState<boolean>(true)
  const [probeDns, setProbeDns] = useState<boolean>(true)
  const [timeoutSeconds, setTimeoutSeconds] = useState<number>(30)

  // State loại bước đang thêm mới
  const [newComponent, setNewComponent] = useState<'firewall' | 'network' | 'systemd'>('firewall')
  const [newAction, setNewAction] = useState<string>('APPLY_POLICY')
  const [newDetails, setNewDetails] = useState<string>('Thay đổi cấu hình')

  // Trạng thái đang bấm nút Launch Rollout
  const [loading, setLoading] = useState<boolean>(false)

  // Thêm bước mới vào chuỗi thực thi
  const handleAddStep = () => {
    const nextOrder = steps.length + 1
    const newStepItem: StepItem = {
      id: `step-${Date.now()}`,
      name: `0${nextOrder}_${newComponent}_${newAction.toLowerCase()}`,
      component: newComponent,
      action: newAction,
      details: newDetails,
    }
    setSteps([...steps, newStepItem])
    setNewDetails('Thay đổi cấu hình')
  }

  // Xóa bước khỏi chuỗi thực thi
  const handleRemoveStep = (id: string) => {
    if (steps.length <= 1) return
    setSteps(steps.filter((s) => s.id !== id))
  }

  // Thực thi phát hành Change Plan mới và Launch Rollout
  const handleLaunchRollout = async (e: React.FormEvent) => {
    e.preventDefault()
    setLoading(true)
    try {
      // Đóng gói Payload gửi sang REST API Controller
      await rolloutApi.createRollout({
        idempotencyKey: planName,
        strategy,
        riskLevel,
        batchSize,
        maxUnavailable: 1,
        failureThresholdPercent: failureThreshold,
        targetNodeId,
      })

      // Tự động kích hoạt callback chuyển Tab sang Tab 1 (Summary & Progress)
      onLaunchSuccess()
    } catch (error) {
      console.error('Lỗi phát hành Change Plan:', error)
    } finally {
      setLoading(false)
    }
  }

  return (
    <form onSubmit={handleLaunchRollout} className="space-y-6">
      {/* 1. Phần Cấu hình Mục tiêu & Chiến lược Triển khai */}
      <div className="bg-slate-900/60 backdrop-blur border border-slate-800 p-6 rounded-2xl space-y-4">
        <h3 className="text-base font-bold text-slate-100 flex items-center gap-2">
          <span className="w-2.5 h-2.5 rounded-full bg-indigo-500" />
          1. Thông tin Chung & Chiến lược Triển khai (Target & Strategy)
        </h3>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <label className="block text-xs font-mono text-slate-300 mb-1.5">Tên Change Plan (Idempotency Key)</label>
            <input
              type="text"
              value={planName}
              onChange={(e) => setPlanName(e.target.value)}
              className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-xs font-mono text-slate-200 focus:outline-none focus:border-indigo-500"
              required
            />
          </div>

          <div>
            <label className="block text-xs font-mono text-slate-300 mb-1.5">Node Mục tiêu (Target Node)</label>
            <select
              value={targetNodeId}
              onChange={(e) => setTargetNodeId(e.target.value)}
              className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-xs font-mono text-slate-200 focus:outline-none focus:border-indigo-500"
            >
              <option value="c9c9379a-79d9-4a27-b9fa-46dee7c728b2">ubuntu-node-2 (192.168.122.102)</option>
              <option value="a2e84822-2162-4842-910f-105406bc733b">ubuntu-node-3 (192.168.122.135)</option>
              <option value="eff5cf5a-6688-47c2-ad91-6ac21f08cdc5">ubuntu-node-1 (192.168.122.109 - Controller)</option>
            </select>
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 pt-2">
          <div>
            <label className="block text-xs font-mono text-slate-300 mb-1.5">Chiến lược (Strategy)</label>
            <select
              value={strategy}
              onChange={(e) => setStrategy(e.target.value as any)}
              className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-xs font-mono text-slate-200 focus:outline-none focus:border-indigo-500"
            >
              <option value="CANARY">CANARY STRATEGY (Test batch 10% trước)</option>
              <option value="BATCH">BATCH STRATEGY (Chia thành nhiều đợt nhỏ)</option>
              <option value="ROLLING">ROLLING UPDATE (Cập nhật cuốn chiếu)</option>
            </select>
          </div>

          <div>
            <label className="block text-xs font-mono text-slate-300 mb-1.5">Mức độ Rủi ro (Risk Level)</label>
            <select
              value={riskLevel}
              onChange={(e) => setRiskLevel(e.target.value as any)}
              className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-xs font-mono text-slate-200 focus:outline-none focus:border-indigo-500"
            >
              <option value="LOW">LOW RISK</option>
              <option value="MEDIUM">MEDIUM RISK</option>
              <option value="HIGH">HIGH RISK</option>
            </select>
          </div>

          <div>
            <label className="block text-xs font-mono text-slate-300 mb-1.5">Batch Size (Node / Đợt)</label>
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
      </div>

      {/* 2. Trình Soạn thảo Chuỗi các Bước Thực thi (Execution Step Chain Builder) */}
      <div className="bg-slate-900/60 backdrop-blur border border-slate-800 p-6 rounded-2xl space-y-4">
        <div className="flex items-center justify-between">
          <h3 className="text-base font-bold text-slate-100 flex items-center gap-2">
            <span className="w-2.5 h-2.5 rounded-full bg-sky-500" />
            2. Trình Soạn thảo Chuỗi Thực thi (Execution Step Chain Builder)
          </h3>
          <span className="text-xs font-mono text-slate-400">{steps.length} Bước được thiết lập</span>
        </div>

        {/* Danh sách các Bước hiện tại */}
        <div className="space-y-3">
          {steps.map((step, idx) => (
            <div
              key={step.id}
              className="bg-slate-950/80 border border-slate-800 rounded-xl p-4 flex items-center justify-between"
            >
              <div className="flex items-center gap-3">
                <span className="flex items-center justify-center w-7 h-7 rounded-lg bg-indigo-500/10 border border-indigo-500/20 text-indigo-400 font-mono text-xs font-bold">
                  #{idx + 1}
                </span>
                <div>
                  <div className="flex items-center gap-2">
                    <span className="font-mono text-xs font-semibold text-slate-100">{step.name}</span>
                    <span
                      className={`px-2 py-0.5 text-[10px] font-mono rounded border ${
                        step.component === 'firewall'
                          ? 'bg-rose-500/10 text-rose-400 border-rose-500/20'
                          : step.component === 'network'
                          ? 'bg-sky-500/10 text-sky-400 border-sky-500/20'
                          : 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
                      }`}
                    >
                      {step.component.toUpperCase()}
                    </span>
                  </div>
                  <p className="text-xs text-slate-400 mt-1">{step.details}</p>
                </div>
              </div>

              <button
                type="button"
                onClick={() => handleRemoveStep(step.id)}
                disabled={steps.length <= 1}
                className="p-2 text-slate-400 hover:text-rose-400 transition-colors disabled:opacity-30"
              >
                <Trash2Icon className="w-4 h-4" />
              </button>
            </div>
          ))}
        </div>

        {/* Form thêm bước mới */}
        <div className="pt-4 border-t border-slate-800/80 grid grid-cols-1 md:grid-cols-4 gap-3">
          <div>
            <label className="block text-[11px] font-mono text-slate-400 mb-1">Thành phần (Component)</label>
            <select
              value={newComponent}
              onChange={(e) => setNewComponent(e.target.value as any)}
              className="w-full bg-slate-950 border border-slate-800 rounded-lg px-2.5 py-1.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-indigo-500"
            >
              <option value="firewall">Firewall Policy</option>
              <option value="network">Network Interface</option>
              <option value="systemd">Systemd Service</option>
            </select>
          </div>

          <div>
            <label className="block text-[11px] font-mono text-slate-400 mb-1">Hành động (Action)</label>
            <input
              type="text"
              value={newAction}
              onChange={(e) => setNewAction(e.target.value)}
              className="w-full bg-slate-950 border border-slate-800 rounded-lg px-2.5 py-1.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-indigo-500"
            />
          </div>

          <div>
            <label className="block text-[11px] font-mono text-slate-400 mb-1">Mô tả Chi tiết</label>
            <input
              type="text"
              value={newDetails}
              onChange={(e) => setNewDetails(e.target.value)}
              className="w-full bg-slate-950 border border-slate-800 rounded-lg px-2.5 py-1.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-indigo-500"
            />
          </div>

          <div className="flex items-end">
            <button
              type="button"
              onClick={handleAddStep}
              className="w-full py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 font-medium text-xs rounded-lg transition-colors flex items-center justify-center gap-1.5 border border-slate-700"
            >
              <PlusIcon className="w-3.5 h-3.5" />
              Thêm Bước mới
            </button>
          </div>
        </div>
      </div>

      {/* 3. Phần Cấu hình Kiểm tra An toàn (Health Check Probes) */}
      <div className="bg-slate-900/60 backdrop-blur border border-slate-800 p-6 rounded-2xl space-y-4">
        <h3 className="text-base font-bold text-slate-100 flex items-center gap-2">
          <span className="w-2.5 h-2.5 rounded-full bg-emerald-500" />
          3. Kiểm tra An toàn & Khôi phục Tự động (Health Check Probes)
        </h3>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <label className="flex items-center gap-3 p-3 bg-slate-950/60 border border-slate-800 rounded-xl cursor-pointer hover:border-slate-700">
            <input
              type="checkbox"
              checked={probeGateway}
              onChange={(e) => setProbeGateway(e.target.checked)}
              className="rounded bg-slate-900 border-slate-700 text-indigo-600 focus:ring-indigo-500"
            />
            <div>
              <p className="text-xs font-semibold text-slate-200">Probe Gateway Ping</p>
              <p className="text-[10px] text-slate-400">Kiểm tra kết nối tới Default Gateway</p>
            </div>
          </label>

          <label className="flex items-center gap-3 p-3 bg-slate-950/60 border border-slate-800 rounded-xl cursor-pointer hover:border-slate-700">
            <input
              type="checkbox"
              checked={probeDns}
              onChange={(e) => setProbeDns(e.target.checked)}
              className="rounded bg-slate-900 border-slate-700 text-indigo-600 focus:ring-indigo-500"
            />
            <div>
              <p className="text-xs font-semibold text-slate-200">Probe DNS Resolution</p>
              <p className="text-[10px] text-slate-400">Kiểm tra phân giải tên miền DNS</p>
            </div>
          </label>

          <div>
            <label className="block text-xs font-mono text-slate-300 mb-1.5">Timeout Khôi phục (Giây)</label>
            <input
              type="number"
              min={5}
              max={300}
              value={timeoutSeconds}
              onChange={(e) => setTimeoutSeconds(parseInt(e.target.value, 10) || 30)}
              className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-xs font-mono text-slate-200 focus:outline-none focus:border-indigo-500"
            />
          </div>
        </div>
      </div>

      {/* Footer Submit Button: Launch Rollout */}
      <div className="flex justify-end pt-2">
        <button
          type="submit"
          disabled={loading}
          className="px-6 py-3 bg-indigo-600 hover:bg-indigo-500 text-white font-medium text-xs rounded-xl transition-all shadow-lg shadow-indigo-600/25 flex items-center gap-2 disabled:opacity-50"
        >
          <RocketIcon className="w-4 h-4" />
          {loading ? 'Đang kích hoạt...' : '🚀 Launch Multi-Node Rollout Now'}
        </button>
      </div>
    </form>
  )
}
