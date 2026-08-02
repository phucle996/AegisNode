// Page: Policy Editor - Soạn thảo Policy (Form/YAML mode), Validate và Safe Apply
import { FileCodeIcon } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'

export default function PolicyEditorPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-foreground flex items-center gap-2">
          <FileCodeIcon className="h-6 w-6 text-primary" />
          Policy Editor
        </h1>
        <p className="text-muted-foreground text-sm mt-1">
          Soạn thảo và áp dụng Firewall Policy an toàn với Safe Apply + Rollback
        </p>
      </div>
      <Card className="glass-card">
        <CardHeader>
          <CardTitle className="text-sm">Policy Editor — Coming Soon</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground text-sm">
            Trình soạn thảo Policy YAML và Form mode đang được phát triển.
          </p>
        </CardContent>
      </Card>
    </div>
  )
}
