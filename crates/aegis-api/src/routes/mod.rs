//! AegisNode API Domain Routes Module
//! Tập hợp và phân loại toàn bộ REST API Handlers theo ranh giới miền (Domain Boundaries).

// Submodules cho Controller Rest API
pub mod controller {
    pub use crate::routes::enrollment::*;
    pub use crate::routes::health::*;
    pub use crate::routes::inventory::*;
    pub use crate::routes::rollout::*;
}

// Submodules cho Agent Rest API
pub mod agent {
    pub use crate::routes::blocker::*;
    pub use crate::routes::firewall::*;
    pub use crate::routes::network::*;
    pub use crate::routes::systemd::*;
}

pub mod blocker;
pub mod enrollment;
pub mod firewall;
pub mod health;
pub mod inventory;
pub mod network;
pub mod rollout;
pub mod systemd;

pub use blocker::{
    add_block_entry_handler, get_blocker_entries_handler, remove_block_entry_handler,
};
pub use enrollment::{
    create_enrollment_token_handler, node_heartbeat_handler, sign_agent_csr_handler,
};
pub use firewall::{
    apply_policy_handler, confirm_policy_handler, get_audit_logs_handler,
    get_docker_exposure_handler, get_policy_handler, get_status_handler,
    prometheus_metrics_handler, rollback_policy_handler, set_router_forwarding_handler,
    validate_policy_handler,
};
pub use health::{ha_status_handler, health_check_handler, readiness_check_handler};
pub use inventory::{
    get_node_handler, get_node_inventory_handler, list_nodes_handler,
    report_node_inventory_handler, update_node_labels_handler,
};
pub use network::{
    apply_network_config_handler, create_network_profile_handler, get_network_interfaces_handler,
    list_network_profiles_handler,
};
pub use rollout::{
    cancel_rollout_handler, create_rollout_handler, get_rollout_status_handler,
    pause_rollout_handler, resume_rollout_handler, rollback_rollout_handler,
};
pub use systemd::{
    control_systemd_service_handler, execute_service_op_handler, list_systemd_services_handler,
    query_journal_logs_handler,
};
