//! Team invitation models

use super::member::TeamRole;
use crate::core::models::Metadata;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Team invitation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamInvitation {
    /// Invitation metadata
    #[serde(flatten)]
    pub metadata: Metadata,
    /// Team ID
    pub team_id: Uuid,
    /// Email address
    pub email: String,
    /// Invited role
    pub role: TeamRole,
    /// Invitation token
    #[serde(skip_serializing)]
    pub token: String,
    /// Invited by
    pub invited_by: Uuid,
    /// Expires at
    pub expires_at: chrono::DateTime<chrono::Utc>,
    /// Invitation status
    pub status: InvitationStatus,
}

/// Invitation status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvitationStatus {
    /// Pending acceptance
    Pending,
    /// Accepted
    Accepted,
    /// Declined
    Declined,
    /// Expired
    Expired,
    /// Cancelled
    Cancelled,
}

impl TeamInvitation {
    /// Create a new invitation
    pub fn new(
        team_id: Uuid,
        email: String,
        role: TeamRole,
        token: String,
        invited_by: Uuid,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            metadata: Metadata::new(),
            team_id,
            email,
            role,
            token,
            invited_by,
            expires_at,
            status: InvitationStatus::Pending,
        }
    }

    /// Check if invitation is expired
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }

    /// Accept invitation
    pub fn accept(&mut self) {
        self.status = InvitationStatus::Accepted;
        self.metadata.touch();
    }

    /// Decline invitation
    pub fn decline(&mut self) {
        self.status = InvitationStatus::Declined;
        self.metadata.touch();
    }

    /// Cancel invitation
    pub fn cancel(&mut self) {
        self.status = InvitationStatus::Cancelled;
        self.metadata.touch();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    fn create_test_invitation() -> TeamInvitation {
        TeamInvitation::new(
            Uuid::new_v4(),
            "test@example.com".to_string(),
            TeamRole::Member,
            "test_token_123".to_string(),
            Uuid::new_v4(),
            Utc::now() + Duration::days(7),
        )
    }

    // ==================== InvitationStatus Tests ====================

    #[test]
    fn test_invitation_status_pending() {
        let status = InvitationStatus::Pending;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"pending\"");
    }

    #[test]
    fn test_invitation_status_accepted() {
        let status = InvitationStatus::Accepted;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"accepted\"");
    }

    #[test]
    fn test_invitation_status_declined() {
        let status = InvitationStatus::Declined;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"declined\"");
    }

    #[test]
    fn test_invitation_status_expired() {
        let status = InvitationStatus::Expired;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"expired\"");
    }

    #[test]
    fn test_invitation_status_cancelled() {
        let status = InvitationStatus::Cancelled;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"cancelled\"");
    }

    #[test]
    fn test_invitation_status_deserialize() {
        let status: InvitationStatus = serde_json::from_str("\"pending\"").unwrap();
        assert!(matches!(status, InvitationStatus::Pending));

        let status: InvitationStatus = serde_json::from_str("\"accepted\"").unwrap();
        assert!(matches!(status, InvitationStatus::Accepted));
    }

    #[test]
    fn test_invitation_status_clone() {
        let original = InvitationStatus::Pending;
        let cloned = original.clone();
        let json1 = serde_json::to_string(&original).unwrap();
        let json2 = serde_json::to_string(&cloned).unwrap();
        assert_eq!(json1, json2);
    }

    // ==================== TeamInvitation Creation Tests ====================

    #[test]
    fn test_team_invitation_new() {
        let team_id = Uuid::new_v4();
        let invited_by = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::days(7);

        let invitation = TeamInvitation::new(
            team_id,
            "user@example.com".to_string(),
            TeamRole::Member,
            "token123".to_string(),
            invited_by,
            expires_at,
        );

        assert_eq!(invitation.team_id, team_id);
        assert_eq!(invitation.email, "user@example.com");
        assert!(matches!(invitation.role, TeamRole::Member));
        assert_eq!(invitation.token, "token123");
        assert_eq!(invitation.invited_by, invited_by);
        assert!(matches!(invitation.status, InvitationStatus::Pending));
    }

    #[test]
    fn test_team_invitation_new_with_admin_role() {
        let invitation = TeamInvitation::new(
            Uuid::new_v4(),
            "admin@example.com".to_string(),
            TeamRole::Admin,
            "admin_token".to_string(),
            Uuid::new_v4(),
            Utc::now() + Duration::days(3),
        );

        assert!(matches!(invitation.role, TeamRole::Admin));
    }

    #[test]
    fn test_team_invitation_new_with_viewer_role() {
        let invitation = TeamInvitation::new(
            Uuid::new_v4(),
            "viewer@example.com".to_string(),
            TeamRole::Viewer,
            "viewer_token".to_string(),
            Uuid::new_v4(),
            Utc::now() + Duration::days(1),
        );

        assert!(matches!(invitation.role, TeamRole::Viewer));
    }

    // ==================== TeamInvitation is_expired Tests ====================

    #[test]
    fn test_team_invitation_not_expired() {
        let invitation = TeamInvitation::new(
            Uuid::new_v4(),
            "test@example.com".to_string(),
            TeamRole::Member,
            "token".to_string(),
            Uuid::new_v4(),
            Utc::now() + Duration::days(7),
        );

        assert!(!invitation.is_expired());
    }

    #[test]
    fn test_team_invitation_expired() {
        let invitation = TeamInvitation::new(
            Uuid::new_v4(),
            "test@example.com".to_string(),
            TeamRole::Member,
            "token".to_string(),
            Uuid::new_v4(),
            Utc::now() - Duration::hours(1),
        );

        assert!(invitation.is_expired());
    }

    #[test]
    fn test_team_invitation_just_expired() {
        let invitation = TeamInvitation::new(
            Uuid::new_v4(),
            "test@example.com".to_string(),
            TeamRole::Member,
            "token".to_string(),
            Uuid::new_v4(),
            Utc::now() - Duration::seconds(1),
        );

        assert!(invitation.is_expired());
    }

    // ==================== TeamInvitation Actions Tests ====================

    #[test]
    fn test_team_invitation_accept() {
        let mut invitation = create_test_invitation();
        assert!(matches!(invitation.status, InvitationStatus::Pending));

        invitation.accept();

        assert!(matches!(invitation.status, InvitationStatus::Accepted));
    }

    #[test]
    fn test_team_invitation_decline() {
        let mut invitation = create_test_invitation();
        assert!(matches!(invitation.status, InvitationStatus::Pending));

        invitation.decline();

        assert!(matches!(invitation.status, InvitationStatus::Declined));
    }

    #[test]
    fn test_team_invitation_cancel() {
        let mut invitation = create_test_invitation();
        assert!(matches!(invitation.status, InvitationStatus::Pending));

        invitation.cancel();

        assert!(matches!(invitation.status, InvitationStatus::Cancelled));
    }

    // ==================== TeamInvitation Serialization Tests ====================

    #[test]
    fn test_team_invitation_serialize() {
        let invitation = create_test_invitation();

        let json = serde_json::to_string(&invitation).unwrap();

        // Token should NOT be serialized (skip_serializing)
        assert!(!json.contains("test_token_123"));
        // Email should be serialized
        assert!(json.contains("test@example.com"));
        // Status should be serialized
        assert!(json.contains("\"status\":\"pending\""));
    }

    #[test]
    fn test_team_invitation_serialize_after_accept() {
        let mut invitation = create_test_invitation();
        invitation.accept();

        let json = serde_json::to_string(&invitation).unwrap();
        assert!(json.contains("\"status\":\"accepted\""));
    }

    #[test]
    fn test_team_invitation_clone() {
        let invitation = create_test_invitation();
        let cloned = invitation.clone();

        assert_eq!(invitation.team_id, cloned.team_id);
        assert_eq!(invitation.email, cloned.email);
        assert_eq!(invitation.token, cloned.token);
    }

    #[test]
    fn test_team_invitation_debug() {
        let invitation = create_test_invitation();
        let debug_str = format!("{:?}", invitation);

        assert!(debug_str.contains("TeamInvitation"));
        assert!(debug_str.contains("test@example.com"));
    }

    // ==================== TeamInvitation Edge Cases ====================

    #[test]
    fn test_team_invitation_short_expiry() {
        let invitation = TeamInvitation::new(
            Uuid::new_v4(),
            "test@example.com".to_string(),
            TeamRole::Member,
            "token".to_string(),
            Uuid::new_v4(),
            Utc::now() + Duration::hours(1),
        );

        assert!(!invitation.is_expired());
    }

    #[test]
    fn test_team_invitation_long_expiry() {
        let invitation = TeamInvitation::new(
            Uuid::new_v4(),
            "test@example.com".to_string(),
            TeamRole::Member,
            "token".to_string(),
            Uuid::new_v4(),
            Utc::now() + Duration::days(30),
        );

        assert!(!invitation.is_expired());
    }

    #[test]
    fn test_team_invitation_various_emails() {
        let emails = vec![
            "simple@example.com",
            "user.name@domain.org",
            "user+tag@example.com",
            "test@subdomain.example.com",
        ];

        for email in emails {
            let invitation = TeamInvitation::new(
                Uuid::new_v4(),
                email.to_string(),
                TeamRole::Member,
                "token".to_string(),
                Uuid::new_v4(),
                Utc::now() + Duration::days(7),
            );

            assert_eq!(invitation.email, email);
        }
    }

    #[test]
    fn test_team_invitation_all_roles() {
        let roles = vec![
            TeamRole::Owner,
            TeamRole::Admin,
            TeamRole::Manager,
            TeamRole::Member,
            TeamRole::Viewer,
        ];

        for role in roles {
            let invitation = TeamInvitation::new(
                Uuid::new_v4(),
                "test@example.com".to_string(),
                role,
                "token".to_string(),
                Uuid::new_v4(),
                Utc::now() + Duration::days(7),
            );

            let json = serde_json::to_string(&invitation).unwrap();
            assert!(json.contains("role"));
        }
    }
}
