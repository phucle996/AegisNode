// Page: Audit Logs - Bảng lịch sử vết hoạt động của AegisNode Agent
import { useState, useEffect } from 'react'
import { ScrollTextIcon, RefreshCwIcon } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { api, type AuditLogEntry } from '@/api/client'

export default function AuditLogsPage() {
  const [logs, setLogs] = useState<AuditLogEntry[]>([])
  const [loading, setLoading] = useState<boolean>(true)

  const fetchLogs = async () => {
    setLoading(true)
    try {
      const res = await api.getAuditLogs()
      setLogs(res)
    } catch {
      // Fallback sample audit trail
      setLogs([
        { id: 101, eventType: 'POLICY_APPLIED', actor: 'admin', details: 'Applied policy web-server-policy (hash: a3f9c2d1)', createdAt: '2026-08-02T08:00:00Z' },
        { id: 102, eventType: 'SNAPSHOT_CREATED', actor: 'system', details: 'Created snapshot pre-apply-101', createdAt: '2026-08-02T07:59:58Z' },
        { id: 103, eventType: 'IP_BLOCKED', actor: 'ssh_detector', details: 'Auto-blocked IP 198.51.100.45 for SSH brute-force', createdAt: '2026-08-02T07:45:12Z' },
        { id: 104, eventType: 'SYSCTL_FORWARDING_ENABLED', actor: 'admin', details: 'Router mode enabled ip_forward=1', createdAt: '2026-08-02T07:30:00Z' },
      ])
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    fetchLogs()
  }, [])

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-foreground flex items-center gap-2">
            <ScrollTextIcon className="h-6 w-6 text-primary" />
            Audit Trail & Logs
          </h1>
          <p className="text-muted-foreground text-sm mt-1">
            Lịch sử lưu vết mọi thao tác thay đổi Firewall, Policy Apply, Rollback và IP Block Events
          </p>
        </div>
        <Button variant="outline" size="sm" onClick={fetchLogs} disabled={loading} className="gap-1.5">
          <RefreshCwIcon className={`h-3.5 w-3.5 ${loading ? 'animate-spin' : ''}`} />
          Refresh
        </Button>
      </div>

      <Card className="glass-card">
        <CardHeader>
          <CardTitle className="text-sm">Agent Audit Events ({logs.length})</CardTitle>
        </CardHeader>
        <CardContent className="p-0">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead className="w-[60px]">ID</TableHead>
                <TableHead>Event Type</TableHead>
                <TableHead>Actor</TableHead>
                <TableHead>Details</TableHead>
                <TableHead className="text-right">Timestamp</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {logs.map((l) => (
                <TableRow key={l.id}>
                  <TableCell className="font-mono text-xs text-muted-foreground">#{l.id}</TableCell>
                  <TableCell>
                    <Badge variant="outline" className="font-mono text-xs">{l.eventType}</Badge>
                  </TableCell>
                  <TableCell className="text-xs font-semibold">{l.actor}</TableCell>
                  <TableCell className="text-xs text-muted-foreground">{l.details}</TableCell>
                  <TableCell className="text-right text-xs font-mono text-muted-foreground">{l.createdAt}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  )
}
