# AegisNode Implementation Roadmap

Tóm tắt lộ trình 3 Stage phát triển AegisNode:

## Stage 1 — MVP (Single Node Autonomy)
- **Phase 0**: Repository Skeleton & Monorepo Bootstrap
- **Phase 1**: Core Domain Models & Errors
- **Phase 2**: Policy Validation Engine
- **Phase 3**: nftables Compiler
- **Phase 4**: nftables Runtime Backend
- **Phase 5**: Safe Apply & Automatic Rollback
- **Phase 6**: Local Agent Daemon & SQLite Storage
- **Phase 7**: Local CLI `aegisctl`
- **Phase 8**: Docker Host & Router Mode (NAT, Port Forwarding)
- **Phase 9**: Auto-Blocker (SSH Brute Force) & Journald Integration
- **Phase 10**: Embedded Local Web UI
- **Phase 11**: End-to-End Testing & Packaging

## Stage 2 — Middle (Multi-Node Central Management)
- **Phase 12-18**: Controller Foundation, Agent/Execd Separation, Central Storage (PostgreSQL), mTLS Node Enrollment, Network Domain Management, Systemd Domain Management, Canary Rollouts.

## Stage 3 — Production Ready (Cloud-Native & HA Infrastructure)
- **Phase 19-25**: HA Controller, RBAC & OIDC Integration, Signed Policy Bundles, Immutable Audit Trail, DR & Disaster Recovery.
