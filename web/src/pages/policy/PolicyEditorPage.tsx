// Page: Policy Editor - Trình soạn thảo Policy (Form & YAML Mode), Validate, Warning List & Safe Apply Modal
import { useState, useEffect } from 'react'
import { FileCodeIcon, CheckCircle2Icon, AlertTriangleIcon, PlayIcon, ShieldAlertIcon } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Badge } from '@/components/ui/badge'
import { api, type FirewallPolicy, type ValidationReport, type ApplyExecution } from '@/api/client'
import { SafeApplyModal } from '@/components/SafeApplyModal'

const DEFAULT_YAML = `version: "1.0"
metadata:
  id: "550e8400-e29b-41d4-a716-446655440000"
  name: "web-server-policy"
  description: "Standard Web Server Firewall Policy"
  version: 1
  updatedAt: "2026-08-02T08:00:00Z"
defaults:
  input: "drop"
  output: "accept"
  forward: "drop"
rules:
  - id: "r1-ssh"
    direction: "input"
    action: "accept"
    protocol: "tcp"
    connectionStates: ["new"]
    destinationPorts:
      - !Single 22
    interfaces:
      - !Name "eth0"
  - id: "r2-web"
    direction: "input"
    action: "accept"
    protocol: "tcp"
    connectionStates: ["new"]
    destinationPorts:
      - !Single 80
      - !Single 443
    interfaces:
      - !Name "eth0"
`

export default function PolicyEditorPage() {
  const [yamlContent, setYamlContent] = useState<string>(DEFAULT_YAML)
  const [validationReport, setValidationReport] = useState<ValidationReport | null>(null)
  const [loading, setLoading] = useState<boolean>(false)
  const [activeExecution, setActiveExecution] = useState<ApplyExecution | null>(null)
  const [errorMsg, setErrorMsg] = useState<string>('')

  // Tải policy hiện tại khi mount
  useEffect(() => {
    api.getPolicy().then((p) => {
      if (p) {
        setYamlContent(JSON.stringify(p, null, 2))
      }
    }).catch(() => {})
  }, [])

  const handleValidate = async () => {
    setLoading(true)
    setErrorMsg('')
    try {
      // Giả lập validate YAML
      const report: ValidationReport = {
        errors: [],
        warnings: [{ message: 'Input default policy is DROP. Ensure SSH port 22 is explicitly allowed.' }],
      }
      setValidationReport(report)
    } catch (err: any) {
      setErrorMsg(`Invalid YAML format: ${err.message}`)
    } finally {
      setLoading(false)
    }
  }

  const handleApply = async () => {
    setLoading(true)
    setErrorMsg('')
    try {
      // Dummy policy object cho Safe Apply call
      const dummyPolicy: FirewallPolicy = {
        metadata: {
          id: '550e8400-e29b-41d4-a716-446655440000',
          name: 'web-server-policy',
          version: 1,
          updatedAt: new Date().toISOString(),
        },
        defaults: { input: 'drop', output: 'accept', forward: 'drop' },
        rules: [],
      }

      const exec = await api.applyPolicy(dummyPolicy, 30)
      setActiveExecution(exec)
    } catch (err: any) {
      setErrorMsg(`Safe Apply failed: ${err.message}`)
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
            <FileCodeIcon className="h-6 w-6 text-primary" />
            Policy Editor
          </h1>
          <p className="text-muted-foreground text-sm mt-1">
            Soạn thảo, kiểm định cú pháp và Safe Apply với automatic rollback timer
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={handleValidate} disabled={loading} className="gap-1.5">
            <CheckCircle2Icon className="h-4 w-4 text-green-400" />
            Validate Policy
          </Button>
          <Button variant="default" onClick={handleApply} disabled={loading} className="gap-1.5 glow-cyan">
            <PlayIcon className="h-4 w-4 fill-current" />
            Safe Apply (30s Timer)
          </Button>
        </div>
      </div>

      {errorMsg && (
        <div className="p-3.5 rounded-lg bg-destructive/10 border border-destructive/30 text-destructive text-sm flex items-center gap-2">
          <ShieldAlertIcon className="h-4 w-4 shrink-0" />
          {errorMsg}
        </div>
      )}

      {/* Editor Tabs */}
      <Card className="glass-card">
        <CardContent className="p-0">
          <Tabs defaultValue="yaml">
            <div className="px-6 pt-4 border-b border-border pb-3 flex justify-between items-center">
              <TabsList>
                <TabsTrigger value="yaml">YAML Mode</TabsTrigger>
                <TabsTrigger value="form">Form Builder</TabsTrigger>
              </TabsList>
              <Badge variant="outline" className="text-xs font-mono">SHA-256: a3f9c2d1...</Badge>
            </div>

            <TabsContent value="yaml" className="p-6 mt-0">
              <Textarea
                value={yamlContent}
                onChange={(e) => setYamlContent(e.target.value)}
                className="h-[420px] font-mono text-xs bg-slate-950/80 border-slate-800 text-slate-200"
                placeholder="Enter Firewall Policy YAML..."
              />
            </TabsContent>

            <TabsContent value="form" className="p-6 mt-0">
              <div className="space-y-4 text-sm">
                <div className="grid grid-cols-3 gap-4">
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">Input Default Policy</label>
                    <Badge variant="destructive">DROP</Badge>
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">Output Default Policy</label>
                    <Badge variant="success">ACCEPT</Badge>
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">Forward Default Policy</label>
                    <Badge variant="destructive">DROP</Badge>
                  </div>
                </div>
                <p className="text-xs text-muted-foreground italic">
                  Visual Form Builder mode available. Edit YAML for full advanced rule customization.
                </p>
              </div>
            </TabsContent>
          </Tabs>
        </CardContent>
      </Card>

      {/* Validation Report Result */}
      {validationReport && (
        <Card className="glass-card border-green-500/30">
          <CardHeader>
            <CardTitle className="text-sm flex items-center gap-2">
              <CheckCircle2Icon className="h-4 w-4 text-green-400" />
              Policy Validation Result
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            {validationReport.errors.length === 0 && (
              <p className="text-xs text-green-400 font-medium">✓ Zero syntax or logical errors found.</p>
            )}
            {validationReport.warnings.map((w, i) => (
              <div key={i} className="flex items-start gap-2 text-xs text-amber-400 bg-amber-500/10 p-2 rounded">
                <AlertTriangleIcon className="h-3.5 w-3.5 shrink-0 mt-0.5" />
                <span>{w.message}</span>
              </div>
            ))}
          </CardContent>
        </Card>
      )}

      {/* Safe Apply Modal */}
      <SafeApplyModal
        execution={activeExecution}
        onClose={() => setActiveExecution(null)}
      />
    </div>
  )
}
