//! RBAC type definitions

use std::collections::HashSet;

/// Role definition
#[derive(Debug, Clone)]
pub struct Role {
    /// Role name
    pub name: String,
    /// Role description
    pub description: String,
    /// Permissions granted by this role
    pub permissions: HashSet<String>,
    /// Parent roles (inheritance)
    pub parent_roles: HashSet<String>,
    /// Whether this is a system role
    pub is_system: bool,
}

/// Permission definition
#[derive(Debug, Clone)]
pub struct Permission {
    /// Permission name
    pub name: String,
    /// Permission description
    pub description: String,
    /// Resource this permission applies to
    pub resource: String,
    /// Action this permission allows
    pub action: String,
    /// Whether this is a system permission
    pub is_system: bool,
}

/// Permission check result
#[derive(Debug, Clone)]
pub struct PermissionCheck {
    /// Whether permission is granted
    pub granted: bool,
    /// Roles that granted the permission
    pub granted_by_roles: Vec<String>,
    /// Reason for denial (if not granted)
    pub denial_reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Role Tests ====================

    #[test]
    fn test_role_creation() {
        let mut permissions = HashSet::new();
        permissions.insert("read:users".to_string());
        permissions.insert("write:users".to_string());

        let role = Role {
            name: "admin".to_string(),
            description: "Administrator role".to_string(),
            permissions,
            parent_roles: HashSet::new(),
            is_system: true,
        };

        assert_eq!(role.name, "admin");
        assert!(role.is_system);
        assert_eq!(role.permissions.len(), 2);
    }

    #[test]
    fn test_role_with_parent_roles() {
        let mut parent_roles = HashSet::new();
        parent_roles.insert("user".to_string());
        parent_roles.insert("viewer".to_string());

        let role = Role {
            name: "editor".to_string(),
            description: "Editor with viewer and user permissions".to_string(),
            permissions: HashSet::new(),
            parent_roles,
            is_system: false,
        };

        assert_eq!(role.parent_roles.len(), 2);
        assert!(role.parent_roles.contains("user"));
        assert!(role.parent_roles.contains("viewer"));
    }

    #[test]
    fn test_role_permission_check() {
        let mut permissions = HashSet::new();
        permissions.insert("read:api".to_string());
        permissions.insert("write:api".to_string());
        permissions.insert("delete:api".to_string());

        let role = Role {
            name: "api_manager".to_string(),
            description: "API management role".to_string(),
            permissions,
            parent_roles: HashSet::new(),
            is_system: false,
        };

        assert!(role.permissions.contains("read:api"));
        assert!(role.permissions.contains("write:api"));
        assert!(!role.permissions.contains("admin:api"));
    }

    #[test]
    fn test_role_clone() {
        let mut permissions = HashSet::new();
        permissions.insert("read".to_string());

        let role = Role {
            name: "reader".to_string(),
            description: "Read-only role".to_string(),
            permissions,
            parent_roles: HashSet::new(),
            is_system: false,
        };

        let cloned = role.clone();
        assert_eq!(cloned.name, role.name);
        assert_eq!(cloned.permissions.len(), role.permissions.len());
    }

    #[test]
    fn test_role_debug() {
        let role = Role {
            name: "test".to_string(),
            description: "Test role".to_string(),
            permissions: HashSet::new(),
            parent_roles: HashSet::new(),
            is_system: false,
        };

        let debug_str = format!("{:?}", role);
        assert!(debug_str.contains("Role"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_role_empty_permissions() {
        let role = Role {
            name: "empty".to_string(),
            description: "Role with no permissions".to_string(),
            permissions: HashSet::new(),
            parent_roles: HashSet::new(),
            is_system: false,
        };

        assert!(role.permissions.is_empty());
    }

    #[test]
    fn test_system_vs_user_role() {
        let system_role = Role {
            name: "super_admin".to_string(),
            description: "System administrator".to_string(),
            permissions: HashSet::new(),
            parent_roles: HashSet::new(),
            is_system: true,
        };

        let user_role = Role {
            name: "custom_role".to_string(),
            description: "User-defined role".to_string(),
            permissions: HashSet::new(),
            parent_roles: HashSet::new(),
            is_system: false,
        };

        assert!(system_role.is_system);
        assert!(!user_role.is_system);
    }

    // ==================== Permission Tests ====================

    #[test]
    fn test_permission_creation() {
        let permission = Permission {
            name: "read:users".to_string(),
            description: "Read user data".to_string(),
            resource: "users".to_string(),
            action: "read".to_string(),
            is_system: true,
        };

        assert_eq!(permission.name, "read:users");
        assert_eq!(permission.resource, "users");
        assert_eq!(permission.action, "read");
        assert!(permission.is_system);
    }

    #[test]
    fn test_permission_different_actions() {
        let actions = vec!["read", "write", "delete", "create", "update"];

        for action in actions {
            let permission = Permission {
                name: format!("{}:resource", action),
                description: format!("{} permission", action),
                resource: "resource".to_string(),
                action: action.to_string(),
                is_system: false,
            };

            assert_eq!(permission.action, action);
        }
    }

    #[test]
    fn test_permission_clone() {
        let permission = Permission {
            name: "write:api".to_string(),
            description: "Write API access".to_string(),
            resource: "api".to_string(),
            action: "write".to_string(),
            is_system: false,
        };

        let cloned = permission.clone();
        assert_eq!(cloned.name, permission.name);
        assert_eq!(cloned.resource, permission.resource);
    }

    #[test]
    fn test_permission_debug() {
        let permission = Permission {
            name: "test:perm".to_string(),
            description: "Test permission".to_string(),
            resource: "test".to_string(),
            action: "test".to_string(),
            is_system: false,
        };

        let debug_str = format!("{:?}", permission);
        assert!(debug_str.contains("Permission"));
        assert!(debug_str.contains("test:perm"));
    }

    #[test]
    fn test_permission_resources() {
        let resources = vec!["users", "teams", "api_keys", "models", "billing"];

        for resource in resources {
            let permission = Permission {
                name: format!("manage:{}", resource),
                description: format!("Manage {}", resource),
                resource: resource.to_string(),
                action: "manage".to_string(),
                is_system: false,
            };

            assert_eq!(permission.resource, resource);
        }
    }

    // ==================== PermissionCheck Tests ====================

    #[test]
    fn test_permission_check_granted() {
        let check = PermissionCheck {
            granted: true,
            granted_by_roles: vec!["admin".to_string(), "manager".to_string()],
            denial_reason: None,
        };

        assert!(check.granted);
        assert_eq!(check.granted_by_roles.len(), 2);
        assert!(check.denial_reason.is_none());
    }

    #[test]
    fn test_permission_check_denied() {
        let check = PermissionCheck {
            granted: false,
            granted_by_roles: vec![],
            denial_reason: Some("Insufficient permissions".to_string()),
        };

        assert!(!check.granted);
        assert!(check.granted_by_roles.is_empty());
        assert_eq!(check.denial_reason.as_ref().unwrap(), "Insufficient permissions");
    }

    #[test]
    fn test_permission_check_single_role() {
        let check = PermissionCheck {
            granted: true,
            granted_by_roles: vec!["user".to_string()],
            denial_reason: None,
        };

        assert_eq!(check.granted_by_roles.len(), 1);
        assert_eq!(check.granted_by_roles[0], "user");
    }

    #[test]
    fn test_permission_check_clone() {
        let check = PermissionCheck {
            granted: true,
            granted_by_roles: vec!["admin".to_string()],
            denial_reason: None,
        };

        let cloned = check.clone();
        assert_eq!(cloned.granted, check.granted);
        assert_eq!(cloned.granted_by_roles, check.granted_by_roles);
    }

    #[test]
    fn test_permission_check_debug() {
        let check = PermissionCheck {
            granted: false,
            granted_by_roles: vec![],
            denial_reason: Some("Access denied".to_string()),
        };

        let debug_str = format!("{:?}", check);
        assert!(debug_str.contains("PermissionCheck"));
        assert!(debug_str.contains("false"));
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_role_permission_matching() {
        let mut permissions = HashSet::new();
        permissions.insert("read:users".to_string());
        permissions.insert("write:users".to_string());

        let role = Role {
            name: "user_manager".to_string(),
            description: "User management role".to_string(),
            permissions,
            parent_roles: HashSet::new(),
            is_system: false,
        };

        let required_permission = "read:users";
        let has_permission = role.permissions.contains(required_permission);

        let check = PermissionCheck {
            granted: has_permission,
            granted_by_roles: if has_permission {
                vec![role.name.clone()]
            } else {
                vec![]
            },
            denial_reason: if has_permission {
                None
            } else {
                Some("Permission not found".to_string())
            },
        };

        assert!(check.granted);
        assert_eq!(check.granted_by_roles[0], "user_manager");
    }

    #[test]
    fn test_role_hierarchy() {
        // Create base role
        let mut viewer_perms = HashSet::new();
        viewer_perms.insert("read:*".to_string());

        let viewer = Role {
            name: "viewer".to_string(),
            description: "Read-only access".to_string(),
            permissions: viewer_perms,
            parent_roles: HashSet::new(),
            is_system: true,
        };

        // Create editor that inherits from viewer
        let mut editor_parents = HashSet::new();
        editor_parents.insert("viewer".to_string());

        let mut editor_perms = HashSet::new();
        editor_perms.insert("write:*".to_string());

        let editor = Role {
            name: "editor".to_string(),
            description: "Can edit content".to_string(),
            permissions: editor_perms,
            parent_roles: editor_parents,
            is_system: true,
        };

        assert!(editor.parent_roles.contains(&viewer.name));
        assert!(editor.permissions.contains("write:*"));
    }

    #[test]
    fn test_permission_check_multiple_roles() {
        let check = PermissionCheck {
            granted: true,
            granted_by_roles: vec![
                "admin".to_string(),
                "manager".to_string(),
                "user".to_string(),
            ],
            denial_reason: None,
        };

        assert!(check.granted_by_roles.contains(&"admin".to_string()));
        assert!(check.granted_by_roles.contains(&"manager".to_string()));
        assert!(check.granted_by_roles.contains(&"user".to_string()));
    }

    #[test]
    fn test_denial_reasons() {
        let reasons = vec![
            "Insufficient permissions",
            "Role not assigned",
            "Permission expired",
            "Resource not found",
            "Action not allowed",
        ];

        for reason in reasons {
            let check = PermissionCheck {
                granted: false,
                granted_by_roles: vec![],
                denial_reason: Some(reason.to_string()),
            };

            assert!(!check.granted);
            assert_eq!(check.denial_reason.as_ref().unwrap(), reason);
        }
    }
}
