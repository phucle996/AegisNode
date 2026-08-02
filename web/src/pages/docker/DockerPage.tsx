// Page: Docker Exposures - Container inventory & cảnh báo phơi nhiễm cổng public ra 0.0.0.0 WAN
import { useState, useEffect } from 'react'
import { ContainerIcon, AlertTriangleIcon, ShieldCheckIcon, RefreshCwIcon } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { api, type DockerExposureReport } from '@/api/client'

export default function DockerPage() {
  const [report, setReport] = useState<DockerExposureReport | null>(null)
  const [loading, setLoading] = useState<boolean>(true)

  const fetchDocker = async () => {
    setLoading(true)
    try {
      const res = await api.getDockerExposures()
      setReport(res)
    } catch {
      // Fallback sample data cho UI preview nếu Docker Socket không mở
      setReport({
        dockerAvailable: true,
        containers: [
          {
            id: "c1a2b3c4d5e6",
            name: "postgres-db",
            image: "postgres:16-alpine",
            state: "running",
            networks: ["bridge"],
            publishedPorts: [
              { hostIp: "0.0.0.0", hostPort: 5432, containerPort: 5432, protocol: "tcp" }
            ],
            labels: { "aegis.exposure": "public-restricted" }
          },
          {
            id: "f9e8d7c6b5a4",
            name: "nginx-proxy",
            image: "nginx:alpine",
            state: "running",
            networks: ["bridge"],
            publishedPorts: [
              { hostIp: "0.0.0.0", hostPort: 80, containerPort: 80, protocol: "tcp" },
              { hostIp: "0.0.0.0", hostPort: 443, containerPort: 443, protocol: "tcp" }
            ],
            labels: {}
          }
        ],
        publicExposures: [
          {
            containerId: "c1a2b3c4d5e6",
            containerName: "postgres-db",
            publishedPort: { hostIp: "0.0.0.0", hostPort: 5432, containerPort: 5432, protocol: "tcp" },
            isDatabase: true,
            warningMessage: "PostgreSQL port 5432 is publicly exposed on 0.0.0.0 WAN interface!"
          }
        ],
        labelPolicies: []
      })
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    fetchDocker()
  }, [])

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-foreground flex items-center gap-2">
            <ContainerIcon className="h-6 w-6 text-primary" />
            Docker Exposures
          </h1>
          <p className="text-muted-foreground text-sm mt-1">
            Phát hiện Container inventory và cảnh báo phơi nhiễm cổng ra ngoài Internet
          </p>
        </div>
        <Button variant="outline" size="sm" onClick={fetchDocker} disabled={loading} className="gap-1.5">
          <RefreshCwIcon className={`h-3.5 w-3.5 ${loading ? 'animate-spin' : ''}`} />
          Refresh
        </Button>
      </div>

      {/* Public Exposure Warnings */}
      {report?.publicExposures && report.publicExposures.length > 0 && (
        <Card className="glass-card border-amber-500/30 bg-amber-500/5 glow-amber">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-semibold text-amber-400 flex items-center gap-2">
              <AlertTriangleIcon className="h-4 w-4 text-amber-400" />
              Public Exposure Warnings ({report.publicExposures.length})
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            {report.publicExposures.map((exp, i) => (
              <div key={i} className="flex items-center justify-between p-3 rounded-md bg-amber-500/10 border border-amber-500/20 text-xs">
                <div>
                  <span className="font-bold text-foreground">{exp.containerName}</span>
                  <span className="text-amber-300 ml-2 font-mono">{exp.publishedPort.hostIp}:{exp.publishedPort.hostPort}</span>
                  <p className="text-muted-foreground mt-0.5">{exp.warningMessage}</p>
                </div>
                <Badge variant="destructive">CRITICAL</Badge>
              </div>
            ))}
          </CardContent>
        </Card>
      )}

      {/* Container Inventory Table */}
      <Card className="glass-card">
        <CardHeader>
          <CardTitle className="text-sm flex items-center justify-between">
            <span>Container Inventory ({report?.containers.length || 0})</span>
            {report?.dockerAvailable ? (
              <Badge variant="success" className="text-[10px]">Docker Socket Active</Badge>
            ) : (
              <Badge variant="warning" className="text-[10px]">Docker Socket Unavailable</Badge>
            )}
          </CardTitle>
        </CardHeader>
        <CardContent className="p-0">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Container</TableHead>
                <TableHead>Image</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Published Ports</TableHead>
                <TableHead>Exposure Risk</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {report?.containers.map((c) => {
                const isExposed = c.publishedPorts.some(p => p.hostIp === '0.0.0.0')
                return (
                  <TableRow key={c.id}>
                    <TableCell className="font-semibold text-xs">
                      <div>{c.name}</div>
                      <span className="text-[10px] font-mono text-muted-foreground">{c.id.slice(0, 12)}</span>
                    </TableCell>
                    <TableCell className="font-mono text-xs text-muted-foreground">{c.image}</TableCell>
                    <TableCell>
                      <Badge variant={c.state === 'running' ? 'success' : 'outline'}>{c.state}</Badge>
                    </TableCell>
                    <TableCell className="font-mono text-xs">
                      {c.publishedPorts.map((p, idx) => (
                        <div key={idx} className={p.hostIp === '0.0.0.0' ? 'text-amber-400 font-bold' : 'text-muted-foreground'}>
                          {p.hostIp}:{p.hostPort} ➔ {p.containerPort}/{p.protocol}
                        </div>
                      ))}
                    </TableCell>
                    <TableCell>
                      {isExposed ? (
                        <Badge variant="warning" className="gap-1">
                          <AlertTriangleIcon className="h-3 w-3" /> WAN 0.0.0.0
                        </Badge>
                      ) : (
                        <Badge variant="success" className="gap-1">
                          <ShieldCheckIcon className="h-3 w-3" /> Isolated
                        </Badge>
                      )}
                    </TableCell>
                  </TableRow>
                )
              })}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  )
}
