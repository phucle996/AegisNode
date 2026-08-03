// Page: Docker Exposures - Hiển thị Container Inventory và Cảnh báo Phơi nhiễm cổng public ra 0.0.0.0 WAN thực tế từ OS
import { useState, useEffect } from 'react'
import { ContainerIcon, AlertTriangleIcon, ShieldCheckIcon, RefreshCwIcon, InfoIcon } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { api, type DockerExposureReport } from '@/api/client'

export default function DockerPage() {
  // State chứa kết quả báo cáo thực tế từ REST API /v1/docker/exposure
  const [report, setReport] = useState<DockerExposureReport | null>(null)
  const [loading, setLoading] = useState<boolean>(true)

  // Hàm gọi REST API từ Controller Backend (Loại bỏ 100% Mock Data)
  const fetchDocker = async () => {
    setLoading(true)
    try {
      const res = await api.getDockerExposures()
      setReport(res)
    } catch (error) {
      console.error('Lỗi lấy báo cáo Docker Exposures:', error)
      // Khi không có kết nối hoặc Docker chưa cài đặt, trả về đối tượng khả dụng = false (Không nạp dữ liệu giả)
      setReport({
        dockerAvailable: false,
        containers: [],
        publicExposures: [],
        labelPolicies: [],
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
      {/* Header Phân hệ Docker Exposures */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-foreground flex items-center gap-2">
            <ContainerIcon className="h-6 w-6 text-primary" />
            Docker Exposures (Real OS Engine Telemetry)
          </h1>
          <p className="text-muted-foreground text-sm mt-1">
            Phát hiện Container inventory và cảnh báo phơi nhiễm cổng ra ngoài Internet thời gian thực
          </p>
        </div>
        <Button variant="outline" size="sm" onClick={fetchDocker} disabled={loading} className="gap-1.5">
          <RefreshCwIcon className={`h-3.5 w-3.5 ${loading ? 'animate-spin' : ''}`} />
          Refresh
        </Button>
      </div>

      {/* Thông báo nếu Docker Engine chưa cài đặt hoặc chưa khởi chạy */}
      {report && !report.dockerAvailable && (
        <Card className="glass-card border-slate-800 bg-slate-900/60">
          <CardContent className="p-6 text-center space-y-3">
            <div className="inline-flex p-3 rounded-full bg-slate-800 text-slate-400">
              <InfoIcon className="w-6 h-6" />
            </div>
            <h3 className="text-base font-bold text-slate-200">Docker Engine Unavailable / Not Installed</h3>
            <p className="text-xs text-slate-400 max-w-md mx-auto">
              Không tìm thấy Docker Socket (`/var/run/docker.sock`). Máy chủ này chưa được cài đặt Docker Engine hoặc dịch vụ Docker daemon hiện tại không hoạt động.
            </p>
          </CardContent>
        </Card>
      )}

      {/* Cảnh báo phơi nhiễm cổng public 0.0.0.0 WAN thực tế */}
      {report?.dockerAvailable && report.publicExposures && report.publicExposures.length > 0 && (
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

      {/* Danh sách Container Inventory Thực tế */}
      {report?.dockerAvailable && (
        <Card className="glass-card">
          <CardHeader>
            <CardTitle className="text-sm flex items-center justify-between">
              <span>Container Inventory ({report.containers.length})</span>
              <Badge variant="success" className="text-[10px]">
                Docker Engine Active
              </Badge>
            </CardTitle>
          </CardHeader>
          <CardContent className="p-0">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Container Name</TableHead>
                  <TableHead>Image</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>CPU Usage</TableHead>
                  <TableHead>Memory Usage</TableHead>
                  <TableHead>Published Ports</TableHead>
                  <TableHead>Exposure Risk</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {report.containers.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={7} className="text-center py-8 text-xs text-muted-foreground font-mono">
                      Docker Engine đang chạy nhưng chưa có Container nào được khởi tạo.
                    </TableCell>
                  </TableRow>
                ) : (
                  report.containers.map((c) => {
                    const isExposed = c.publishedPorts.some((p) => p.hostIp === '0.0.0.0' || p.hostIp === '::')
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
                        {/* 1. Cột chỉ số % CPU tiêu thụ thực tế từ docker stats */}
                        <TableCell className="font-mono text-xs font-semibold text-emerald-400">
                          {c.cpuPerc || '0.00%'}
                        </TableCell>
                        {/* 2. Cột chỉ số RAM tiêu thụ (Usage / Limit) thực tế từ docker stats */}
                        <TableCell className="font-mono text-xs text-cyan-300">
                          {c.memUsage || '0.00 B / 0.00 B'}
                        </TableCell>
                        <TableCell className="font-mono text-xs">
                          {c.publishedPorts.length === 0 ? (
                            <span className="text-slate-500">No published ports</span>
                          ) : (
                            c.publishedPorts.map((p, idx) => (
                              <div
                                key={idx}
                                className={p.hostIp === '0.0.0.0' || p.hostIp === '::' ? 'text-amber-400 font-bold' : 'text-muted-foreground'}
                              >
                                {p.hostIp}:{p.hostPort} ➔ {p.containerPort}/{p.protocol}
                              </div>
                            ))
                          )}
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
                  })
                )}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
