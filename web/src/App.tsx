// App.tsx: Root routing với React Router DOM v6 BrowserRouter
// Thêm bớt các Routes cho Phase 19: FleetOverview và RolloutConsole
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import AppLayout from '@/components/layout/AppLayout'
import DashboardPage from '@/pages/dashboard/DashboardPage'
import FirewallRulesPage from '@/pages/firewall/FirewallRulesPage'
import PolicyEditorPage from '@/pages/policy/PolicyEditorPage'
import DockerPage from '@/pages/docker/DockerPage'
import BlockedIPsPage from '@/pages/blocked/BlockedIPsPage'
import AuditLogsPage from '@/pages/audit/AuditLogsPage'
import { NodesOverview } from '@/components/NodesOverview'
import { ChangePlansPage } from '@/components/ChangePlansPage'

// Placeholder cho các trang chưa khả dụng
const ComingSoon = ({ title }: { title: string }) => (
  <div className="flex items-center justify-center h-64">
    <p className="text-muted-foreground">{title} — Coming Soon</p>
  </div>
)

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        {/* AppLayout bọc toàn bộ: sidebar + main area */}
        <Route path="/" element={<AppLayout />}>
          {/* Index route: Dashboard */}
          <Route index element={<DashboardPage />} />

          {/* Route Quản lý danh sách Nodes máy chủ thực tế */}
          <Route path="nodes" element={<NodesOverview />} />

          {/* Route Quản lý Change Plans và Rollout Console */}
          <Route path="plans" element={<ChangePlansPage />} />
          <Route path="rollouts" element={<Navigate to="/plans" replace />} />

          {/* Firewall Rules: Inbound / Outbound / Forward visualization */}
          <Route path="firewall" element={<FirewallRulesPage />} />

          {/* Policy Editor: Form mode + YAML mode + Safe Apply */}
          <Route path="policy" element={<PolicyEditorPage />} />

          {/* Docker Container inventory & exposure warnings */}
          <Route path="docker" element={<DockerPage />} />

          {/* Blocked IPs: Manual block + SSH auto-block list */}
          <Route path="blocked" element={<BlockedIPsPage />} />

          {/* Audit Logs: Agent activity trail */}
          <Route path="audit" element={<AuditLogsPage />} />

          {/* Settings */}
          <Route path="settings" element={<ComingSoon title="Settings" />} />

          {/* 404 fallback */}
          <Route path="*" element={<ComingSoon title="Page Not Found" />} />
        </Route>
      </Routes>
    </BrowserRouter>
  )
}
