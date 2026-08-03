// AegisNode Web API Client
// Kết nối trực tiếp tới REST API server `/v1/*`

export interface NftCapabilityReport {
  nftInstalled: boolean
  nftVersion: string
  hasPermissions: boolean
  kernelSupport: boolean
  ipv6Support: boolean
}

export interface StatusResponse {
  status: string
  version: string
  capability: NftCapabilityReport
}

export interface FirewallMetadata {
  id: string
  name: string
  description?: string
  version: number
  updatedAt: string
}

export interface FirewallDefaults {
  input: 'accept' | 'drop' | 'reject'
  output: 'accept' | 'drop' | 'reject'
  forward: 'accept' | 'drop' | 'reject'
}

export interface FirewallRule {
  id: string
  direction: 'input' | 'output' | 'forward'
  action: 'accept' | 'drop' | 'reject'
  protocol?: 'tcp' | 'udp' | 'icmp' | 'icmpv6'
  connectionStates?: ('new' | 'established' | 'related' | 'invalid')[]
  sourceCidrs?: { 0: string }[]
  destinationCidrs?: { 0: string }[]
  destinationPorts?: any[]
  interfaces?: any[]
}

export interface FirewallPolicy {
  metadata: FirewallMetadata
  defaults: FirewallDefaults
  rules: FirewallRule[]
}

export interface ValidationError {
  ruleId?: string
  field: string
  message: string
}

export interface ValidationWarning {
  ruleId?: string
  message: string
}

export interface ValidationReport {
  errors: ValidationError[]
  warnings: ValidationWarning[]
}

export interface ApplyExecution {
  executionId: string
  policyId: string
  snapshotId: string
  state: 'CREATED' | 'VALIDATED' | 'SNAPSHOTTED' | 'APPLIED_PENDING_CONFIRMATION' | 'COMMITTED' | 'ROLLING_BACK' | 'ROLLED_BACK' | 'FAILED'
  timeoutSeconds: number
  createdAt: string
  expiresAt: string
  errorMessage?: string
}

export interface BlockEntry {
  ip: string
  reason: string
  actor: string
  durationSeconds?: number
  createdAt: string
  expiresAt?: string
}

export interface PublishedPort {
  hostIp: string
  hostPort: number
  containerPort: number
  protocol: string
}

export interface ExposureWarning {
  containerId: string
  containerName: string
  publishedPort: PublishedPort
  isDatabase: boolean
  warningMessage: string
}

export interface DockerContainer {
  id: string
  name: string
  image: string
  state: string
  // Chỉ số % CPU tiêu thụ thực tế đọc từ docker stats
  cpuPerc?: string
  // Chỉ số RAM tiêu thụ (Usage / Limit) thực tế đọc từ docker stats
  memUsage?: string
  networks: string[]
  publishedPorts: PublishedPort[]
  labels: Record<string, string>
}

export interface DockerExposureReport {
  dockerAvailable: boolean
  containers: DockerContainer[]
  publicExposures: ExposureWarning[]
  labelPolicies: any[]
}

export interface AuditLogEntry {
  id: number
  eventType: string
  actor: string
  details: string
  createdAt: string
}

const API_BASE = '/v1'

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
    ...options,
  })

  if (!res.ok) {
    const errorText = await res.text()
    throw new Error(`API Error (${res.status}): ${errorText}`)
  }

  return res.json()
}

export const api = {
  // Agent Status
  getStatus: () => request<StatusResponse>('/status'),

  // Firewall Policy
  getPolicy: () => request<FirewallPolicy | null>('/firewall/policy'),
  validatePolicy: (policy: FirewallPolicy) =>
    request<ValidationReport>('/firewall/validate', {
      method: 'POST',
      body: JSON.stringify(policy),
    }),
  applyPolicy: (policy: FirewallPolicy, rollbackTimeoutSeconds = 30) =>
    request<ApplyExecution>('/firewall/apply', {
      method: 'POST',
      body: JSON.stringify({ policy, rollbackTimeoutSeconds }),
    }),
  confirmApply: (executionId: string) =>
    request<ApplyExecution>('/firewall/confirm', {
      method: 'POST',
      body: JSON.stringify({ executionId }),
    }),
  rollbackApply: (executionId: string) =>
    request<ApplyExecution>('/firewall/rollback', {
      method: 'POST',
      body: JSON.stringify({ executionId }),
    }),

  // Docker Exposures
  getDockerExposures: () => request<DockerExposureReport>('/docker/exposure'),

  // Blocker IPs
  getBlockEntries: () => request<BlockEntry[]>('/blocker/entries'),
  addBlockEntry: (ip: string, durationSeconds?: number, reason?: string) =>
    request<BlockEntry>('/blocker/add', {
      method: 'POST',
      body: JSON.stringify({ ip, durationSeconds, reason }),
    }),
  removeBlockEntry: (ip: string) =>
    request<BlockEntry>('/blocker/remove', {
      method: 'POST',
      body: JSON.stringify({ ip }),
    }),

  // Audit Logs
  getAuditLogs: () => request<AuditLogEntry[]>('/audit'),
}
