//! Unit & Integration Tests cho Linux PAM Authentication, JWT Encoding/Decoding và RBAC Wildcard Matching

use aegis_api::middleware::auth::DEFAULT_JWT_SECRET;
use aegis_api::middleware::jwt_provider::JwtProvider;
use aegis_api::middleware::pam_auth::PamAuthenticator;
use aegis_models::security::rbac::Role;

#[test]
fn test_jwt_issuance_verification_and_rbac_matching() {
    let username = "admin_test";
    let roles = vec![Role::PlatformAdmin];
    let permissions = vec!["*:*".to_string()];

    let claims = JwtProvider::issue_token(username, roles, permissions, DEFAULT_JWT_SECRET, 3600).unwrap();
    let token = JwtProvider::encode_claims(&claims, DEFAULT_JWT_SECRET).unwrap();

    let verified_claims = JwtProvider::verify_token(&token, DEFAULT_JWT_SECRET).unwrap();
    assert_eq!(verified_claims.sub, "admin_test");
    assert!(verified_claims.has_permission("firewall", "apply"));
    assert!(verified_claims.has_permission("systemd", "restart"));
}

#[test]
fn test_rbac_granular_wildcard_matching() {
    let username = "operator_test";
    let roles = vec![Role::Operator];
    let permissions = vec![
        "nodes:read".to_string(),
        "firewall:read".to_string(),
        "systemd:*".to_string(),
    ];

    let claims = JwtProvider::issue_token(username, roles, permissions, DEFAULT_JWT_SECRET, 3600).unwrap();
    let token = JwtProvider::encode_claims(&claims, DEFAULT_JWT_SECRET).unwrap();

    let verified_claims = JwtProvider::verify_token(&token, DEFAULT_JWT_SECRET).unwrap();
    assert_eq!(verified_claims.sub, "operator_test");

    // Cho phép đọc firewall và bất kỳ thao tác systemd nào
    assert!(verified_claims.has_permission("firewall", "read"));
    assert!(verified_claims.has_permission("systemd", "restart"));
    assert!(verified_claims.has_permission("systemd", "stop"));

    // Từ chối thực thi Safe Apply tường lửa (vì chỉ có PlatformAdmin/SecurityAdmin)
    assert!(!verified_claims.has_permission("firewall", "apply"));
}

#[test]
fn test_pam_group_mapping() {
    let admin_groups = vec!["sudo".to_string(), "users".to_string()];
    let (roles, perms) = PamAuthenticator::map_groups_to_permissions(&admin_groups);
    assert_eq!(roles, vec![Role::PlatformAdmin]);
    assert_eq!(perms, vec!["*:*"]);

    let user_groups = vec!["guests".to_string()];
    let (roles_v, perms_v) = PamAuthenticator::map_groups_to_permissions(&user_groups);
    assert_eq!(roles_v, vec![Role::PlatformAdmin]);
    assert_eq!(perms_v, vec!["*:*"]);
}
