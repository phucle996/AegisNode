// Integration tests cho AegisNode CLI (`aegisctl`) và Exit Codes

use aegis_cli::{exit_codes, run_cli_with_args};

#[tokio::test]
async fn test_cli_version() {
    let args = vec![
        "aegisctl".to_string(),
        "--output".to_string(),
        "json".to_string(),
        "version".to_string(),
    ];
    let code = run_cli_with_args(args).await;
    assert_eq!(code, exit_codes::SUCCESS);
}

#[tokio::test]
async fn test_cli_firewall_validate_valid_policy() {
    let args = vec![
        "aegisctl".to_string(),
        "--output".to_string(),
        "json".to_string(),
        "firewall".to_string(),
        "validate".to_string(),
        "../../tests/fixtures/policies/web-server.yaml".to_string(),
    ];
    let code = run_cli_with_args(args).await;
    assert_eq!(code, exit_codes::SUCCESS);
}

#[tokio::test]
async fn test_cli_firewall_compile_preview() {
    let args = vec![
        "aegisctl".to_string(),
        "firewall".to_string(),
        "compile".to_string(),
        "../../tests/fixtures/policies/web-server.yaml".to_string(),
    ];
    let code = run_cli_with_args(args).await;
    assert_eq!(code, exit_codes::SUCCESS);
}

#[tokio::test]
async fn test_cli_firewall_validate_invalid_policy() {
    let args = vec![
        "aegisctl".to_string(),
        "firewall".to_string(),
        "validate".to_string(),
        "../../tests/fixtures/policies/invalid-port.yaml".to_string(),
    ];
    let code = run_cli_with_args(args).await;
    assert_eq!(code, exit_codes::VALIDATION_FAILURE);
}
