// Firewall Client API Service
// Truy vấn danh sách luật Tường lửa thực tế kết nối từ Controller REST API (/v1/firewall/rules)

export interface LiveFirewallRule {
  id: string
  nodeId: string
  chain: 'INPUT' | 'OUTPUT' | 'FORWARD' | string
  ruleId: string
  protocol: string
  srcCidr: string
  dstCidr: string
  portSpec: string
  action: 'ACCEPT' | 'DROP' | 'REJECT' | string
  packets: number
  bytes: number
  updatedAt: string
}

/** Service API truy vấn luật Firewall Kernel thực tế */
export const firewallApi = {
  /** Truy vấn danh sách luật Firewall thực tế từ PostgreSQL CSDL (`GET /v1/firewall/rules`) */
  getLiveRules: async (nodeId?: string): Promise<LiveFirewallRule[]> => {
    try {
      const url = nodeId ? `/v1/firewall/rules?node_id=${nodeId}` : '/v1/firewall/rules'
      const response = await fetch(url, {
        headers: {
          'Content-Type': 'application/json',
        },
      })

      if (!response.ok) {
        throw new Error(`Controller Firewall API Error (${response.status})`)
      }

      const data = await response.json()
      if (!Array.isArray(data)) return []
      return data.map((item) => ({
        id: item.id || item.rule_id || item.ruleId,
        nodeId: item.node_id || item.nodeId,
        chain: (item.chain || 'INPUT').toUpperCase(),
        ruleId: item.rule_id || item.ruleId || 'rule-01',
        protocol: item.protocol || 'ANY',
        srcCidr: item.src_cidr || item.srcCidr || '0.0.0.0/0',
        dstCidr: item.dst_cidr || item.dstCidr || 'any',
        portSpec: item.port_spec || item.portSpec || 'any',
        action: (item.action || 'ACCEPT').toUpperCase(),
        packets: item.packets || 0,
        bytes: item.bytes || 0,
        updatedAt: item.updated_at || item.updatedAt || new Date().toISOString(),
      }))
    } catch (error) {
      console.error('Lỗi truy vấn live firewall rules:', error)
      return []
    }
  },
}
