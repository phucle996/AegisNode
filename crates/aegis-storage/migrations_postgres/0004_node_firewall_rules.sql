-- Phase 25: Real OS Kernel Firewall Rules Sync Table
CREATE TABLE IF NOT EXISTS node_firewall_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    node_id UUID NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    chain VARCHAR(32) NOT NULL, -- INPUT, OUTPUT, FORWARD
    rule_id VARCHAR(64) NOT NULL,
    protocol VARCHAR(16) NOT NULL DEFAULT 'ANY',
    src_cidr VARCHAR(45) NOT NULL DEFAULT '0.0.0.0/0',
    dst_cidr VARCHAR(45) NOT NULL DEFAULT 'any',
    port_spec VARCHAR(64) NOT NULL DEFAULT 'any',
    action VARCHAR(32) NOT NULL DEFAULT 'ACCEPT',
    packets BIGINT NOT NULL DEFAULT 0,
    bytes BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT unique_node_rule UNIQUE(node_id, chain, rule_id)
);

CREATE INDEX IF NOT EXISTS idx_node_fw_rules_node ON node_firewall_rules(node_id);
CREATE INDEX IF NOT EXISTS idx_node_fw_rules_chain ON node_firewall_rules(chain);
