// Page: Dashboard - Tổng quan trạng thái hệ thống AegisNode
import { ShieldIcon, ActivityIcon, ServerIcon, AlertTriangleIcon, ContainerIcon, CheckCircleIcon } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'

export default function DashboardPage() {
  return (
    <div className="space-y-6">
      {/* Page header */}
      <div>
        <h1 className="text-2xl font-bold text-foreground">Dashboard</h1>
        <p className="text-muted-foreground text-sm mt-1">
          Tổng quan trạng thái Firewall & Agent
        </p>
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
            <div className="text-2xl font-bold text-green-400">ACTIVE</div>
            <Badge variant="success" className="mt-1 text-xs">
              inet aegis_filter
            </Badge>
          </CardContent>
        </Card>

        <Card className="glass-card">
          <CardHeader className="pb-2">
            <CardTitle className="text-xs text-muted-foreground font-medium flex items-center gap-2">
              <ActivityIcon className="h-3.5 w-3.5 text-primary" />
              Policy Hash
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-sm font-mono text-primary truncate">a3f9c2d1...</div>
            <p className="text-xs text-muted-foreground mt-1">Last applied: 2m ago</p>
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
            <div className="text-2xl font-bold text-amber-400">3</div>
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
            <div className="text-2xl font-bold text-foreground">5</div>
            <Badge variant="warning" className="mt-1 text-xs">2 public ports</Badge>
          </CardContent>
        </Card>
      </div>

      {/* System Health */}
      <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
        <Card className="glass-card">
          <CardHeader>
            <CardTitle className="text-sm flex items-center gap-2">
              <CheckCircleIcon className="h-4 w-4 text-green-400" />
              System Health
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            {['Managed table active', 'Loopback reachable', 'No pending execution'].map((item) => (
              <div key={item} className="flex items-center gap-2 text-sm">
                <div className="h-1.5 w-1.5 rounded-full bg-green-400" />
                <span className="text-muted-foreground">{item}</span>
              </div>
            ))}
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
            {[
              { action: 'Policy applied', time: '2m ago', type: 'info' },
              { action: 'IP 1.2.3.4 blocked (SSH brute-force)', time: '5m ago', type: 'warn' },
              { action: 'Snapshot created', time: '2m ago', type: 'info' },
            ].map((event, i) => (
              <div key={i} className="flex items-start justify-between text-xs">
                <span className={event.type === 'warn' ? 'text-amber-400' : 'text-muted-foreground'}>
                  {event.action}
                </span>
                <span className="text-muted-foreground ml-2 shrink-0">{event.time}</span>
              </div>
            ))}
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
