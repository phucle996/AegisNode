import { useEffect, useState } from 'react'
import { ShieldIcon, ArrowDownIcon, ArrowUpIcon, ArrowRightIcon, ServerIcon, RefreshCwIcon } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent } from '@/components/ui/card'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { firewallApi, LiveFirewallRule } from '../../api/firewallClient'
import { nodesApi } from '../../api/nodesClient'
import { FleetNode } from '../../types/fleet'

const ActionBadge = ({ action }: { action: string }) => {
  const act = action.toUpperCase()
  if (act === 'ACCEPT') return <Badge variant="success">{act}</Badge>
  if (act === 'DROP') return <Badge variant="destructive">{act}</Badge>
  return <Badge variant="warning">{act}</Badge>
}

const LiveRulesTable = ({ rules }: { rules: LiveFirewallRule[] }) => (
  <Table>
    <TableHeader>
      <TableRow>
        <TableHead className="w-[120px]">Rule ID</TableHead>
        <TableHead>Protocol</TableHead>
        <TableHead>Source IP</TableHead>
        <TableHead>Destination IP</TableHead>
        <TableHead>Port / Spec</TableHead>
        <TableHead>Action</TableHead>
        <TableHead>Kernel Packets</TableHead>
        <TableHead>Kernel Traffic</TableHead>
      </TableRow>
    </TableHeader>
    <TableBody>
      {rules.length === 0 ? (
        <TableRow>
          <TableCell colSpan={8} className="text-center py-8 text-muted-foreground text-xs font-mono">
            Chưa có bản ghi luật Kernel nftables nào được ghi nhận.
          </TableCell>
        </TableRow>
      ) : (
        rules.map((rule) => (
          <TableRow key={rule.id}>
            <TableCell className="font-mono text-xs text-muted-foreground">{rule.ruleId || rule.id.substring(0, 10)}</TableCell>
            <TableCell>
              <Badge variant="outline" className="text-xs font-mono">
                {rule.protocol}
              </Badge>
            </TableCell>
            <TableCell className="font-mono text-xs">{rule.srcCidr}</TableCell>
            <TableCell className="font-mono text-xs">{rule.dstCidr}</TableCell>
            <TableCell className="font-mono text-xs">{rule.portSpec}</TableCell>
            <TableCell>
              <ActionBadge action={rule.action} />
            </TableCell>
            <TableCell className="font-mono text-xs text-emerald-400 font-semibold">
              {rule.packets.toLocaleString()} pkts
            </TableCell>
            <TableCell className="font-mono text-xs text-slate-400">
              {(rule.bytes / 1024).toFixed(1)} KB
            </TableCell>
          </TableRow>
        ))
      )}
    </TableBody>
  </Table>
)

export default function FirewallRulesPage() {
  // State danh sách các máy chủ Nodes thực tế từ PostgreSQL
  const [nodes, setNodes] = useState<FleetNode[]>([])
  // State NodeID đang chọn để lọc rules (Rỗng = All Nodes)
  const [selectedNodeId, setSelectedNodeId] = useState<string>('')
  // State danh sách luật Firewall thực tế từ OS Kernel
  const [rules, setRules] = useState<LiveFirewallRule[]>([])
  // State loading
  const [loading, setLoading] = useState<boolean>(false)

  // Fetch danh sách Nodes thực tế
  const loadNodes = async () => {
    const data = await nodesApi.getNodes()
    setNodes(data)
  }

  // Fetch danh sách Live Rules từ Controller REST API
  const loadLiveRules = async () => {
    setLoading(true)
    try {
      const data = await firewallApi.getLiveRules(selectedNodeId || undefined)
      setRules(data)
    } catch (error) {
      console.error('Lỗi lấy live rules:', error)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadNodes()
  }, [])

  useEffect(() => {
    loadLiveRules()
    // Tự động Polling 3s cập nhật bộ đếm gói tin (packet counters) thời gian thực từ OS Kernel
    const timer = setInterval(loadLiveRules, 3000)
    return () => clearInterval(timer)
  }, [selectedNodeId])

  // Phân loại rules theo Chain: INPUT (Inbound), OUTPUT (Outbound), FORWARD (Forward)
  const inboundRules = rules.filter((r) => r.chain === 'INPUT' || r.chain === 'INBOUND')
  const outboundRules = rules.filter((r) => r.chain === 'OUTPUT' || r.chain === 'OUTBOUND')
  const forwardRules = rules.filter((r) => r.chain === 'FORWARD')

  return (
    <div className="space-y-6">
      {/* Header & Chọn máy chủ Node */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold text-foreground flex items-center gap-2">
            <ShieldIcon className="h-6 w-6 text-primary" />
            Firewall Rules (Real OS Kernel Telemetry)
          </h1>
          <p className="text-muted-foreground text-sm mt-1">
            Quy tắc Tường lửa `nftables` thực tế đồng bộ tự động từ Kernel OS Linux và PostgreSQL CSDL
          </p>
        </div>

        {/* Dropdown bộ lọc chọn máy chủ Linux Node */}
        <div className="flex items-center gap-2">
          <ServerIcon className="w-4 h-4 text-slate-400" />
          <select
            value={selectedNodeId}
            onChange={(e) => setSelectedNodeId(e.target.value)}
            className="bg-slate-900 border border-slate-800 rounded-lg px-3 py-2 text-xs font-mono text-slate-200 focus:outline-none focus:border-indigo-500"
          >
            <option value="">TẤT CẢ MÁY CHỦ (ALL FLEET NODES)</option>
            {nodes.map((node) => (
              <option key={node.id} value={node.id}>
                {node.hostname} ({node.ipAddress})
              </option>
            ))}
          </select>

          <button
            onClick={loadLiveRules}
            disabled={loading}
            className="p-2 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded-lg transition-colors"
          >
            <RefreshCwIcon className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
          </button>
        </div>
      </div>

      {/* Chú thích các luồng lưu lượng Direction summary */}
      <div className="flex flex-wrap gap-4 bg-slate-900/60 border border-slate-800 p-4 rounded-xl">
        <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
          <ArrowDownIcon className="h-4 w-4 text-cyan-400" />
          <span className="text-cyan-400 font-bold font-mono">INBOUND (INPUT)</span>
          <span>— Gói tin đi vào máy chủ Kernel Linux</span>
        </div>
        <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
          <ArrowUpIcon className="h-4 w-4 text-emerald-400" />
          <span className="text-emerald-400 font-bold font-mono">OUTBOUND (OUTPUT)</span>
          <span>— Gói tin đi ra từ máy chủ</span>
        </div>
        <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
          <ArrowRightIcon className="h-4 w-4 text-amber-400" />
          <span className="text-amber-400 font-bold font-mono">FORWARD</span>
          <span>— Gói tin định tuyến qua máy chủ (Docker/Router)</span>
        </div>
      </div>

      <Card className="glass-card">
        <CardContent className="p-0">
          <Tabs defaultValue="inbound">
            <div className="px-6 pt-4">
              <TabsList>
                <TabsTrigger value="inbound" className="gap-1.5">
                  <ArrowDownIcon className="h-3.5 w-3.5 text-cyan-400" />
                  Inbound Rules ({inboundRules.length})
                </TabsTrigger>
                <TabsTrigger value="outbound" className="gap-1.5">
                  <ArrowUpIcon className="h-3.5 w-3.5 text-emerald-400" />
                  Outbound Rules ({outboundRules.length})
                </TabsTrigger>
                <TabsTrigger value="forward" className="gap-1.5">
                  <ArrowRightIcon className="h-3.5 w-3.5 text-amber-400" />
                  Forwarding Rules ({forwardRules.length})
                </TabsTrigger>
              </TabsList>
            </div>
            <TabsContent value="inbound" className="mt-0">
              <LiveRulesTable rules={inboundRules} />
            </TabsContent>
            <TabsContent value="outbound" className="mt-0">
              <LiveRulesTable rules={outboundRules} />
            </TabsContent>
            <TabsContent value="forward" className="mt-0">
              <LiveRulesTable rules={forwardRules} />
            </TabsContent>
          </Tabs>
        </CardContent>
      </Card>
    </div>
  )
}
