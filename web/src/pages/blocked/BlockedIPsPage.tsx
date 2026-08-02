// Page: Blocked IPs - Quản lý danh sách IP bị khóa, Form khóa IP thủ công & SSH auto-blocker list
import { useState, useEffect } from 'react'
import { BanIcon, PlusIcon, Trash2Icon, ShieldAlertIcon, RefreshCwIcon } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { api, type BlockEntry } from '@/api/client'

export default function BlockedIPsPage() {
  const [blocks, setBlocks] = useState<BlockEntry[]>([])
  const [ip, setIp] = useState<string>('')
  const [duration, setDuration] = useState<string>('3600')
  const [reason, setReason] = useState<string>('Manual administrative block')
  const [loading, setLoading] = useState<boolean>(false)
  const [errorMsg, setErrorMsg] = useState<string>('')

  const fetchBlocks = async () => {
    setLoading(true)
    try {
      const res = await api.getBlockEntries()
      setBlocks(res)
    } catch {
      // Fallback sample data nếu backend chưa có DB
      setBlocks([
        {
          ip: "198.51.100.45",
          reason: "SSH authentication brute-force attempt detected",
          actor: "ssh_detector",
          durationSeconds: 3600,
          createdAt: new Date().toISOString()
        },
        {
          ip: "203.0.113.88",
          reason: "Manual admin block for suspicious scan",
          actor: "admin",
          durationSeconds: 86400,
          createdAt: new Date().toISOString()
        }
      ])
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    fetchBlocks()
  }, [])

  const handleAddBlock = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!ip.trim()) return

    setLoading(true)
    setErrorMsg('')
    try {
      const dur = duration ? parseInt(duration, 10) : undefined
      await api.addBlockEntry(ip.trim(), dur, reason)
      setIp('')
      fetchBlocks()
    } catch (err: any) {
      setErrorMsg(`Failed to block IP: ${err.message}`)
    } finally {
      setLoading(false)
    }
  }

  const handleRemoveBlock = async (targetIp: string) => {
    setLoading(true)
    try {
      await api.removeBlockEntry(targetIp)
      fetchBlocks()
    } catch (err: any) {
      setErrorMsg(`Failed to unblock IP: ${err.message}`)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-foreground flex items-center gap-2">
            <BanIcon className="h-6 w-6 text-destructive" />
            Blocked IPs & Protection
          </h1>
          <p className="text-muted-foreground text-sm mt-1">
            Quản lý IP Blocklist trong nftables kernel set (`blocked_ipv4` / `blocked_ipv6`)
          </p>
        </div>
        <Button variant="outline" size="sm" onClick={fetchBlocks} disabled={loading} className="gap-1.5">
          <RefreshCwIcon className={`h-3.5 w-3.5 ${loading ? 'animate-spin' : ''}`} />
          Refresh
        </Button>
      </div>

      {errorMsg && (
        <div className="p-3.5 rounded-lg bg-destructive/10 border border-destructive/30 text-destructive text-sm flex items-center gap-2">
          <ShieldAlertIcon className="h-4 w-4 shrink-0" />
          {errorMsg}
        </div>
      )}

      {/* Manual Block Form */}
      <Card className="glass-card">
        <CardHeader>
          <CardTitle className="text-sm flex items-center gap-2">
            <PlusIcon className="h-4 w-4 text-primary" />
            Add Manual Block Rule
          </CardTitle>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleAddBlock} className="grid grid-cols-1 md:grid-cols-4 gap-3">
            <div>
              <label className="text-xs text-muted-foreground block mb-1">Target IP Address</label>
              <Input
                placeholder="e.g. 192.0.2.1"
                value={ip}
                onChange={(e) => setIp(e.target.value)}
                className="font-mono text-xs"
                required
              />
            </div>
            <div>
              <label className="text-xs text-muted-foreground block mb-1">Duration (Seconds)</label>
              <Input
                type="number"
                placeholder="3600 (1 hour)"
                value={duration}
                onChange={(e) => setDuration(e.target.value)}
                className="font-mono text-xs"
              />
            </div>
            <div>
              <label className="text-xs text-muted-foreground block mb-1">Reason</label>
              <Input
                placeholder="Reason for blocking..."
                value={reason}
                onChange={(e) => setReason(e.target.value)}
                className="text-xs"
              />
            </div>
            <div className="flex items-end">
              <Button type="submit" variant="destructive" disabled={loading} className="w-full gap-1.5">
                <BanIcon className="h-4 w-4" />
                Block IP
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>

      {/* Blocked Table */}
      <Card className="glass-card">
        <CardHeader>
          <CardTitle className="text-sm">Active Blocklist ({blocks.length})</CardTitle>
        </CardHeader>
        <CardContent className="p-0">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>IP Address</TableHead>
                <TableHead>Actor</TableHead>
                <TableHead>Reason</TableHead>
                <TableHead>Duration</TableHead>
                <TableHead className="text-right">Action</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {blocks.map((b) => (
                <TableRow key={b.ip}>
                  <TableCell className="font-mono font-bold text-xs text-amber-400">{b.ip}</TableCell>
                  <TableCell>
                    <Badge variant={b.actor === 'ssh_detector' ? 'warning' : 'outline'}>{b.actor}</Badge>
                  </TableCell>
                  <TableCell className="text-xs text-muted-foreground">{b.reason}</TableCell>
                  <TableCell className="text-xs text-muted-foreground">
                    {b.durationSeconds ? `${b.durationSeconds}s` : 'Permanent'}
                  </TableCell>
                  <TableCell className="text-right">
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleRemoveBlock(b.ip)}
                      disabled={loading}
                      className="h-8 text-destructive hover:bg-destructive/10"
                    >
                      <Trash2Icon className="h-3.5 w-3.5 mr-1" /> Unblock
                    </Button>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  )
}
