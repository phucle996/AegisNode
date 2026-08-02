// Page: Docker Exposures - Kiểm soát Container inventory & cảnh báo cổng public
import { ContainerIcon } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'

export default function DockerPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-foreground flex items-center gap-2">
          <ContainerIcon className="h-6 w-6 text-primary" />
          Docker Exposures
        </h1>
        <p className="text-muted-foreground text-sm mt-1">
          Container inventory & cảnh báo cổng phơi nhiễm ra WAN
        </p>
      </div>
      <Card className="glass-card">
        <CardHeader>
          <CardTitle className="text-sm">Docker Inspector — Coming Soon</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground text-sm">
            Danh sách containers và exposure analyzer đang được phát triển.
          </p>
        </CardContent>
      </Card>
    </div>
  )
}
