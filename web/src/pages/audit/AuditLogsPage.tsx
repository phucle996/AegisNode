// Page: Audit Logs - Lịch sử hoạt động của Agent
import { ScrollTextIcon } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'

export default function AuditLogsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-foreground flex items-center gap-2">
          <ScrollTextIcon className="h-6 w-6 text-primary" />
          Audit Logs
        </h1>
        <p className="text-muted-foreground text-sm mt-1">
          Lịch sử hoạt động và vết kiểm tra của AegisNode Agent
        </p>
      </div>
      <Card className="glass-card">
        <CardHeader>
          <CardTitle className="text-sm">Audit Trail — Coming Soon</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground text-sm">
            Lịch sử audit events đang được phát triển.
          </p>
        </CardContent>
      </Card>
    </div>
  )
}
