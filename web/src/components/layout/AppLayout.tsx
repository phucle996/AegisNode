// AppLayout: Sidebar navigation + main content area (Updated for Phase 19 Fleet & Rollout Navigation)
import { NavLink, Outlet } from 'react-router-dom'
import {
  LayoutDashboardIcon,
  ShieldIcon,
  FileCodeIcon,
  ContainerIcon,
  BanIcon,
  ScrollTextIcon,
  SettingsIcon,
  ZapIcon,
  ServerIcon,
  LayersIcon,
} from 'lucide-react'
import { cn } from '@/lib/utils'
import { Separator } from '@/components/ui/separator'
import { TooltipProvider } from '@/components/ui/tooltip'

// Định nghĩa danh sách các mục điều hướng điều hướng Sidebar (Bao gồm Fleet & Rollouts)
const NAV_ITEMS = [
  { to: '/', icon: LayoutDashboardIcon, label: 'Dashboard', end: true },
  { to: '/fleet', icon: ServerIcon, label: 'Fleet Nodes' },
  { to: '/rollouts', icon: LayersIcon, label: 'Rollout Console' },
  { to: '/firewall', icon: ShieldIcon, label: 'Firewall Rules' },
  { to: '/policy', icon: FileCodeIcon, label: 'Policy Editor' },
  { to: '/docker', icon: ContainerIcon, label: 'Docker' },
  { to: '/blocked', icon: BanIcon, label: 'Blocked IPs' },
  { to: '/audit', icon: ScrollTextIcon, label: 'Audit Logs' },
]

export default function AppLayout() {
  return (
    <TooltipProvider delayDuration={200}>
      <div className="flex h-screen overflow-hidden bg-background">
        {/* Sidebar */}
        <aside className="flex flex-col w-64 border-r border-sidebar-border bg-sidebar-background shrink-0">
          {/* Logo / Brand Header */}
          <div className="flex items-center gap-2.5 px-4 h-14 border-b border-sidebar-border">
            <div className="flex items-center justify-center w-7 h-7 rounded-md bg-primary/10 border border-primary/30">
              <ZapIcon className="h-4 w-4 text-primary" />
            </div>
            <div>
              <p className="text-sm font-semibold text-foreground leading-none">AegisNode</p>
              <p className="text-[10px] text-muted-foreground mt-0.5">Central Fleet Manager</p>
            </div>
          </div>

          {/* Thanh menu điều hướng chính */}
          <nav className="flex-1 px-3 py-4 space-y-1 overflow-y-auto">
            {NAV_ITEMS.map(({ to, icon: Icon, label, end }) => (
              <NavLink
                key={to}
                to={to}
                end={end}
                className={({ isActive }) =>
                  cn(
                    'flex items-center gap-3 px-3 py-2 rounded-md text-sm transition-all duration-150',
                    isActive
                      ? 'bg-sidebar-accent text-sidebar-primary font-medium'
                      : 'text-sidebar-foreground hover:bg-sidebar-accent/50 hover:text-sidebar-accent-foreground',
                  )
                }
              >
                <Icon className="h-4 w-4 shrink-0" />
                {label}
              </NavLink>
            ))}
          </nav>

          <Separator className="mx-3 w-auto" />

          {/* Menu cài đặt và chỉ báo trạng thái mTLS kết nối bên dưới */}
          <div className="px-3 py-3">
            <NavLink
              to="/settings"
              className={({ isActive }) =>
                cn(
                  'flex items-center gap-3 px-3 py-2 rounded-md text-sm transition-all duration-150',
                  isActive
                    ? 'bg-sidebar-accent text-sidebar-primary font-medium'
                    : 'text-sidebar-foreground hover:bg-sidebar-accent/50 hover:text-sidebar-accent-foreground',
                )
              }
            >
              <SettingsIcon className="h-4 w-4 shrink-0" />
              Settings
            </NavLink>

            {/* Chỉ báo trạng thái Controller & mTLS Fleet */}
            <div className="mt-3 px-3 py-2 rounded-md bg-emerald-500/10 border border-emerald-500/20">
              <div className="flex items-center gap-2">
                <div className="h-1.5 w-1.5 rounded-full bg-emerald-400 animate-pulse" />
                <span className="text-xs text-emerald-400 font-medium">Controller Active</span>
              </div>
              <p className="text-[10px] text-muted-foreground mt-0.5">mTLS Fleet Manager</p>
            </div>
          </div>
        </aside>

        {/* Nội dung chính các trang (Outlet render) */}
        <main className="flex-1 overflow-y-auto">
          <div className="p-6 max-w-6xl mx-auto">
            <Outlet />
          </div>
        </main>
      </div>
    </TooltipProvider>
  )
}
