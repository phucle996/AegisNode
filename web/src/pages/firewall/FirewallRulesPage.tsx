// Page: Firewall Rules - Hiển thị danh sách rules phân loại Inbound/Outbound/Forward
import { ShieldIcon, ArrowDownIcon, ArrowUpIcon, ArrowRightIcon } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent } from '@/components/ui/card'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'

// Mockup data phân loại Inbound / Outbound / Forward rules
const MOCK_RULES = {
  inbound: [
    { id: 'r1', proto: 'TCP', src: '0.0.0.0/0', dst: 'any', port: '22', action: 'ACCEPT', counter: '1,203 pkts' },
    { id: 'r2', proto: 'TCP', src: '0.0.0.0/0', dst: 'any', port: '80,443', action: 'ACCEPT', counter: '45,610 pkts' },
    { id: 'r3', proto: 'ANY', src: '0.0.0.0/0', dst: 'any', port: 'any', action: 'DROP', counter: '894 pkts' },
  ],
  outbound: [
    { id: 'r4', proto: 'TCP', src: 'any', dst: '0.0.0.0/0', port: '443', action: 'ACCEPT', counter: '22,100 pkts' },
    { id: 'r5', proto: 'UDP', src: 'any', dst: '0.0.0.0/0', port: '53', action: 'ACCEPT', counter: '5,432 pkts' },
  ],
  forward: [
    { id: 'r6', proto: 'TCP', src: '10.0.0.0/8', dst: '192.168.1.0/24', port: '8080', action: 'ACCEPT', counter: '1,045 pkts' },
  ],
}

const ActionBadge = ({ action }: { action: string }) => {
  if (action === 'ACCEPT') return <Badge variant="success">{action}</Badge>
  if (action === 'DROP') return <Badge variant="destructive">{action}</Badge>
  return <Badge variant="warning">{action}</Badge>
}

const RulesTable = ({ rules }: { rules: typeof MOCK_RULES.inbound }) => (
  <Table>
    <TableHeader>
      <TableRow>
        <TableHead className="w-[80px]">Rule ID</TableHead>
        <TableHead>Protocol</TableHead>
        <TableHead>Source</TableHead>
        <TableHead>Destination</TableHead>
        <TableHead>Port</TableHead>
        <TableHead>Action</TableHead>
        <TableHead>Traffic</TableHead>
      </TableRow>
    </TableHeader>
    <TableBody>
      {rules.map((rule) => (
        <TableRow key={rule.id}>
          <TableCell className="font-mono text-xs text-muted-foreground">{rule.id}</TableCell>
          <TableCell>
            <Badge variant="outline" className="text-xs font-mono">{rule.proto}</Badge>
          </TableCell>
          <TableCell className="font-mono text-xs">{rule.src}</TableCell>
          <TableCell className="font-mono text-xs">{rule.dst}</TableCell>
          <TableCell className="font-mono text-xs">{rule.port}</TableCell>
          <TableCell><ActionBadge action={rule.action} /></TableCell>
          <TableCell className="text-xs text-muted-foreground">{rule.counter}</TableCell>
        </TableRow>
      ))}
    </TableBody>
  </Table>
)

export default function FirewallRulesPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-foreground flex items-center gap-2">
          <ShieldIcon className="h-6 w-6 text-primary" />
          Firewall Rules
        </h1>
        <p className="text-muted-foreground text-sm mt-1">
          Danh sách rules phân loại Inbound / Outbound / Forward với bộ đếm lưu lượng
        </p>
      </div>

      {/* Direction summary */}
      <div className="flex gap-3">
        <div className="flex items-center gap-1.5 text-sm text-muted-foreground">
          <ArrowDownIcon className="h-4 w-4 text-cyan-400" />
          <span className="text-cyan-400 font-medium">INBOUND</span>
          <span>— packets received into host</span>
        </div>
        <div className="flex items-center gap-1.5 text-sm text-muted-foreground">
          <ArrowUpIcon className="h-4 w-4 text-emerald-400" />
          <span className="text-emerald-400 font-medium">OUTBOUND</span>
          <span>— packets sent from host</span>
        </div>
        <div className="flex items-center gap-1.5 text-sm text-muted-foreground">
          <ArrowRightIcon className="h-4 w-4 text-amber-400" />
          <span className="text-amber-400 font-medium">FORWARD</span>
          <span>— packets routed through host</span>
        </div>
      </div>

      <Card className="glass-card">
        <CardContent className="p-0">
          <Tabs defaultValue="inbound">
            <div className="px-6 pt-4">
              <TabsList>
                <TabsTrigger value="inbound" className="gap-1.5">
                  <ArrowDownIcon className="h-3.5 w-3.5 text-cyan-400" />
                  Inbound ({MOCK_RULES.inbound.length})
                </TabsTrigger>
                <TabsTrigger value="outbound" className="gap-1.5">
                  <ArrowUpIcon className="h-3.5 w-3.5 text-emerald-400" />
                  Outbound ({MOCK_RULES.outbound.length})
                </TabsTrigger>
                <TabsTrigger value="forward" className="gap-1.5">
                  <ArrowRightIcon className="h-3.5 w-3.5 text-amber-400" />
                  Forward ({MOCK_RULES.forward.length})
                </TabsTrigger>
              </TabsList>
            </div>
            <TabsContent value="inbound" className="mt-0">
              <RulesTable rules={MOCK_RULES.inbound} />
            </TabsContent>
            <TabsContent value="outbound" className="mt-0">
              <RulesTable rules={MOCK_RULES.outbound} />
            </TabsContent>
            <TabsContent value="forward" className="mt-0">
              <RulesTable rules={MOCK_RULES.forward} />
            </TabsContent>
          </Tabs>
        </CardContent>
      </Card>
    </div>
  )
}
