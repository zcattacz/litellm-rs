//! Team member models

use crate::core::models::Metadata;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Team member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    /// Member metadata
    #[serde(flatten)]
    pub metadata: Metadata,
    /// Team ID
    pub team_id: Uuid,
    /// User ID
    pub user_id: Uuid,
    /// Member role
    pub role: TeamRole,
    /// Member status
    pub status: MemberStatus,
    /// Joined at
    pub joined_at: chrono::DateTime<chrono::Utc>,
    /// Invited by
    pub invited_by: Option<Uuid>,
    /// Member permissions
    pub permissions: Vec<String>,
}

/// Team role
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TeamRole {
    /// Team owner
    Owner,
    /// Team admin
    Admin,
    /// Team manager
    Manager,
    /// Team member
    Member,
    /// Read-only member
    Viewer,
}

/// Member status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemberStatus {
    /// Active member
    Active,
    /// Pending invitation
    Pending,
    /// Suspended member
    Suspended,
    /// Left team
    Left,
}

impl TeamMember {
    /// Create a new team member
    pub fn new(team_id: Uuid, user_id: Uuid, role: TeamRole, invited_by: Option<Uuid>) -> Self {
        Self {
            metadata: Metadata::new(),
            team_id,
            user_id,
            role,
            status: MemberStatus::Active,
            joined_at: chrono::Utc::now(),
            invited_by,
            permissions: vec![],
        }
    }

    /// Check if member is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, MemberStatus::Active)
    }

    /// Check if member has permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }

    /// Add permission
    pub fn add_permission(&mut self, permission: String) {
        if !self.permissions.contains(&permission) {
            self.permissions.push(permission);
            self.metadata.touch();
        }
    }

    /// Remove permission
    pub fn remove_permission(&mut self, permission: &str) {
        if let Some(pos) = self.permissions.iter().position(|p| p == permission) {
            self.permissions.remove(pos);
            self.metadata.touch();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== TeamRole Tests ====================

    #[test]
    fn test_team_role_owner() {
        let role = TeamRole::Owner;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"owner\"");
    }

    #[test]
    fn test_team_role_admin() {
        let role = TeamRole::Admin;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"admin\"");
    }

    #[test]
    fn test_team_role_manager() {
        let role = TeamRole::Manager;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"manager\"");
    }

    #[test]
    fn test_team_role_member() {
        let role = TeamRole::Member;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"member\"");
    }

    #[test]
    fn test_team_role_viewer() {
        let role = TeamRole::Viewer;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"viewer\"");
    }

    #[test]
    fn test_team_role_deserialize() {
        let role: TeamRole = serde_json::from_str("\"owner\"").unwrap();
        assert!(matches!(role, TeamRole::Owner));

        let role: TeamRole = serde_json::from_str("\"viewer\"").unwrap();
        assert!(matches!(role, TeamRole::Viewer));
    }

    #[test]
    fn test_team_role_clone() {
        let original = TeamRole::Admin;
        let cloned = original.clone();
        let json1 = serde_json::to_string(&original).unwrap();
        let json2 = serde_json::to_string(&cloned).unwrap();
        assert_eq!(json1, json2);
    }

    // ==================== MemberStatus Tests ====================

    #[test]
    fn test_member_status_active() {
        let status = MemberStatus::Active;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"active\"");
    }

    #[test]
    fn test_member_status_pending() {
        let status = MemberStatus::Pending;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"pending\"");
    }

    #[test]
    fn test_member_status_suspended() {
        let status = MemberStatus::Suspended;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"suspended\"");
    }

    #[test]
    fn test_member_status_left() {
        let status = MemberStatus::Left;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"left\"");
    }

    #[test]
    fn test_member_status_deserialize() {
        let status: MemberStatus = serde_json::from_str("\"active\"").unwrap();
        assert!(matches!(status, MemberStatus::Active));

        let status: MemberStatus = serde_json::from_str("\"left\"").unwrap();
        assert!(matches!(status, MemberStatus::Left));
    }

    // ==================== TeamMember Creation Tests ====================

    #[test]
    fn test_team_member_new() {
        let team_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let invited_by = Uuid::new_v4();

        let member = TeamMember::new(team_id, user_id, TeamRole::Member, Some(invited_by));

        assert_eq!(member.team_id, team_id);
        assert_eq!(member.user_id, user_id);
        assert!(matches!(member.role, TeamRole::Member));
        assert!(matches!(member.status, MemberStatus::Active));
        assert_eq!(member.invited_by, Some(invited_by));
        assert!(member.permissions.is_empty());
    }

    #[test]
    fn test_team_member_new_owner() {
        let team_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let member = TeamMember::new(team_id, user_id, TeamRole::Owner, None);

        assert!(matches!(member.role, TeamRole::Owner));
        assert!(member.invited_by.is_none());
    }

    #[test]
    fn test_team_member_new_admin() {
        let team_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let invited_by = Uuid::new_v4();

        let member = TeamMember::new(team_id, user_id, TeamRole::Admin, Some(invited_by));

        assert!(matches!(member.role, TeamRole::Admin));
    }

    // ==================== TeamMember is_active Tests ====================

    #[test]
    fn test_team_member_is_active_when_active() {
        let member = TeamMember::new(Uuid::new_v4(), Uuid::new_v4(), TeamRole::Member, None);
        assert!(member.is_active());
    }

    #[test]
    fn test_team_member_is_active_when_pending() {
        let mut member = TeamMember::new(Uuid::new_v4(), Uuid::new_v4(), TeamRole::Member, None);
        member.status = MemberStatus::Pending;
        assert!(!member.is_active());
    }

    #[test]
    fn test_team_member_is_active_when_suspended() {
        let mut member = TeamMember::new(Uuid::new_v4(), Uuid::new_v4(), TeamRole::Member, None);
        member.status = MemberStatus::Suspended;
        assert!(!member.is_active());
    }

    #[test]
    fn test_team_member_is_active_when_left() {
        let mut member = TeamMember::new(Uuid::new_v4(), Uuid::new_v4(), TeamRole::Member, None);
        member.status = MemberStatus::Left;
        assert!(!member.is_active());
    }

    // ==================== TeamMember Permission Tests ====================

    #[test]
    fn test_team_member_has_permission_empty() {
        let member = TeamMember::new(Uuid::new_v4(), Uuid::new_v4(), TeamRole::Member, None);
        assert!(!member.has_permission("read"));
    }

    #[test]
    fn test_team_member_has_permission_exists() {
        let mut member = TeamMember::new(Uuid::new_v4(), Uuid::new_v4(), TeamRole::Member, None);
        member.permissions.push("read".to_string());

        assert!(member.has_permission("read"));
        assert!(!member.has_permission("write"));
    }

    #[test]
    fn test_team_member_add_permission() {
        let mut member = TeamMember::new(Uuid::new_v4(), Uuid::new_v4(), TeamRole::Member, None);

        member.add_permission("read".to_string());
        member.add_permission("write".to_string());

        assert_eq!(member.permissions.len(), 2);
        assert!(member.has_permission("read"));
        assert!(member.has_permission("write"));
    }

    #[test]
    fn test_team_member_add_permission_no_duplicate() {
        let mut member = TeamMember::new(Uuid::new_v4(), Uuid::new_v4(), TeamRole::Member, None);

        member.add_permission("read".to_string());
        member.add_permission("read".to_string());

        assert_eq!(member.permissions.len(), 1);
    }

    #[test]
    fn test_team_member_remove_permission() {
        let mut member = TeamMember::new(Uuid::new_v4(), Uuid::new_v4(), TeamRole::Member, None);
        member.add_permission("read".to_string());
        member.add_permission("write".to_string());

        member.remove_permission("read");

        assert_eq!(member.permissions.len(), 1);
        assert!(!member.has_permission("read"));
        assert!(member.has_permission("write"));
    }

    #[test]
    fn test_team_member_remove_permission_nonexistent() {
        let mut member = TeamMember::new(Uuid::new_v4(), Uuid::new_v4(), TeamRole::Member, None);
        member.add_permission("read".to_string());

        // Should not panic
        member.remove_permission("nonexistent");

        assert_eq!(member.permissions.len(), 1);
    }

    #[test]
    fn test_team_member_remove_all_permissions() {
        let mut member = TeamMember::new(Uuid::new_v4(), Uuid::new_v4(), TeamRole::Member, None);
        member.add_permission("read".to_string());
        member.add_permission("write".to_string());
        member.add_permission("delete".to_string());

        member.remove_permission("read");
        member.remove_permission("write");
        member.remove_permission("delete");

        assert!(member.permissions.is_empty());
    }

    // ==================== TeamMember Serialization Tests ====================

    #[test]
    fn test_team_member_serialize() {
        let member = TeamMember::new(Uuid::new_v4(), Uuid::new_v4(), TeamRole::Admin, None);

        let json = serde_json::to_string(&member).unwrap();

        assert!(json.contains("\"role\":\"admin\""));
        assert!(json.contains("\"status\":\"active\""));
    }

    #[test]
    fn test_team_member_clone() {
        let mut member = TeamMember::new(Uuid::new_v4(), Uuid::new_v4(), TeamRole::Manager, None);
        member.add_permission("read".to_string());

        let cloned = member.clone();

        assert_eq!(member.team_id, cloned.team_id);
        assert_eq!(member.user_id, cloned.user_id);
        assert_eq!(member.permissions.len(), cloned.permissions.len());
    }

    #[test]
    fn test_team_member_debug() {
        let member = TeamMember::new(Uuid::new_v4(), Uuid::new_v4(), TeamRole::Viewer, None);
        let debug_str = format!("{:?}", member);

        assert!(debug_str.contains("TeamMember"));
        assert!(debug_str.contains("Viewer"));
    }

    // ==================== TeamMember Edge Cases ====================

    #[test]
    fn test_team_member_many_permissions() {
        let mut member = TeamMember::new(Uuid::new_v4(), Uuid::new_v4(), TeamRole::Admin, None);

        for i in 0..100 {
            member.add_permission(format!("permission_{}", i));
        }

        assert_eq!(member.permissions.len(), 100);
        assert!(member.has_permission("permission_50"));
    }

    #[test]
    fn test_team_member_special_permission_names() {
        let mut member = TeamMember::new(Uuid::new_v4(), Uuid::new_v4(), TeamRole::Member, None);

        member.add_permission("api:read".to_string());
        member.add_permission("api:write".to_string());
        member.add_permission("admin.users.manage".to_string());

        assert!(member.has_permission("api:read"));
        assert!(member.has_permission("admin.users.manage"));
    }
}
