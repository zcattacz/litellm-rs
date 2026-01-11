//! OAuth module tests

use super::*;

#[cfg(test)]
mod integration_tests {
    use crate::auth::oauth::client::OAuthClient;
    use crate::auth::oauth::config::{OAuthConfig, OAuthGatewayConfig, OAuthProvider};
    use crate::auth::oauth::session::{InMemorySessionStore, OAuthSession, SessionStore};
    use crate::auth::oauth::types::{OAuthState, TokenResponse, UserInfo};
    use std::sync::Arc;

    #[test]
    fn test_full_oauth_config_setup() {
        let mut config = OAuthGatewayConfig::default();

        // Add multiple providers
        config.add_provider(
            "google",
            OAuthConfig::google("google_client_id", "https://app.example.com/oauth/google/callback")
                .with_client_secret("google_secret")
                .add_scope("calendar.readonly"),
        );

        config.add_provider(
            "github",
            OAuthConfig::github("github_client_id", "https://app.example.com/oauth/github/callback")
                .with_client_secret("github_secret"),
        );

        config.add_provider(
            "okta",
            OAuthConfig::okta(
                "okta_client_id",
                "https://app.example.com/oauth/okta/callback",
                "dev-12345.okta.com",
            )
            .with_client_secret("okta_secret"),
        );

        config.default_provider = Some("google".to_string());
        config.session_ttl_seconds = 7200;
        config.auto_create_users = true;

        assert!(config.validate().is_ok());
        assert_eq!(config.providers.len(), 3);
        assert!(config.get_default_provider().is_some());
    }

    #[test]
    fn test_oauth_client_authorization_flow() {
        let config = OAuthConfig::google("test_client", "https://app.example.com/callback")
            .with_client_secret("test_secret")
            .with_param("access_type", "offline")
            .with_param("prompt", "consent");

        let client = OAuthClient::new(config).unwrap();
        let (url, state) = client.get_authorization_url();

        // Verify URL contains all required parameters
        assert!(url.contains("client_id=test_client"));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("access_type=offline"));
        assert!(url.contains("prompt=consent"));
        assert!(url.contains("code_challenge=")); // PKCE
        assert!(url.contains("code_challenge_method=S256"));

        // Verify state is properly generated
        assert!(!state.state.is_empty());
        assert!(state.code_verifier.is_some());
        assert!(!state.is_expired());
    }

    #[tokio::test]
    async fn test_session_lifecycle() {
        let store = InMemorySessionStore::new();
        let user_info = UserInfo::new("user123", "test@example.com", "google")
            .with_name("Test User")
            .with_email_verified(true);

        // Create session
        let session = OAuthSession::new(user_info.clone(), "access_token_123".to_string(), 3600, 7200)
            .with_refresh_token("refresh_token_456")
            .with_role("user")
            .with_client_info(Some("127.0.0.1".to_string()), Some("TestBrowser".to_string()));

        let session_id = session.session_id.clone();

        // Store session
        store.set(session).await.unwrap();

        // Retrieve session
        let retrieved = store.get(&session_id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.user_info.email, "test@example.com");
        assert_eq!(retrieved.role, Some("user".to_string()));

        // Update session
        let mut updated = retrieved.clone();
        updated.update_token("new_access_token".to_string(), 7200);
        store.update(updated).await.unwrap();

        // Verify update
        let after_update = store.get(&session_id).await.unwrap().unwrap();
        assert_eq!(after_update.access_token, "new_access_token");

        // Delete session
        store.delete(&session_id).await.unwrap();
        let deleted = store.get(&session_id).await.unwrap();
        assert!(deleted.is_none());
    }

    #[tokio::test]
    async fn test_oauth_state_storage() {
        let store = InMemorySessionStore::new();
        let state = OAuthState::with_pkce("google")
            .with_redirect_uri("https://app.example.com/callback")
            .with_ttl(300);

        let state_id = state.state.clone();

        // Store state
        store.set_state(state).await.unwrap();

        // Retrieve and delete state (atomic operation)
        let retrieved = store.get_and_delete_state(&state_id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert!(retrieved.code_verifier.is_some());

        // Second retrieval should return None
        let second = store.get_and_delete_state(&state_id).await.unwrap();
        assert!(second.is_none());
    }

    #[tokio::test]
    async fn test_user_sessions_management() {
        let store = InMemorySessionStore::new();
        let email = "multidevice@example.com";

        // Create multiple sessions for same user
        for i in 0..3 {
            let user_info = UserInfo::new(format!("user{}", i), email, "google");
            let session =
                OAuthSession::new(user_info, format!("token{}", i), 3600, 7200);
            store.set(session).await.unwrap();
        }

        // Get all user sessions
        let sessions = store.get_user_sessions(email).await.unwrap();
        assert_eq!(sessions.len(), 3);

        // Delete all user sessions
        let deleted = store.delete_user_sessions(email).await.unwrap();
        assert_eq!(deleted, 3);

        // Verify deletion
        let after_delete = store.get_user_sessions(email).await.unwrap();
        assert!(after_delete.is_empty());
    }

    #[tokio::test]
    async fn test_expired_session_cleanup() {
        let store = InMemorySessionStore::new();

        // Create expired session
        let user_info = UserInfo::new("expired_user", "expired@example.com", "google");
        let mut session = OAuthSession::new(user_info, "token".to_string(), 3600, 1);
        session.expires_at = chrono::Utc::now() - chrono::Duration::seconds(100);
        let session_id = session.session_id.clone();
        store.set(session).await.unwrap();

        // Create valid session
        let user_info2 = UserInfo::new("valid_user", "valid@example.com", "google");
        let session2 = OAuthSession::new(user_info2, "token2".to_string(), 3600, 7200);
        let session_id2 = session2.session_id.clone();
        store.set(session2).await.unwrap();

        // Run cleanup
        let cleaned = store.cleanup_expired().await.unwrap();
        assert_eq!(cleaned, 1);

        // Expired session should be gone
        assert!(store.get(&session_id).await.unwrap().is_none());

        // Valid session should remain
        assert!(store.get(&session_id2).await.unwrap().is_some());
    }

    #[test]
    fn test_provider_specific_configurations() {
        // Test Google
        let google = OAuthConfig::google("id", "https://app.com/callback");
        assert!(google.use_pkce);
        assert!(google.auth_url.contains("accounts.google.com"));
        assert!(google.scopes.contains(&"openid".to_string()));

        // Test GitHub (no PKCE)
        let github = OAuthConfig::github("id", "https://app.com/callback");
        assert!(!github.use_pkce);
        assert!(github.scopes.contains(&"read:user".to_string()));

        // Test Okta
        let okta = OAuthConfig::okta("id", "https://app.com/callback", "dev.okta.com");
        assert!(okta.use_pkce);
        assert!(okta.auth_url.contains("dev.okta.com"));
        assert!(okta.jwks_uri.is_some());

        // Test Auth0
        let auth0 = OAuthConfig::auth0("id", "https://app.com/callback", "tenant.auth0.com");
        assert!(auth0.use_pkce);
        assert!(auth0.logout_url.unwrap().contains("v2/logout"));
    }

    #[test]
    fn test_role_mapping_configuration() {
        let config = OAuthConfig::google("id", "https://app.com/callback")
            .with_role_mapping("admin@company.com", "super_admin")
            .with_role_mapping("*@company.com", "employee")
            .with_role_mapping("*", "guest");

        assert_eq!(config.role_mapping.len(), 3);
        assert_eq!(
            config.role_mapping.get("admin@company.com"),
            Some(&"super_admin".to_string())
        );
    }

    #[test]
    fn test_token_response_handling() {
        // Standard JSON response
        let json_response = TokenResponse {
            access_token: "ya29.abc123".to_string(),
            refresh_token: Some("1//xyz789".to_string()),
            expires_in: 3600,
            token_type: "Bearer".to_string(),
            scope: Some("openid email profile".to_string()),
            id_token: Some("eyJ.abc.xyz".to_string()),
        };

        assert_eq!(json_response.token_type, "Bearer");
        assert!(json_response.refresh_token.is_some());
        assert!(json_response.id_token.is_some());

        // Serialization roundtrip
        let json = serde_json::to_string(&json_response).unwrap();
        let parsed: TokenResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.access_token, json_response.access_token);
    }

    #[test]
    fn test_user_info_from_different_providers() {
        // Google-style user info
        let google_user = UserInfo::new("123456789", "user@gmail.com", "google")
            .with_name("Google User")
            .with_picture("https://lh3.googleusercontent.com/photo.jpg")
            .with_email_verified(true);

        assert_eq!(google_user.provider, "google");
        assert!(google_user.email_verified);

        // GitHub-style user info
        let github_user = UserInfo::new("12345", "user@github.com", "github")
            .with_name("githubuser")
            .with_picture("https://avatars.githubusercontent.com/u/12345")
            .with_claim("login", serde_json::json!("githubuser"))
            .with_claim("public_repos", serde_json::json!(42));

        assert_eq!(github_user.provider, "github");
        assert!(github_user.extra_claims.contains_key("login"));
        assert!(github_user.extra_claims.contains_key("public_repos"));

        // Microsoft-style user info
        let ms_user = UserInfo::new("abc-def-ghi", "user@company.onmicrosoft.com", "microsoft")
            .with_name("Enterprise User")
            .with_claim("preferred_username", serde_json::json!("user@company.com"))
            .with_claim("tid", serde_json::json!("tenant-id"));

        assert_eq!(ms_user.provider, "microsoft");
        assert!(ms_user.extra_claims.contains_key("tid"));
    }

    #[test]
    fn test_oauth_state_pkce_generation() {
        let state = OAuthState::with_pkce("google");

        // Verify PKCE components
        let verifier = state.code_verifier.as_ref().unwrap();
        assert_eq!(verifier.len(), 64);

        let challenge = state.code_challenge().unwrap();
        assert!(!challenge.is_empty());
        // Challenge should be base64url encoded (no +, /, =)
        assert!(!challenge.contains('+'));
        assert!(!challenge.contains('/'));
        assert!(!challenge.contains('='));

        // Verify nonce is generated
        assert!(state.nonce.is_some());
    }

    #[test]
    fn test_oauth_provider_from_string() {
        assert_eq!("google".parse::<OAuthProvider>().unwrap(), OAuthProvider::Google);
        assert_eq!("GOOGLE".parse::<OAuthProvider>().unwrap(), OAuthProvider::Google);
        assert_eq!("azure".parse::<OAuthProvider>().unwrap(), OAuthProvider::Microsoft);
        assert_eq!("entra".parse::<OAuthProvider>().unwrap(), OAuthProvider::Microsoft);
        assert!("unknown_provider".parse::<OAuthProvider>().is_err());
    }

    #[test]
    fn test_session_token_management() {
        let user_info = UserInfo::new("123", "test@example.com", "google");
        let mut session = OAuthSession::new(user_info, "token1".to_string(), 3600, 7200);

        // Initial token
        assert_eq!(session.access_token, "token1");
        assert!(!session.is_token_expired());

        // Update token
        session.update_token("token2".to_string(), 7200);
        assert_eq!(session.access_token, "token2");

        // Make token expired
        session.token_expires_at = chrono::Utc::now() - chrono::Duration::seconds(1);
        assert!(session.is_token_expired());
    }
}
