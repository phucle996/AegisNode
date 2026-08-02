// Registered Nodes Management Component
// Hiển thị tổng quan danh sách các máy chủ Nodes trong hệ thống, chỉ số giám sát và trạng thái thực tế

import React, { useEffect, useState } from 'react'
import { nodesApi } from '../api/nodesClient'
import { FleetNode, SystemMetrics } from '../types/fleet'

export const NodesOverview: React.FC = () => {
  // State quản lý danh sách Node thực tế
  const [nodes, setNodes] = useState<FleetNode[]>([])
  // State quản lý các chỉ số giám sát Prometheus Metrics
  const [metrics, setMetrics] = useState<SystemMetrics | null>(null)
  // State lưu trạng thái loading dữ liệu
  const [loading, setLoading] = useState<boolean>(true)

  // Fetch dữ liệu thực tế từ Controller API khi Component mount
  useEffect(() => {
    const fetchData = async () => {
      setLoading(true)
      // Lấy danh sách Node thực tế từ Controller /v1/nodes
      const fetchedNodes = await nodesApi.getNodes()
      const fetchedMetrics = await nodesApi.getMetrics()
      setNodes(fetchedNodes)
      setMetrics(fetchedMetrics)
      setLoading(false)
    }

    fetchData()
    // Tự động làm mới dữ liệu định kỳ mỗi 5 giây
    const interval = setInterval(fetchData, 5000)
    return () => clearInterval(interval)
  }, [])

  if (loading && !nodes.length) {
    return <div className="p-6 text-slate-400 font-mono">Đang kết nối Controller & tải danh sách Nodes...</div>
  }

  // Tính toán số lượng Node theo trạng thái thực tế
  const onlineCount = nodes.filter((n) => n.status === 'ONLINE').length
  const degradedCount = nodes.filter((n) => n.status === 'DEGRADED' || n.status === 'OFFLINE').length

  return (
    <div className="space-y-6">
      {/* 1. Header & Summary Cards */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-bold text-slate-100">Registered Nodes Overview</h2>
          <p className="text-sm text-slate-400">Quản lý danh sách máy chủ AegisNode Controller & Agents trong hệ thống</p>
        </div>
        <span className="px-3 py-1 bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 text-xs font-mono rounded-full">
          ● Central Controller Active
        </span>
      </div>

      {/* Grid chứa 4 thẻ Thống kê Metrics */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="bg-slate-900/60 border border-slate-800 rounded-xl p-4">
          <div className="text-xs font-medium text-slate-400">Total Registered Nodes</div>
          <div className="text-2xl font-bold text-slate-100 mt-1">{nodes.length} Nodes</div>
          <div className="text-xs text-emerald-400 mt-2 font-mono">
            {onlineCount} Online | {degradedCount} Offline
          </div>
        </div>

        <div className="bg-slate-900/60 border border-slate-800 rounded-xl p-4">
          <div className="text-xs font-medium text-slate-400">Connected mTLS Agents</div>
          <div className="text-2xl font-bold text-sky-400 mt-1">{metrics?.connectedAgents || nodes.length} Connected</div>
          <div className="text-xs text-slate-500 mt-2">Certified agent identity</div>
        </div>

        <div className="bg-slate-900/60 border border-slate-800 rounded-xl p-4">
          <div className="text-xs font-medium text-slate-400">Firewall Packets Dropped</div>
          <div className="text-2xl font-bold text-amber-400 mt-1">
            {metrics?.firewallDropsTotal.toLocaleString() || 0}
          </div>
          <div className="text-xs text-slate-500 mt-2">nftables kernel filter drops</div>
        </div>

        <div className="bg-slate-900/60 border border-slate-800 rounded-xl p-4">
          <div className="text-xs font-medium text-slate-400">Auto-Blocked IPs</div>
          <div className="text-2xl font-bold text-rose-400 mt-1">{metrics?.activeBlocksTotal || 0} Active</div>
          <div className="text-xs text-rose-400/80 mt-2">SSH Brute-force blocked</div>
        </div>
      </div>

      {/* 2. Bảng Danh sách Nodes thực tế */}
      <div className="bg-slate-900/60 border border-slate-800 rounded-xl overflow-hidden">
        <div className="px-6 py-4 border-b border-slate-800 flex items-center justify-between">
          <h3 className="text-sm font-semibold text-slate-200">Registered Nodes List ({nodes.length})</h3>
          <span className="text-xs text-slate-500 font-mono">Real-time Data (Auto 5s)</span>
        </div>

        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs">
            <thead className="bg-slate-950/60 text-slate-400 font-mono uppercase text-[10px]">
              <tr>
                <th className="px-6 py-3">Status</th>
                <th className="px-6 py-3">Hostname / Node Name</th>
                <th className="px-6 py-3">IP Address</th>
                <th className="px-6 py-3">Group / Role</th>
                <th className="px-6 py-3">OS Version</th>
                <th className="px-6 py-3">Version</th>
                <th className="px-6 py-3">Last Heartbeat</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-800/60 text-slate-300">
              {nodes.length === 0 ? (
                <tr>
                  <td colSpan={7} className="px-6 py-8 text-center text-slate-500 font-mono">
                    Chưa có máy chủ nào được đăng ký trong hệ thống
                  </td>
                </tr>
              ) : (
                nodes.map((node) => (
                  <tr key={node.id} className="hover:bg-slate-800/30 transition-colors">
                    <td className="px-6 py-4">
                      <span
                        className={`inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-[11px] font-medium border ${
                          node.status === 'ONLINE'
                            ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
                            : 'bg-amber-500/10 text-amber-400 border-amber-500/20'
                        }`}
                      >
                        <span
                          className={`w-1.5 h-1.5 rounded-full ${
                            node.status === 'ONLINE' ? 'bg-emerald-400' : 'bg-amber-400'
                          }`}
                        />
                        {node.status}
                      </span>
                    </td>
                    <td className="px-6 py-4">
                      <div className="font-semibold text-slate-100">{node.hostname}</div>
                      <div className="font-mono text-[10px] text-slate-500">ID: {node.id}</div>
                    </td>
                    <td className="px-6 py-4 font-mono text-emerald-400 font-semibold">{node.ipAddress}</td>
                    <td className="px-6 py-4 font-mono text-slate-400">{node.group || 'default'}</td>
                    <td className="px-6 py-4 text-slate-400">{node.osVersion || 'Ubuntu 24.04 LTS'}</td>
                    <td className="px-6 py-4">
                      <span className="font-mono text-slate-300 bg-slate-800 px-2 py-0.5 rounded">
                        {node.policyVersion || 'v1.1.3'}
                      </span>
                    </td>
                    <td className="px-6 py-4 text-slate-400 font-mono">{node.lastHeartbeat}</td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  )
}
