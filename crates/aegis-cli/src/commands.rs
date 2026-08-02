// Triển khai xử lý các lệnh CLI trong aegisctl
// Giao tiếp bất đồng bộ qua HTTP REST API / Unix Socket với Agent Daemon

use std::io::{self, Write};

use aegis_core::{AegisError, Result};
use aegis_firewall::{CompiledFirewallPolicy, FirewallCompiler, NftablesCompiler};
use aegis_models::firewall::FirewallPolicy;
use aegis_policy::PolicyValidator;
use reqwest::Client;
use serde_json::json;

use crate::args::{CliArgs, Commands, DockerCommands, FirewallCommands};
use crate::formatter::Formatter;

pub async fn handle_command(args: CliArgs) -> Result<()> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| AegisError::Internal(format!("Failed to build HTTP client: {e}")))?;

    let format = args.output;
    let endpoint = args.endpoint.trim_end_matches('/');

    match args.command {
        Commands::Status => {
            let url = format!("{endpoint}/v1/status");
            let resp = client.get(&url).send().await.map_err(|e| {
                AegisError::Internal(format!("Failed to connect to agent daemon at '{url}': {e}"))
            })?;

            if !resp.status().is_success() {
                let err_text = resp.text().await.unwrap_or_default();
                return Err(AegisError::Internal(format!(
                    "Status check failed: {err_text}"
                )));
            }

            let body: serde_json::Value = resp.json().await.map_err(|e| {
                AegisError::Internal(format!("Failed to parse status response: {e}"))
            })?;

            Formatter::print(&body, format);
        }

        Commands::Firewall { subcommand } => match subcommand {
            FirewallCommands::Validate { file } => {
                let content = tokio::fs::read_to_string(&file).await.map_err(|e| {
                    AegisError::NotFound(format!("Policy file '{file:?}' not found: {e}"))
                })?;
                let policy: FirewallPolicy = serde_yaml::from_str(&content).map_err(|e| {
                    AegisError::Validation(format!("Invalid policy YAML in '{file:?}': {e}"))
                })?;

                let report = PolicyValidator::validate(&policy);
                Formatter::print(&report, format);

                if !report.is_valid() {
                    return Err(AegisError::Validation(
                        "Policy validation failed with errors!".to_string(),
                    ));
                }
            }

            FirewallCommands::Compile { file } => {
                let content = tokio::fs::read_to_string(&file).await.map_err(|e| {
                    AegisError::NotFound(format!("Policy file '{file:?}' not found: {e}"))
                })?;
                let policy: FirewallPolicy = serde_yaml::from_str(&content).map_err(|e| {
                    AegisError::Validation(format!("Invalid policy YAML in '{file:?}': {e}"))
                })?;

                let compiler = NftablesCompiler::new();
                let compiled: CompiledFirewallPolicy = compiler.compile(&policy)?;

                Formatter::print(&compiled.nft_script, format);
            }

            FirewallCommands::Apply {
                file,
                rollback_after,
                yes,
            } => {
                let content = tokio::fs::read_to_string(&file).await.map_err(|e| {
                    AegisError::NotFound(format!("Policy file '{file:?}' not found: {e}"))
                })?;
                let policy: FirewallPolicy = serde_yaml::from_str(&content).map_err(|e| {
                    AegisError::Validation(format!("Invalid policy YAML in '{file:?}': {e}"))
                })?;

                let report = PolicyValidator::validate(&policy);
                if !report.is_valid() {
                    return Err(AegisError::Validation(
                        "Cannot apply policy containing critical validation errors!".to_string(),
                    ));
                }

                if !report.warnings.is_empty() && !yes {
                    println!("Policy contains security warnings:");
                    for warning in &report.warnings {
                        println!("  - [{}] {}", warning.code, warning.message);
                    }
                    print!("\nDo you want to proceed? [y/N]: ");
                    io::stdout().flush().ok();

                    let mut input = String::new();
                    io::stdin().read_line(&mut input).ok();
                    if !input.trim().eq_ignore_ascii_case("y") {
                        println!("Apply operation cancelled by user.");
                        return Ok(());
                    }
                }

                let url = format!("{endpoint}/v1/firewall/apply");
                let payload = json!({
                    "policy": policy,
                    "rollbackTimeoutSeconds": rollback_after,
                });

                let resp = client.post(&url).json(&payload).send().await.map_err(|e| {
                    AegisError::Internal(format!(
                        "Failed to connect to agent daemon at '{url}': {e}"
                    ))
                })?;

                if !resp.status().is_success() {
                    let err_text = resp.text().await.unwrap_or_default();
                    return Err(AegisError::Firewall(format!("Apply failed: {err_text}")));
                }

                let execution: aegis_firewall::ApplyExecution = resp.json().await.map_err(|e| {
                    AegisError::Internal(format!("Failed to parse apply response: {e}"))
                })?;

                Formatter::print(&execution, format);

                println!("\nPolicy applied in PENDING confirmation mode.");
                println!(
                    "Run the following command within {} seconds to commit:",
                    execution.timeout_seconds
                );
                println!("  aegisctl firewall confirm {}", execution.execution_id);
            }

            FirewallCommands::Confirm { execution_id } => {
                let url = format!("{endpoint}/v1/firewall/confirm");
                let payload = json!({ "executionId": execution_id });

                let resp = client.post(&url).json(&payload).send().await.map_err(|e| {
                    AegisError::Internal(format!(
                        "Failed to connect to agent daemon at '{url}': {e}"
                    ))
                })?;

                if !resp.status().is_success() {
                    let err_text = resp.text().await.unwrap_or_default();
                    return Err(AegisError::Firewall(format!("Confirm failed: {err_text}")));
                }

                let execution: aegis_firewall::ApplyExecution = resp.json().await.map_err(|e| {
                    AegisError::Internal(format!("Failed to parse confirm response: {e}"))
                })?;

                Formatter::print(&execution, format);
                println!("Apply Execution {} successfully COMMITTED!", execution_id);
            }

            FirewallCommands::Rollback { execution_id } => {
                let url = format!("{endpoint}/v1/firewall/rollback");
                let payload = json!({ "executionId": execution_id });

                let resp = client.post(&url).json(&payload).send().await.map_err(|e| {
                    AegisError::Internal(format!(
                        "Failed to connect to agent daemon at '{url}': {e}"
                    ))
                })?;

                if !resp.status().is_success() {
                    let err_text = resp.text().await.unwrap_or_default();
                    return Err(AegisError::Firewall(format!("Rollback failed: {err_text}")));
                }

                let execution: aegis_firewall::ApplyExecution = resp.json().await.map_err(|e| {
                    AegisError::Internal(format!("Failed to parse rollback response: {e}"))
                })?;

                Formatter::print(&execution, format);
                println!("Apply Execution {} successfully ROLLED BACK!", execution_id);
            }

            FirewallCommands::Rules | FirewallCommands::Counters => {
                let url = format!("{endpoint}/v1/firewall/policy");
                let resp = client.get(&url).send().await.map_err(|e| {
                    AegisError::Internal(format!(
                        "Failed to connect to agent daemon at '{url}': {e}"
                    ))
                })?;

                let body: serde_json::Value = resp.json().await.map_err(|e| {
                    AegisError::Internal(format!("Failed to parse rules response: {e}"))
                })?;

                Formatter::print(&body, format);
            }
        },

        Commands::Docker { subcommand } => match subcommand {
            DockerCommands::Containers | DockerCommands::Exposure => {
                let url = format!("{endpoint}/v1/docker/exposure");
                let resp = client.get(&url).send().await.map_err(|e| {
                    AegisError::Internal(format!(
                        "Failed to connect to agent daemon at '{url}': {e}"
                    ))
                })?;

                let body: serde_json::Value = resp.json().await.map_err(|e| {
                    AegisError::Internal(format!("Failed to parse docker exposure response: {e}"))
                })?;

                Formatter::print(&body, format);
            }
        },

        Commands::Audit { limit } => {
            let url = format!("{endpoint}/v1/audit?limit={limit}");
            let resp = client.get(&url).send().await.map_err(|e| {
                AegisError::Internal(format!("Failed to connect to agent daemon at '{url}': {e}"))
            })?;

            let body: serde_json::Value = resp.json().await.map_err(|e| {
                AegisError::Internal(format!("Failed to parse audit response: {e}"))
            })?;

            Formatter::print(&body, format);
        }

        Commands::Version => {
            let ver_info = json!({
                "name": "aegisctl",
                "version": env!("CARGO_PKG_VERSION"),
                "edition": "Rust 2024 Stable",
            });
            Formatter::print(&ver_info, format);
        }
    }

    Ok(())
}
