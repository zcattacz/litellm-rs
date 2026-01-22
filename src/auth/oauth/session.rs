//! Session management for OAuth authentication

use super::types::{OAuthState, UserInfo};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// Session data stored for authenticated users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthSession {
    /// Session ID
    pub session_id: String,

    /// User information from OAuth provider
    pub user_info: UserInfo,

    /// Access token from the OAuth provider
    pub access_token: String,

    /// Refresh token (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,

    /// ID token (for OIDC)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,

    /// When the access token expires
    pub token_expires_at: DateTime<Utc>,

    /// When the session was created
    pub created_at: DateTime<Utc>,

    /// When the session was last accessed
    pub last_accessed_at: DateTime<Utc>,

    /// Session expiration time
    pub expires_at: DateTime<Utc>,

    /// IP address of the client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,

    /// User agent of the client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,

    /// Internal user ID (after user creation/lookup)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal_user_id: Option<Uuid>,

    /// Assigned role
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

impl OAuthSession {
    /// Create a new OAuth session
    pub fn new(
        user_info: UserInfo,
        access_token: String,
        token_expires_in: u64,
        session_ttl: u64,
    ) -> Self {
        let now = Utc::now();
        Self {
            session_id: Uuid::new_v4().to_string(),
            user_info,
            access_token,
            refresh_token: None,
            id_token: None,
            token_expires_at: now + chrono::Duration::seconds(token_expires_in as i64),
            created_at: now,
            last_accessed_at: now,
            expires_at: now + chrono::Duration::seconds(session_ttl as i64),
            ip_address: None,
            user_agent: None,
            internal_user_id: None,
            role: None,
        }
    }

    /// Set the refresh token
    pub fn with_refresh_token(mut self, token: impl Into<String>) -> Self {
        self.refresh_token = Some(token.into());
        self
    }

    /// Set the ID token
    pub fn with_id_token(mut self, token: impl Into<String>) -> Self {
        self.id_token = Some(token.into());
        self
    }

    /// Set client metadata
    pub fn with_client_info(
        mut self,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        self.ip_address = ip_address;
        self.user_agent = user_agent;
        self
    }

    /// Set the internal user ID
    pub fn with_internal_user_id(mut self, user_id: Uuid) -> Self {
        self.internal_user_id = Some(user_id);
        self
    }

    /// Set the role
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.role = Some(role.into());
        self
    }

    /// Check if the session has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if the access token has expired
    pub fn is_token_expired(&self) -> bool {
        Utc::now() > self.token_expires_at
    }

    /// Update the last accessed timestamp
    pub fn touch(&mut self) {
        self.last_accessed_at = Utc::now();
    }

    /// Extend the session expiration
    pub fn extend(&mut self, additional_seconds: u64) {
        self.expires_at += chrono::Duration::seconds(additional_seconds as i64);
    }

    /// Update the access token
    pub fn update_token(&mut self, access_token: String, expires_in: u64) {
        self.access_token = access_token;
        self.token_expires_at = Utc::now() + chrono::Duration::seconds(expires_in as i64);
    }
}

/// Session store trait for managing OAuth sessions
#[async_trait]
pub trait SessionStore: Send + Sync {
    /// Store a session
    async fn set(&self, session: OAuthSession) -> Result<(), SessionError>;

    /// Retrieve a session by ID
    async fn get(&self, session_id: &str) -> Result<Option<OAuthSession>, SessionError>;

    /// Delete a session
    async fn delete(&self, session_id: &str) -> Result<(), SessionError>;

    /// Update a session
    async fn update(&self, session: OAuthSession) -> Result<(), SessionError>;

    /// Store an OAuth state for CSRF protection
    async fn set_state(&self, state: OAuthState) -> Result<(), SessionError>;

    /// Retrieve and remove an OAuth state
    async fn get_and_delete_state(&self, state_id: &str) -> Result<Option<OAuthState>, SessionError>;

    /// Get all sessions for a user
    async fn get_user_sessions(&self, user_email: &str) -> Result<Vec<OAuthSession>, SessionError>;

    /// Delete all sessions for a user
    async fn delete_user_sessions(&self, user_email: &str) -> Result<usize, SessionError>;

    /// Clean up expired sessions
    async fn cleanup_expired(&self) -> Result<usize, SessionError>;
}

/// Session store errors
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Session not found")]
    NotFound,

    #[error("Session expired")]
    Expired,

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Connection error: {0}")]
    Connection(String),
}

/// In-memory session store implementation
#[derive(Clone)]
pub struct InMemorySessionStore {
    sessions: Arc<DashMap<String, OAuthSession>>,
    states: Arc<DashMap<String, OAuthState>>,
    cleanup_interval: Duration,
}

impl Default for InMemorySessionStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemorySessionStore {
    /// Create a new in-memory session store
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            states: Arc::new(DashMap::new()),
            cleanup_interval: Duration::from_secs(300), // 5 minutes
        }
    }

    /// Create with custom cleanup interval
    pub fn with_cleanup_interval(mut self, interval: Duration) -> Self {
        self.cleanup_interval = interval;
        self
    }

    /// Start background cleanup task
    pub fn start_cleanup_task(self: Arc<Self>) {
        let store = self.clone();
        let interval = self.cleanup_interval;

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                if let Err(e) = store.cleanup_expired().await {
                    tracing::warn!("Session cleanup error: {}", e);
                }
            }
        });
    }
}

impl std::fmt::Debug for InMemorySessionStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemorySessionStore")
            .field("session_count", &self.sessions.len())
            .field("state_count", &self.states.len())
            .finish()
    }
}

#[async_trait]
impl SessionStore for InMemorySessionStore {
    async fn set(&self, session: OAuthSession) -> Result<(), SessionError> {
        self.sessions.insert(session.session_id.clone(), session);
        Ok(())
    }

    async fn get(&self, session_id: &str) -> Result<Option<OAuthSession>, SessionError> {
        match self.sessions.get(session_id) {
            Some(entry) => {
                let session = entry.value().clone();
                if session.is_expired() {
                    drop(entry);
                    self.sessions.remove(session_id);
                    Ok(None)
                } else {
                    Ok(Some(session))
                }
            }
            None => Ok(None),
        }
    }

    async fn delete(&self, session_id: &str) -> Result<(), SessionError> {
        self.sessions.remove(session_id);
        Ok(())
    }

    async fn update(&self, session: OAuthSession) -> Result<(), SessionError> {
        self.sessions.insert(session.session_id.clone(), session);
        Ok(())
    }

    async fn set_state(&self, state: OAuthState) -> Result<(), SessionError> {
        self.states.insert(state.state.clone(), state);
        Ok(())
    }

    async fn get_and_delete_state(&self, state_id: &str) -> Result<Option<OAuthState>, SessionError> {
        match self.states.remove(state_id) {
            Some((_, state)) => {
                if state.is_expired() {
                    Ok(None)
                } else {
                    Ok(Some(state))
                }
            }
            None => Ok(None),
        }
    }

    async fn get_user_sessions(&self, user_email: &str) -> Result<Vec<OAuthSession>, SessionError> {
        let sessions: Vec<OAuthSession> = self
            .sessions
            .iter()
            .filter(|entry| entry.value().user_info.email == user_email && !entry.value().is_expired())
            .map(|entry| entry.value().clone())
            .collect();
        Ok(sessions)
    }

    async fn delete_user_sessions(&self, user_email: &str) -> Result<usize, SessionError> {
        let to_delete: Vec<String> = self
            .sessions
            .iter()
            .filter(|entry| entry.value().user_info.email == user_email)
            .map(|entry| entry.key().clone())
            .collect();

        let count = to_delete.len();
        for session_id in to_delete {
            self.sessions.remove(&session_id);
        }
        Ok(count)
    }

    async fn cleanup_expired(&self) -> Result<usize, SessionError> {
        let now = Utc::now();

        // Clean up expired sessions
        let expired_sessions: Vec<String> = self
            .sessions
            .iter()
            .filter(|entry| entry.value().expires_at < now)
            .map(|entry| entry.key().clone())
            .collect();

        let session_count = expired_sessions.len();
        for session_id in expired_sessions {
            self.sessions.remove(&session_id);
        }

        // Clean up expired states
        let expired_states: Vec<String> = self
            .states
            .iter()
            .filter(|entry| entry.value().is_expired())
            .map(|entry| entry.key().clone())
            .collect();

        for state_id in expired_states {
            self.states.remove(&state_id);
        }

        Ok(session_count)
    }
}

/// Redis session store implementation
#[cfg(feature = "redis")]
pub struct RedisSessionStore {
    client: redis::Client,
    prefix: String,
    session_ttl: u64,
    state_ttl: u64,
}

#[cfg(feature = "redis")]
impl RedisSessionStore {
    /// Create a new Redis session store
    pub fn new(redis_url: &str) -> Result<Self, SessionError> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| SessionError::Connection(e.to_string()))?;

        Ok(Self {
            client,
            prefix: "oauth:".to_string(),
            session_ttl: 3600,
            state_ttl: 600,
        })
    }

    /// Set custom prefix for Redis keys
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// Set session TTL
    pub fn with_session_ttl(mut self, ttl: u64) -> Self {
        self.session_ttl = ttl;
        self
    }

    /// Set state TTL
    pub fn with_state_ttl(mut self, ttl: u64) -> Self {
        self.state_ttl = ttl;
        self
    }

    fn session_key(&self, session_id: &str) -> String {
        format!("{}session:{}", self.prefix, session_id)
    }

    fn state_key(&self, state_id: &str) -> String {
        format!("{}state:{}", self.prefix, state_id)
    }

    fn user_sessions_key(&self, email: &str) -> String {
        format!("{}user_sessions:{}", self.prefix, email)
    }
}

#[cfg(feature = "redis")]
impl std::fmt::Debug for RedisSessionStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisSessionStore")
            .field("prefix", &self.prefix)
            .field("session_ttl", &self.session_ttl)
            .finish()
    }
}

#[cfg(feature = "redis")]
#[async_trait]
impl SessionStore for RedisSessionStore {
    async fn set(&self, session: OAuthSession) -> Result<(), SessionError> {
        use redis::AsyncCommands;

        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| SessionError::Connection(e.to_string()))?;

        let key = self.session_key(&session.session_id);
        let value = serde_json::to_string(&session)
            .map_err(|e| SessionError::Serialization(e.to_string()))?;

        // Calculate TTL from session expiration
        let ttl = (session.expires_at - Utc::now()).num_seconds().max(0) as u64;

        let _: () = conn
            .set_ex(&key, &value, ttl)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        // Add to user's session set
        let user_key = self.user_sessions_key(&session.user_info.email);
        let _: () = conn
            .sadd(&user_key, &session.session_id)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;
        let _: () = conn
            .expire(&user_key, ttl as i64)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        Ok(())
    }

    async fn get(&self, session_id: &str) -> Result<Option<OAuthSession>, SessionError> {
        use redis::AsyncCommands;

        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| SessionError::Connection(e.to_string()))?;

        let key = self.session_key(session_id);
        let value: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        match value {
            Some(v) => {
                let session: OAuthSession = serde_json::from_str(&v)
                    .map_err(|e| SessionError::Serialization(e.to_string()))?;

                if session.is_expired() {
                    self.delete(session_id).await?;
                    Ok(None)
                } else {
                    Ok(Some(session))
                }
            }
            None => Ok(None),
        }
    }

    async fn delete(&self, session_id: &str) -> Result<(), SessionError> {
        use redis::AsyncCommands;

        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| SessionError::Connection(e.to_string()))?;

        // Get session first to remove from user's set
        if let Some(session) = self.get(session_id).await? {
            let user_key = self.user_sessions_key(&session.user_info.email);
            let _: () = conn
                .srem(&user_key, session_id)
                .await
                .map_err(|e| SessionError::Storage(e.to_string()))?;
        }

        let key = self.session_key(session_id);
        let _: () = conn
            .del(&key)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        Ok(())
    }

    async fn update(&self, session: OAuthSession) -> Result<(), SessionError> {
        self.set(session).await
    }

    async fn set_state(&self, state: OAuthState) -> Result<(), SessionError> {
        use redis::AsyncCommands;

        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| SessionError::Connection(e.to_string()))?;

        let key = self.state_key(&state.state);
        let value = serde_json::to_string(&state)
            .map_err(|e| SessionError::Serialization(e.to_string()))?;

        let _: () = conn
            .set_ex(&key, &value, state.ttl_seconds)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        Ok(())
    }

    async fn get_and_delete_state(&self, state_id: &str) -> Result<Option<OAuthState>, SessionError> {
        use redis::AsyncCommands;

        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| SessionError::Connection(e.to_string()))?;

        let key = self.state_key(state_id);

        // Get and delete atomically
        let value: Option<String> = conn
            .get_del(&key)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        match value {
            Some(v) => {
                let state: OAuthState = serde_json::from_str(&v)
                    .map_err(|e| SessionError::Serialization(e.to_string()))?;

                if state.is_expired() {
                    Ok(None)
                } else {
                    Ok(Some(state))
                }
            }
            None => Ok(None),
        }
    }

    async fn get_user_sessions(&self, user_email: &str) -> Result<Vec<OAuthSession>, SessionError> {
        use redis::AsyncCommands;

        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| SessionError::Connection(e.to_string()))?;

        let user_key = self.user_sessions_key(user_email);
        let session_ids: Vec<String> = conn
            .smembers(&user_key)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        let mut sessions = Vec::new();
        for session_id in session_ids {
            if let Some(session) = self.get(&session_id).await? {
                sessions.push(session);
            }
        }

        Ok(sessions)
    }

    async fn delete_user_sessions(&self, user_email: &str) -> Result<usize, SessionError> {
        use redis::AsyncCommands;

        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| SessionError::Connection(e.to_string()))?;

        let user_key = self.user_sessions_key(user_email);
        let session_ids: Vec<String> = conn
            .smembers(&user_key)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        let count = session_ids.len();

        for session_id in &session_ids {
            let key = self.session_key(session_id);
            let _: () = conn
                .del(&key)
                .await
                .map_err(|e| SessionError::Storage(e.to_string()))?;
        }

        let _: () = conn
            .del(&user_key)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        Ok(count)
    }

    async fn cleanup_expired(&self) -> Result<usize, SessionError> {
        // Redis handles TTL-based expiration automatically
        // This is a no-op for Redis
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_user_info() -> UserInfo {
        UserInfo::new("123", "test@example.com", "google")
            .with_name("Test User")
    }

    fn create_test_session() -> OAuthSession {
        OAuthSession::new(
            create_test_user_info(),
            "access_token_123".to_string(),
            3600,
            7200,
        )
    }

    #[test]
    fn test_session_creation() {
        let session = create_test_session();

        assert!(!session.session_id.is_empty());
        assert_eq!(session.user_info.email, "test@example.com");
        assert_eq!(session.access_token, "access_token_123");
        assert!(!session.is_expired());
        assert!(!session.is_token_expired());
    }

    #[test]
    fn test_session_builder() {
        let session = OAuthSession::new(
            create_test_user_info(),
            "access_token".to_string(),
            3600,
            7200,
        )
        .with_refresh_token("refresh_token")
        .with_id_token("id_token")
        .with_client_info(Some("127.0.0.1".to_string()), Some("Mozilla/5.0".to_string()))
        .with_internal_user_id(Uuid::new_v4())
        .with_role("admin");

        assert_eq!(session.refresh_token, Some("refresh_token".to_string()));
        assert_eq!(session.id_token, Some("id_token".to_string()));
        assert!(session.ip_address.is_some());
        assert!(session.user_agent.is_some());
        assert!(session.internal_user_id.is_some());
        assert_eq!(session.role, Some("admin".to_string()));
    }

    #[test]
    fn test_session_expiration() {
        let mut session = create_test_session();
        session.expires_at = Utc::now() - chrono::Duration::seconds(1);
        assert!(session.is_expired());
    }

    #[test]
    fn test_token_expiration() {
        let mut session = create_test_session();
        session.token_expires_at = Utc::now() - chrono::Duration::seconds(1);
        assert!(session.is_token_expired());
    }

    #[test]
    fn test_session_touch() {
        let mut session = create_test_session();
        let original = session.last_accessed_at;
        std::thread::sleep(std::time::Duration::from_millis(10));
        session.touch();
        assert!(session.last_accessed_at > original);
    }

    #[test]
    fn test_session_extend() {
        let mut session = create_test_session();
        let original = session.expires_at;
        session.extend(3600);
        assert!(session.expires_at > original);
    }

    #[test]
    fn test_session_update_token() {
        let mut session = create_test_session();
        session.update_token("new_access_token".to_string(), 7200);
        assert_eq!(session.access_token, "new_access_token");
    }

    #[tokio::test]
    async fn test_in_memory_session_store() {
        let store = InMemorySessionStore::new();
        let session = create_test_session();
        let session_id = session.session_id.clone();

        // Set session
        store.set(session).await.unwrap();

        // Get session
        let retrieved = store.get(&session_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().session_id, session_id);

        // Delete session
        store.delete(&session_id).await.unwrap();
        let deleted = store.get(&session_id).await.unwrap();
        assert!(deleted.is_none());
    }

    #[tokio::test]
    async fn test_in_memory_state_store() {
        let store = InMemorySessionStore::new();
        let state = OAuthState::new("google");
        let state_id = state.state.clone();

        // Set state
        store.set_state(state).await.unwrap();

        // Get and delete state
        let retrieved = store.get_and_delete_state(&state_id).await.unwrap();
        assert!(retrieved.is_some());

        // Should be deleted after retrieval
        let again = store.get_and_delete_state(&state_id).await.unwrap();
        assert!(again.is_none());
    }

    #[tokio::test]
    async fn test_in_memory_user_sessions() {
        let store = InMemorySessionStore::new();

        let mut session1 = create_test_session();
        session1.user_info.email = "user@example.com".to_string();

        let mut session2 = create_test_session();
        session2.user_info.email = "user@example.com".to_string();

        store.set(session1).await.unwrap();
        store.set(session2).await.unwrap();

        let user_sessions = store.get_user_sessions("user@example.com").await.unwrap();
        assert_eq!(user_sessions.len(), 2);

        let deleted = store.delete_user_sessions("user@example.com").await.unwrap();
        assert_eq!(deleted, 2);

        let after_delete = store.get_user_sessions("user@example.com").await.unwrap();
        assert!(after_delete.is_empty());
    }

    #[tokio::test]
    async fn test_in_memory_cleanup_expired() {
        let store = InMemorySessionStore::new();

        let mut expired_session = create_test_session();
        expired_session.expires_at = Utc::now() - chrono::Duration::seconds(1);
        store.set(expired_session.clone()).await.unwrap();

        let valid_session = create_test_session();
        store.set(valid_session.clone()).await.unwrap();

        let cleaned = store.cleanup_expired().await.unwrap();
        assert_eq!(cleaned, 1);

        // Expired session should be gone
        let retrieved = store.get(&expired_session.session_id).await.unwrap();
        assert!(retrieved.is_none());

        // Valid session should still exist
        let still_valid = store.get(&valid_session.session_id).await.unwrap();
        assert!(still_valid.is_some());
    }

    #[tokio::test]
    async fn test_in_memory_expired_state_cleanup() {
        let store = InMemorySessionStore::new();

        let mut expired_state = OAuthState::new("google");
        expired_state.created_at = Utc::now() - chrono::Duration::seconds(700);
        store.set_state(expired_state.clone()).await.unwrap();

        // Expired state should return None
        let retrieved = store.get_and_delete_state(&expired_state.state).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_session_serialization() {
        let session = create_test_session();
        let json = serde_json::to_string(&session).unwrap();
        let parsed: OAuthSession = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.session_id, session.session_id);
        assert_eq!(parsed.user_info.email, session.user_info.email);
    }

    #[test]
    fn test_session_error_display() {
        assert_eq!(SessionError::NotFound.to_string(), "Session not found");
        assert_eq!(SessionError::Expired.to_string(), "Session expired");
        assert!(SessionError::Storage("test".to_string()).to_string().contains("test"));
    }
}
