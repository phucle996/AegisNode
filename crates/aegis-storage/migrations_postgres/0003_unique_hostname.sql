-- Phase 24: Unique Hostname Index on Nodes Table
CREATE UNIQUE INDEX IF NOT EXISTS idx_nodes_hostname ON nodes(hostname);
