# AegisNode

[![Rust Workspace CI](https://img.shields.io/badge/Rust-2024%20Edition-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Tests](https://img.shields.io/badge/tests-56%2F56%20passing-success.svg)]()

> **Enterprise-Grade, Cloud-Native Linux Firewall, Network Isolation, and Node Management Platform** written in Rust.

---

## 🚀 Overview

**AegisNode** is a zero-trust, high-availability node management platform engineered for multi-tenant cloud environments, carrier networks, and mission-critical enterprise infrastructure. 

It provides centralized control over **nftables stateful firewalling**, **network interface bonding**, **Virtual Routing and Forwarding (VRF) traffic isolation**, **systemd service lifecycles**, and **fleet-wide rollouts** without ever intercepting network packets in user-space.

```
                  +-----------------------------------+
                  |   AegisNode Central Controller    |
                  |     (Stateless REST & gRPC API)   |
                  +-----------------+-----------------+
                                    |
          +-------------------------+-------------------------+
          | mTLS (Ed25519 Signed Bundles + W3C Trace Context) |
          v                                                   v
+-------------------+                               +-------------------+
|  Node A (Agent)   |                               |  Node B (Agent)   |
| +---------------+ |                               | +---------------+ |
| | Aegis Execd   | |                               | | Aegis Execd   | |
| +-------+-------+ |                               | +-------+-------+ |
|         | UDS     |                               |         | UDS     |
|         v         |                               |         v         |
| [Linux Kernel]    |                               | [Linux Kernel]    |
| (nftables / VRF)  |                               | (nftables / VRF)  |
+-------------------+                               +-------------------+
```

---

## ✨ Key Features

### 🛡️ 1. Kernel-Level Non-Data-Path Architecture
* **Zero Overhead**: Network packets are filtered exclusively inside the Linux kernel via `nftables`. AegisNode operates out-of-band for configuration management and state inspection only.
* **Privilege Separation**: `aegisnode server` and `aegisnode agent` run as unprivileged non-root daemons. Privileged operations are delegated via Unix Domain Sockets (`0600`) to `execd` with kernel `SO_PEERCRED` UID verification.
* **Deterministic Safe Apply & Rollback**: Automatic confirmation timers revert failed network or firewall mutations if health probes or agent connectivity fail.

### 🔐 2. Cryptographic Integrity & Anti-Replay Protection
* **Ed25519 Signed Policy Bundles**: Policies are packaged into `SignedPolicyBundle` payloads signed by the Controller's Ed25519 private key.
* **Replay Attack Protection**: Agents enforce strictly monotonic sequence numbers (`sequence_number > last_applied_sequence`), target node ID matching, and SHA-256 payload checksum validation before applying any rule.
* **Cryptographic Audit Hash Chain**: Every security action is appended to a Merkle-linked audit log chain (`prev_event_hash -> current_event_hash`), guaranteeing instant detection of tampered records.

### 🏢 3. Enterprise Access Control & Multi-Person Approval
* **Role-Based Access Control (RBAC)**: 5 granular roles (`Viewer`, `Operator`, `SecurityAdmin`, `PlatformAdmin`, `Auditor`) across 12 explicit permissions.
* **Anti Self-Approval**: Change plan creators are strictly prohibited from approving their own plans.
* **2-Person Approval for Critical Plans**: High/Critical risk rollout plans require distinct cryptographic signatures from two independent administrators.
* **Break-Glass Emergency Access**: Time-bound emergency override mode with mandatory audit trail logging.

### 🌐 4. Controller High Availability & Agent Failover
* **Stateless Controllers**: Run multiple `aegisnode server` replicas behind HAProxy, NGINX, or Kubernetes Services.
* **PostgreSQL Advisory Lock Leader Election**: Background jobs (rollout coordination, certificate renewals) elect a Leader using `pg_try_advisory_lock` without requiring Redis or Etcd.
* **Multi-Endpoint Agent Failover**: Agents rotate across multiple Controller endpoints with backoff and jitter, falling back to local last-known-good policies if all Controllers go offline.

### ⚡ 5. Advanced Networking & Security Protection
* **Network Bonding**: Supports Active-Backup (mode 1) and 802.3ad LACP (mode 4) with **Management Interface Guard** to prevent SSH lockouts.
* **VRF Isolation**: Multi-tenant Virtual Routing and Forwarding (`ip vrf`) with dedicated routing tables (1-65535).
* **SYN Flood Mitigation**: Automated `synproxy` and per-source IP rate limiting (`limit rate 100/second burst 200 packets`).
* **Zero-Downtime Dynamic Sets**: Updates `nftables` IP sets live in memory without flushing or reloading rulesets.

### 📊 6. Production Observability
* **Prometheus Exporter**: Exposes `/metrics` for connected agents, rollout durations, firewall drop counters, and active blocklists.
* **W3C Distributed Tracing**: Propagates `traceparent` headers (`00-trace_id-parent_id-flags`) from REST API to gRPC and subprocess executions.
* **Grafana Dashboards**: Includes pre-built JSON specs for Fleet Health, Firewall Activity, and Rollout Console.

---

## 📁 Repository Structure

```
.
├── apps/
│   └── aegisnode/              # Primary multi-mode CLI & daemon entry point
├── crates/
│   ├── aegis-api/              # RESTful HTTP API (Axum) & Router
│   ├── aegis-audit/            # Merkle-linked Cryptographic Audit Trail
│   ├── aegis-cli/              # Command-line interface logic
│   ├── aegis-config/           # Configuration parsers & schemas
│   ├── aegis-core/             # Core identifiers, errors, PKI & Hardening
│   ├── aegis-firewall/         # nftables compiler, safe apply, blocker & VRF
│   ├── aegis-models/           # Domain models (Policy, Bundle, RBAC, VRF)
│   ├── aegis-observability/    # Prometheus metrics, W3C tracing & Grafana specs
│   ├── aegis-policy/           # Policy validation, normalizer & RBAC engine
│   ├── aegis-rpc/              # gRPC transport (tonic) & IPC client
│   └── aegis-storage/          # SQLite (Agent) & PostgreSQL (Controller) DBs
├── packaging/
│   └── systemd/                # Hardened production Systemd service units
├── scripts/
│   └── build_release.sh        # Automated release packaging script
└── web/                        # React + TypeScript + Vite Web Console UI
```

---

## 🛠️ Getting Started

### Prerequisites
* Linux OS (Kernel 5.4+ with `nftables` enabled)
* Rust toolchain (2024 Edition, 1.80+)
* Node.js & npm (for Web UI compilation)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/phucle996/AegisNode.git
cd AegisNode

# Build the Web UI static assets
cd web && npm install && npm run build && cd ..

# Compile all workspace crates in release mode
cargo build --release
```

Or run the automated release packaging script:

```bash
./scripts/build_release.sh
```

The compiled single binary will be available at `target/release/aegisnode`.

---

## 🧪 Running Tests

AegisNode maintains a comprehensive 100% passing test suite:

```bash
# Run all workspace unit & integration tests
cargo test --workspace

# Run specific crate tests (e.g. RBAC engine)
cargo test -p aegis-policy --test rbac_engine_test

# Run clippy linter checks
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

---

## 🖥️ Usage Modes

`aegisnode` operates as a single multi-call binary with distinct subcommands:

```bash
# 1. Local Mode: Directly inspect or apply local nftables policy
aegisnode local apply --file policy.yaml

# 2. Server Mode: Run the Central Controller Server
aegisnode server --config /etc/aegisnode/controller.yaml

# 3. Agent Mode: Run the non-root Local Agent Daemon
aegisnode agent --config /etc/aegisnode/agent.yaml

# 4. Execd Mode: Run the privileged Executor Daemon (Socket: /run/aegisnode/execd.sock)
aegisnode execd

# 5. CLI Mode: Interact with local or remote AegisNode daemons
aegisnode ctl status
```

---

## 🔒 Security Model & Hardening

* **Zero `unsafe` Code**: Codebase enforces `#![deny(unsafe_code)]` for memory safety guarantee.
* **API Payload Cap**: Maximum 10MB API request body limit.
* **Strict File Permissions**: Unix sockets and keys enforce `0600` permissions; data directories enforce `0700`.
* **Hardened Systemd Services**:
  * `aegisnode-server.service`: `NoNewPrivileges=true`, `ProtectSystem=strict`
  * `aegisnode-agent.service`: `NoNewPrivileges=true`, `ProtectSystem=full`
  * `aegisnode-execd.service`: `CapabilityBoundingSet=CAP_NET_ADMIN CAP_NET_RAW`

---

## 📜 License

Distributed under the **Apache License 2.0**. See [`LICENSE`](LICENSE) for details.
