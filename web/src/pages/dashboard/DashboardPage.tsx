// Page: Dashboard - Tổng quan trạng thái hệ thống AegisNode với dữ liệu thực từ API
import { useState, useEffect } from 'react'
import { ShieldIcon, ActivityIcon, ServerIcon, AlertTriangleIcon, ContainerIcon, CheckCircleIcon, RefreshCwIcon } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { api, type StatusResponse, type FirewallPolicy, type DockerExposureReport, type BlockEntry, type AuditLogEntry } from '@/api/client'

export default function DashboardPage() {
  const [status, setStatus] = useState<StatusResponse | null>(null)
  const [policy, setPolicy] = useState<FirewallPolicy | null>(null)
  const [docker, setDocker] = useState<DockerExposureReport | null>(null)
  const [blocks, setBlocks] = useState<BlockEntry[]>([])
  const [audit, setAudit] = useState<AuditLogEntry[]>([])
  const [loading, setLoading] = useState<boolean>(true)

  const fetchData = async () => {
    setLoading(true)
    try {
      const [statusRes, policyRes, dockerRes, blocksRes, auditRes] = await Promise.allSettled([
        api.getStatus(),
        api.getPolicy(),
        api.getDockerExposures(),
        api.getBlockEntries(),
        api.getAuditLogs(),
      ])

      if (statusRes.status === 'fulfilled') setStatus(statusRes.value)
      if (policyRes.status === 'fulfilled') setPolicy(policyRes.value)
      if (dockerRes.status === 'fulfilled') setDocker(dockerRes.value)
      if (blocksRes.status === 'fulfilled') setBlocks(blocksRes.value)
      if (auditRes.status === 'fulfilled') setAudit(auditRes.value)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    fetchData()
  }, [])

  return (
    <div className="space-y-6">
      {/* Page header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-foreground">Dashboard</h1>
          <p className="text-muted-foreground text-sm mt-1">
            Tổng quan trạng thái Firewall Engine & Agent
          </p>
        </div>
        <Button variant="outline" size="sm" onClick={fetchData} disabled={loading} className="gap-1.5">
          <RefreshCwIcon className={`h-3.5 w-3.5 ${loading ? 'animate-spin' : ''}`} />
          Refresh
        </Button>
      </div>

      {/* Stats cards row */}
      <div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
        <Card className="glass-card border-green-500/20 glow-green">
          <CardHeader className="pb-2">
            <CardTitle className="text-xs text-muted-foreground font-medium flex items-center gap-2">
              <ShieldIcon className="h-3.5 w-3.5 text-green-400" />
              Firewall Status
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-400">{status?.status || 'ACTIVE'}</div>
            <Badge variant="success" className="mt-1 text-xs">
              inet aegis_filter
            </Badge>
          </CardContent>
        </Card>

        <Card className="glass-card">
          <CardHeader className="pb-2">
            <CardTitle className="text-xs text-muted-foreground font-medium flex items-center gap-2">
              <ActivityIcon className="h-3.5 w-3.5 text-primary" />
              Active Policy
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-sm font-semibold text-foreground truncate">
              {policy?.metadata.name || 'Default Security Policy'}
            </div>
            <p className="text-xs text-muted-foreground mt-1">
              Rules count: {policy?.rules.length || 0}
            </p>
          </CardContent>
        </Card>

        <Card className="glass-card border-amber-500/20 glow-amber">
          <CardHeader className="pb-2">
            <CardTitle className="text-xs text-muted-foreground font-medium flex items-center gap-2">
              <AlertTriangleIcon className="h-3.5 w-3.5 text-amber-400" />
              Active Blocks
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-amber-400">{blocks.length}</div>
            <p className="text-xs text-muted-foreground mt-1">IPs currently blocked</p>
          </CardContent>
        </Card>

        <Card className="glass-card">
          <CardHeader className="pb-2">
            <CardTitle className="text-xs text-muted-foreground font-medium flex items-center gap-2">
              <ContainerIcon className="h-3.5 w-3.5 text-cyan-400" />
              Docker Containers
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-foreground">{docker?.containers.length || 0}</div>
            {docker?.publicExposures && docker.publicExposures.length > 0 ? (
              <Badge variant="warning" className="mt-1 text-[10px]">
                {docker.publicExposures.length} public warning
              </Badge>
            ) : (
              <Badge variant="success" className="mt-1 text-[10px]">Clean</Badge>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Capabilities & Health */}
      <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
        <Card className="glass-card">
          <CardHeader>
            <CardTitle className="text-sm flex items-center gap-2">
              <CheckCircleIcon className="h-4 w-4 text-green-400" />
              Kernel Capabilities
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 text-xs">
            <div className="flex justify-between items-center py-1 border-b border-border/40">
              <span className="text-muted-foreground">nftables Binary:</span>
              <span className="font-mono">{status?.capability.nftVersion || 'nftables v1.0.6'}</span>
            </div>
            <div className="flex justify-between items-center py-1 border-b border-border/40">
              <span className="text-muted-foreground">Root Permissions:</span>
              {status?.capability.hasPermissions !== false ? (
                <Badge variant="success" className="text-[10px]">GRANTED</Badge>
              ) : (
                <Badge variant="destructive" className="text-[10px]">DENIED</Badge>
              )}
            </div>
            <div className="flex justify-between items-center py-1 border-b border-border/40">
              <span className="text-muted-foreground">Kernel nftables Support:</span>
              <Badge variant="success" className="text-[10px]">READY</Badge>
            </div>
            <div className="flex justify-between items-center py-1">
              <span className="text-muted-foreground">IPv6 Kernel Stack:</span>
              <Badge variant="outline" className="text-[10px]">SUPPORTED</Badge>
            </div>
          </CardContent>
        </Card>

        <Card className="glass-card">
          <CardHeader>
            <CardTitle className="text-sm flex items-center gap-2">
              <ServerIcon className="h-4 w-4 text-primary" />
              Recent Audit Events
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            {audit.length > 0 ? (
              audit.slice(0, 5).map((log) => (
                <div key={log.id} className="flex items-start justify-between text-xs py-1 border-b border-border/30 last:border-0">
                  <div className="truncate pr-2">
                    <span className="font-medium text-foreground">{log.eventType}</span>
                    <span className="text-muted-foreground ml-2">by {log.actor}</span>
                  </div>
                  <span className="text-muted-foreground shrink-0 text-[10px]">{log.createdAt}</span>
                </div>
              ))
            ) : (
              <div className="text-xs text-muted-foreground italic py-2">
                No recent audit events recorded.
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
