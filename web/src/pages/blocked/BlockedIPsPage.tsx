// Page: Blocked IPs - Quản lý danh sách IP bị block thủ công và auto-block
import { BanIcon } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'

export default function BlockedIPsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-foreground flex items-center gap-2">
          <BanIcon className="h-6 w-6 text-destructive" />
          Blocked IPs
        </h1>
        <p className="text-muted-foreground text-sm mt-1">
          Quản lý IP bị block thủ công và auto-block từ SSH detector
        </p>
      </div>
      <Card className="glass-card">
        <CardHeader>
          <CardTitle className="text-sm">Block Manager — Coming Soon</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground text-sm">
            Danh sách IP blocklist và SSH brute-force detector đang được phát triển.
          </p>
        </CardContent>
      </Card>
    </div>
  )
}
