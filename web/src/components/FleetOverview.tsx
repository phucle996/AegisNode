// Fleet Overview Dashboard Component (Phase 19 Central Web UI)
// Hiển thị tổng quan trạng thái các Node trong Fleet, metrics hệ thống và bộ lọc Node

import React, { useEffect, useState } from 'react'
import { fleetApi } from '../api/fleetClient'
import { FleetNode, SystemMetrics } from '../types/fleet'

export const FleetOverview: React.FC = () => {
  // State quản lý danh sách Node trong Fleet
  const [nodes, setNodes] = useState<FleetNode[]>([])
  // State quản lý các chỉ số giám sát Prometheus Metrics
  const [metrics, setMetrics] = useState<SystemMetrics | null>(null)
  // State lưu trạng thái loading dữ liệu
  const [loading, setLoading] = useState<boolean>(true)

  // Fetch dữ liệu từ Controller API khi Component được mount
  useEffect(() => {
    const fetchData = async () => {
      setLoading(true)
      const fetchedNodes = await fleetApi.getFleetNodes()
      const fetchedMetrics = await fleetApi.getMetrics()
      setNodes(fetchedNodes)
      setMetrics(fetchedMetrics)
      setLoading(false)
    }

    fetchData()
    // Thiết lập tự động refresh dữ liệu định kỳ mỗi 5 giây
    const interval = setInterval(fetchData, 5000)
    return () => clearInterval(interval)
  }, [])

  if (loading && !nodes.length) {
    return <div className="p-6 text-slate-400">Đang tải dữ liệu Fleet...</div>
  }

  // Tính toán số lượng Node theo trạng thái
  const onlineCount = nodes.filter((n) => n.status === 'ONLINE').length
  const degradedCount = nodes.filter((n) => n.status === 'DEGRADED').length

  return (
    <div className="space-y-6">
      {/* 1. Header & Summary Cards */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-bold text-slate-100">Fleet Overview & Nodes Status</h2>
          <p className="text-sm text-slate-400">Quản lý tập trung danh sách máy chủ AegisNode Agents trong hệ thống</p>
        </div>
        <span className="px-3 py-1 bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 text-xs font-mono rounded-full">
          ● Central Controller Active
        </span>
      </div>

      {/* Grid chứa 4 thẻ Thống kê Metrics */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="bg-slate-900/60 border border-slate-800 rounded-xl p-4">
          <div className="text-xs font-medium text-slate-400">Total Fleet Nodes</div>
          <div className="text-2xl font-bold text-slate-100 mt-1">{nodes.length} Nodes</div>
          <div className="text-xs text-emerald-400 mt-2 font-mono">
            {onlineCount} Online | {degradedCount} Degraded
          </div>
        </div>

        <div className="bg-slate-900/60 border border-slate-800 rounded-xl p-4">
          <div className="text-xs font-medium text-slate-400">Connected mTLS Agents</div>
          <div className="text-2xl font-bold text-sky-400 mt-1">{metrics?.connectedAgents || 0} Connected</div>
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

      {/* 2. Bảng Danh sách Nodes trong Fleet */}
      <div className="bg-slate-900/60 border border-slate-800 rounded-xl overflow-hidden">
        <div className="px-6 py-4 border-b border-slate-800 flex items-center justify-between">
          <h3 className="text-sm font-semibold text-slate-200">Registered Nodes ({nodes.length})</h3>
          <span className="text-xs text-slate-500">Auto-refreshed every 5s</span>
        </div>

        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs">
            <thead className="bg-slate-950/60 text-slate-400 font-mono uppercase text-[10px]">
              <tr>
                <th className="px-6 py-3">Status</th>
                <th className="px-6 py-3">Hostname / IP</th>
                <th className="px-6 py-3">Group</th>
                <th className="px-6 py-3">OS Version</th>
                <th className="px-6 py-3">Policy Version</th>
                <th className="px-6 py-3">Last Heartbeat</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-800/60 text-slate-300">
              {nodes.map((node) => (
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
                    <div className="font-mono text-[11px] text-slate-400">{node.ipAddress}</div>
                  </td>
                  <td className="px-6 py-4 font-mono text-slate-400">{node.group}</td>
                  <td className="px-6 py-4 text-slate-400">{node.osVersion}</td>
                  <td className="px-6 py-4">
                    <span className="font-mono text-slate-300 bg-slate-800 px-2 py-0.5 rounded">
                      {node.policyVersion}
                    </span>
                  </td>
                  <td className="px-6 py-4 text-slate-400">{node.lastHeartbeat}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  )
}
