//! API key types and data structures
//!
//! This module contains request/response types for API key management.

use crate::core::models::user::types::User;
use crate::core::models::{ApiKey, RateLimits};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// API key creation request
#[derive(Debug, Clone)]
pub struct CreateApiKeyRequest {
    /// Key name/description
    pub name: String,
    /// Associated user ID
    pub user_id: Option<Uuid>,
    /// Associated team ID
    pub team_id: Option<Uuid>,
    /// Permissions for the key
    pub permissions: Vec<String>,
    /// Rate limits for the key
    pub rate_limits: Option<RateLimits>,
    /// Expiration date
    pub expires_at: Option<DateTime<Utc>>,
}

/// API key verification result
#[derive(Debug, Clone)]
pub struct ApiKeyVerification {
    /// The API key
    pub api_key: ApiKey,
    /// Associated user (if any)
    pub user: Option<User>,
    /// Whether the key is valid
    pub is_valid: bool,
    /// Reason for invalidity (if any)
    pub invalid_reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{Metadata, UsageStats};
    use crate::core::models::user::types::{UserRole, UserStatus, UserProfile};
    use crate::core::models::user::preferences::UserPreferences;
    use chrono::{Duration, Utc};

    // ==================== Helper Functions ====================

    fn create_test_api_key() -> ApiKey {
        ApiKey {
            metadata: Metadata::new(),
            name: "Test API Key".to_string(),
            key_hash: "hashed_key_value".to_string(),
            key_prefix: "sk-test".to_string(),
            user_id: None,
            team_id: None,
            permissions: vec!["read".to_string(), "write".to_string()],
            rate_limits: None,
            expires_at: None,
            is_active: true,
            last_used_at: None,
            usage_stats: UsageStats::default(),
        }
    }

    fn create_test_user() -> User {
        User {
            metadata: Metadata::new(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            display_name: Some("Test User".to_string()),
            password_hash: "hash".to_string(),
            role: UserRole::User,
            status: UserStatus::Active,
            team_ids: vec![],
            preferences: UserPreferences::default(),
            usage_stats: UsageStats::default(),
            rate_limits: None,
            last_login_at: None,
            email_verified: true,
            two_factor_enabled: false,
            profile: UserProfile::default(),
        }
    }

    // ==================== CreateApiKeyRequest Tests ====================

    #[test]
    fn test_create_api_key_request_minimal() {
        let request = CreateApiKeyRequest {
            name: "My API Key".to_string(),
            user_id: None,
            team_id: None,
            permissions: vec![],
            rate_limits: None,
            expires_at: None,
        };

        assert_eq!(request.name, "My API Key");
        assert!(request.user_id.is_none());
        assert!(request.team_id.is_none());
        assert!(request.permissions.is_empty());
    }

    #[test]
    fn test_create_api_key_request_with_user() {
        let user_id = Uuid::new_v4();
        let request = CreateApiKeyRequest {
            name: "User API Key".to_string(),
            user_id: Some(user_id),
            team_id: None,
            permissions: vec!["api:read".to_string()],
            rate_limits: None,
            expires_at: None,
        };

        assert_eq!(request.user_id, Some(user_id));
        assert_eq!(request.permissions.len(), 1);
    }

    #[test]
    fn test_create_api_key_request_with_team() {
        let team_id = Uuid::new_v4();
        let request = CreateApiKeyRequest {
            name: "Team API Key".to_string(),
            user_id: None,
            team_id: Some(team_id),
            permissions: vec!["team:read".to_string(), "team:write".to_string()],
            rate_limits: None,
            expires_at: None,
        };

        assert_eq!(request.team_id, Some(team_id));
        assert_eq!(request.permissions.len(), 2);
    }

    #[test]
    fn test_create_api_key_request_with_expiration() {
        let expires = Utc::now() + Duration::days(30);
        let request = CreateApiKeyRequest {
            name: "Expiring Key".to_string(),
            user_id: None,
            team_id: None,
            permissions: vec![],
            rate_limits: None,
            expires_at: Some(expires),
        };

        assert!(request.expires_at.is_some());
        assert!(request.expires_at.unwrap() > Utc::now());
    }

    #[test]
    fn test_create_api_key_request_with_rate_limits() {
        let rate_limits = RateLimits {
            rpm: Some(100),
            tpm: Some(10000),
            rpd: Some(1000),
            tpd: Some(100000),
            concurrent: Some(10),
        };
        let request = CreateApiKeyRequest {
            name: "Rate Limited Key".to_string(),
            user_id: None,
            team_id: None,
            permissions: vec![],
            rate_limits: Some(rate_limits),
            expires_at: None,
        };

        assert!(request.rate_limits.is_some());
        assert_eq!(request.rate_limits.as_ref().unwrap().rpm, Some(100));
    }

    #[test]
    fn test_create_api_key_request_full() {
        let user_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();
        let expires = Utc::now() + Duration::days(90);
        let rate_limits = RateLimits {
            rpm: Some(500),
            tpm: Some(50000),
            rpd: Some(5000),
            tpd: Some(500000),
            concurrent: Some(50),
        };

        let request = CreateApiKeyRequest {
            name: "Full Featured Key".to_string(),
            user_id: Some(user_id),
            team_id: Some(team_id),
            permissions: vec!["admin".to_string(), "read".to_string(), "write".to_string()],
            rate_limits: Some(rate_limits),
            expires_at: Some(expires),
        };

        assert_eq!(request.name, "Full Featured Key");
        assert!(request.user_id.is_some());
        assert!(request.team_id.is_some());
        assert_eq!(request.permissions.len(), 3);
        assert!(request.rate_limits.is_some());
        assert!(request.expires_at.is_some());
    }

    #[test]
    fn test_create_api_key_request_clone() {
        let request = CreateApiKeyRequest {
            name: "Clone Test".to_string(),
            user_id: Some(Uuid::new_v4()),
            team_id: None,
            permissions: vec!["test".to_string()],
            rate_limits: None,
            expires_at: None,
        };

        let cloned = request.clone();
        assert_eq!(cloned.name, request.name);
        assert_eq!(cloned.user_id, request.user_id);
        assert_eq!(cloned.permissions, request.permissions);
    }

    #[test]
    fn test_create_api_key_request_debug() {
        let request = CreateApiKeyRequest {
            name: "Debug Test".to_string(),
            user_id: None,
            team_id: None,
            permissions: vec![],
            rate_limits: None,
            expires_at: None,
        };

        let debug_str = format!("{:?}", request);
        assert!(debug_str.contains("CreateApiKeyRequest"));
        assert!(debug_str.contains("Debug Test"));
    }

    // ==================== ApiKeyVerification Tests ====================

    #[test]
    fn test_api_key_verification_valid() {
        let verification = ApiKeyVerification {
            api_key: create_test_api_key(),
            user: None,
            is_valid: true,
            invalid_reason: None,
        };

        assert!(verification.is_valid);
        assert!(verification.invalid_reason.is_none());
        assert!(verification.user.is_none());
    }

    #[test]
    fn test_api_key_verification_invalid_expired() {
        let mut api_key = create_test_api_key();
        api_key.expires_at = Some(Utc::now() - Duration::days(1));

        let verification = ApiKeyVerification {
            api_key,
            user: None,
            is_valid: false,
            invalid_reason: Some("API key has expired".to_string()),
        };

        assert!(!verification.is_valid);
        assert_eq!(verification.invalid_reason, Some("API key has expired".to_string()));
    }

    #[test]
    fn test_api_key_verification_invalid_inactive() {
        let mut api_key = create_test_api_key();
        api_key.is_active = false;

        let verification = ApiKeyVerification {
            api_key,
            user: None,
            is_valid: false,
            invalid_reason: Some("API key is inactive".to_string()),
        };

        assert!(!verification.is_valid);
        assert!(!verification.api_key.is_active);
    }

    #[test]
    fn test_api_key_verification_with_user() {
        let verification = ApiKeyVerification {
            api_key: create_test_api_key(),
            user: Some(create_test_user()),
            is_valid: true,
            invalid_reason: None,
        };

        assert!(verification.is_valid);
        assert!(verification.user.is_some());
        assert_eq!(verification.user.as_ref().unwrap().email, "test@example.com");
    }

    #[test]
    fn test_api_key_verification_invalid_user_inactive() {
        let mut user = create_test_user();
        user.status = UserStatus::Inactive;

        let verification = ApiKeyVerification {
            api_key: create_test_api_key(),
            user: Some(user),
            is_valid: false,
            invalid_reason: Some("Associated user is inactive".to_string()),
        };

        assert!(!verification.is_valid);
        assert!(verification.user.is_some());
    }

    #[test]
    fn test_api_key_verification_clone() {
        let verification = ApiKeyVerification {
            api_key: create_test_api_key(),
            user: Some(create_test_user()),
            is_valid: true,
            invalid_reason: None,
        };

        let cloned = verification.clone();
        assert_eq!(cloned.is_valid, verification.is_valid);
        assert_eq!(cloned.api_key.name, verification.api_key.name);
    }

    #[test]
    fn test_api_key_verification_debug() {
        let verification = ApiKeyVerification {
            api_key: create_test_api_key(),
            user: None,
            is_valid: true,
            invalid_reason: None,
        };

        let debug_str = format!("{:?}", verification);
        assert!(debug_str.contains("ApiKeyVerification"));
        assert!(debug_str.contains("is_valid: true"));
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_api_key_lifecycle_simulation() {
        // Step 1: Create request
        let user_id = Uuid::new_v4();
        let request = CreateApiKeyRequest {
            name: "Production API Key".to_string(),
            user_id: Some(user_id),
            team_id: None,
            permissions: vec!["api:read".to_string(), "api:write".to_string()],
            rate_limits: Some(RateLimits {
                rpm: Some(1000),
                tpm: Some(100000),
                rpd: None,
                tpd: None,
                concurrent: Some(100),
            }),
            expires_at: Some(Utc::now() + Duration::days(365)),
        };

        assert_eq!(request.permissions.len(), 2);

        // Step 2: Simulate key creation (in real code, this would create the key)
        let mut api_key = create_test_api_key();
        api_key.name = request.name.clone();
        api_key.user_id = request.user_id;
        api_key.permissions = request.permissions.clone();
        api_key.rate_limits = request.rate_limits.clone();
        api_key.expires_at = request.expires_at;

        // Step 3: Verify the key
        let verification = ApiKeyVerification {
            api_key,
            user: Some(create_test_user()),
            is_valid: true,
            invalid_reason: None,
        };

        assert!(verification.is_valid);
        assert_eq!(verification.api_key.name, "Production API Key");
        assert!(verification.api_key.expires_at.is_some());
    }

    #[test]
    fn test_api_key_permission_check() {
        let request = CreateApiKeyRequest {
            name: "Permission Test Key".to_string(),
            user_id: None,
            team_id: None,
            permissions: vec![
                "models:read".to_string(),
                "models:write".to_string(),
                "chat:completions".to_string(),
            ],
            rate_limits: None,
            expires_at: None,
        };

        // Check specific permissions
        assert!(request.permissions.contains(&"models:read".to_string()));
        assert!(request.permissions.contains(&"chat:completions".to_string()));
        assert!(!request.permissions.contains(&"admin".to_string()));
    }

    #[test]
    fn test_api_key_expiration_check() {
        let past_date = Utc::now() - Duration::hours(1);
        let future_date = Utc::now() + Duration::hours(1);

        let expired_request = CreateApiKeyRequest {
            name: "Expired".to_string(),
            user_id: None,
            team_id: None,
            permissions: vec![],
            rate_limits: None,
            expires_at: Some(past_date),
        };

        let valid_request = CreateApiKeyRequest {
            name: "Valid".to_string(),
            user_id: None,
            team_id: None,
            permissions: vec![],
            rate_limits: None,
            expires_at: Some(future_date),
        };

        // Check if expired
        let is_expired = |expires: Option<DateTime<Utc>>| -> bool {
            expires.map(|e| e < Utc::now()).unwrap_or(false)
        };

        assert!(is_expired(expired_request.expires_at));
        assert!(!is_expired(valid_request.expires_at));
    }

    #[test]
    fn test_rate_limits_configuration() {
        let standard_limits = RateLimits {
            rpm: Some(60),
            tpm: Some(10000),
            rpd: Some(1000),
            tpd: Some(10000),
            concurrent: Some(5),
        };

        let premium_limits = RateLimits {
            rpm: Some(600),
            tpm: Some(100000),
            rpd: Some(10000),
            tpd: Some(100000),
            concurrent: Some(50),
        };

        let standard_request = CreateApiKeyRequest {
            name: "Standard".to_string(),
            user_id: None,
            team_id: None,
            permissions: vec![],
            rate_limits: Some(standard_limits),
            expires_at: None,
        };

        let premium_request = CreateApiKeyRequest {
            name: "Premium".to_string(),
            user_id: None,
            team_id: None,
            permissions: vec![],
            rate_limits: Some(premium_limits),
            expires_at: None,
        };

        assert!(premium_request.rate_limits.as_ref().unwrap().rpm > standard_request.rate_limits.as_ref().unwrap().rpm);
    }
}
