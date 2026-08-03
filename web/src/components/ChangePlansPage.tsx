// Change Plans Page Component
// Trang Phân hệ Quản lý Change Plans (/plans) với 2 Tabs chính: Tab 1 (Summary & Rollouts), Tab 2 (Plan Setup Studio)

import React, { useState } from 'react'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs'
import { RolloutConsole } from './RolloutConsole'
import { PlanSetupStudio } from './rollout/PlanSetupStudio'
import { BarChart3Icon, SlidersIcon, ShieldCheckIcon } from 'lucide-react'

export const ChangePlansPage: React.FC = () => {
  // State quản lý Tab đang active ('summary' hoặc 'setup')
  const [activeTab, setActiveTab] = useState<string>('summary')

  // Xử lý sau khi phát hành Plan từ Tab 2 -> Tự động chuyển về Tab 1
  const handleLaunchSuccess = () => {
    setActiveTab('summary')
  }

  return (
    <div className="space-y-6">
      {/* Header Phân hệ Change Plans */}
      <div className="flex items-center justify-between">
        <div>
          <div className="flex items-center gap-2">
            <span className="p-2 bg-indigo-500/10 border border-indigo-500/20 text-indigo-400 rounded-lg">
              <ShieldCheckIcon className="w-5 h-5" />
            </span>
            <h1 className="text-2xl font-bold text-slate-100">Change Plans Management</h1>
          </div>
          <p className="text-xs text-slate-400 mt-1">
            Soạn thảo, cấu hình chuỗi các bước thay đổi an ninh và giám sát đợt triển khai Multi-Node Rollout
          </p>
        </div>
      </div>

      {/* Cấu trúc 2 Tabs sử dụng Radix UI Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full space-y-6">
        <TabsList className="bg-slate-900 border border-slate-800 p-1 rounded-xl grid grid-cols-2 w-full max-w-md">
          {/* Tab 1: Summary & Tiến độ Rollouts */}
          <TabsTrigger
            value="summary"
            className="flex items-center gap-2 text-xs font-medium py-2 rounded-lg data-[state=active]:bg-indigo-600 data-[state=active]:text-white text-slate-400 transition-all"
          >
            <BarChart3Icon className="w-4 h-4" />
            Summary & Rollouts
          </TabsTrigger>

          {/* Tab 2: Thiết lập Change Plan Studio */}
          <TabsTrigger
            value="setup"
            className="flex items-center gap-2 text-xs font-medium py-2 rounded-lg data-[state=active]:bg-indigo-600 data-[state=active]:text-white text-slate-400 transition-all"
          >
            <SlidersIcon className="w-4 h-4" />
            Plan Setup Studio
          </TabsTrigger>
        </TabsList>

        {/* Nội dung Tab 1: Summary & Tiến độ Rollouts */}
        <TabsContent value="summary" className="space-y-4">
          <RolloutConsole />
        </TabsContent>

        {/* Nội dung Tab 2: Thiết lập Change Plan Studio */}
        <TabsContent value="setup" className="space-y-4">
          <PlanSetupStudio onLaunchSuccess={handleLaunchSuccess} />
        </TabsContent>
      </Tabs>
    </div>
  )
}
