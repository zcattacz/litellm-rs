//! Deployment information for tag/group-based routing
//!
//! **DEPRECATED**: This module is part of the legacy load balancer system.
//! For new code, use `crate::core::router::Deployment` which has built-in
//! tag support and more sophisticated health tracking.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Deployment information for tag/group-based routing
///
/// **DEPRECATED**: Use `crate::core::router::Deployment` for new code.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeploymentInfo {
    /// Tags for this deployment (e.g., ["fast", "high-quality", "cost-effective"])
    #[serde(default)]
    pub tags: Vec<String>,
    /// Model group this deployment belongs to (e.g., "gpt-4-group")
    #[serde(default)]
    pub model_group: Option<String>,
    /// Priority within the group (lower = higher priority)
    #[serde(default)]
    pub priority: u32,
    /// Custom metadata for this deployment
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl DeploymentInfo {
    /// Create new deployment info
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add multiple tags
    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags.extend(tags.into_iter().map(|t| t.into()));
        self
    }

    /// Set model group
    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.model_group = Some(group.into());
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Check if deployment has all specified tags
    pub fn has_all_tags(&self, required_tags: &[String]) -> bool {
        required_tags.iter().all(|tag| self.tags.contains(tag))
    }

    /// Check if deployment has any of the specified tags
    pub fn has_any_tag(&self, tags: &[String]) -> bool {
        tags.iter().any(|tag| self.tags.contains(tag))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Default Tests ====================

    #[test]
    fn test_deployment_info_default() {
        let info = DeploymentInfo::default();
        assert!(info.tags.is_empty());
        assert!(info.model_group.is_none());
        assert_eq!(info.priority, 0);
        assert!(info.metadata.is_empty());
    }

    #[test]
    fn test_deployment_info_new() {
        let info = DeploymentInfo::new();
        assert!(info.tags.is_empty());
        assert!(info.model_group.is_none());
    }

    // ==================== Builder Pattern Tests ====================

    #[test]
    fn test_with_tag() {
        let info = DeploymentInfo::new().with_tag("fast");
        assert_eq!(info.tags.len(), 1);
        assert!(info.tags.contains(&"fast".to_string()));
    }

    #[test]
    fn test_with_tag_string_owned() {
        let info = DeploymentInfo::new().with_tag(String::from("high-quality"));
        assert!(info.tags.contains(&"high-quality".to_string()));
    }

    #[test]
    fn test_with_multiple_tags_chained() {
        let info = DeploymentInfo::new()
            .with_tag("fast")
            .with_tag("cheap")
            .with_tag("reliable");

        assert_eq!(info.tags.len(), 3);
        assert!(info.tags.contains(&"fast".to_string()));
        assert!(info.tags.contains(&"cheap".to_string()));
        assert!(info.tags.contains(&"reliable".to_string()));
    }

    #[test]
    fn test_with_tags_array() {
        let info = DeploymentInfo::new().with_tags(["tag1", "tag2", "tag3"]);
        assert_eq!(info.tags.len(), 3);
    }

    #[test]
    fn test_with_tags_vec() {
        let tags = vec!["a".to_string(), "b".to_string()];
        let info = DeploymentInfo::new().with_tags(tags);
        assert_eq!(info.tags.len(), 2);
    }

    #[test]
    fn test_with_tags_iterator() {
        let info = DeploymentInfo::new().with_tags(["x", "y", "z"].iter().cloned());
        assert_eq!(info.tags.len(), 3);
    }

    #[test]
    fn test_with_group() {
        let info = DeploymentInfo::new().with_group("gpt-4-group");
        assert_eq!(info.model_group, Some("gpt-4-group".to_string()));
    }

    #[test]
    fn test_with_group_string_owned() {
        let info = DeploymentInfo::new().with_group(String::from("claude-group"));
        assert_eq!(info.model_group, Some("claude-group".to_string()));
    }

    #[test]
    fn test_with_priority() {
        let info = DeploymentInfo::new().with_priority(10);
        assert_eq!(info.priority, 10);
    }

    #[test]
    fn test_with_priority_zero() {
        let info = DeploymentInfo::new().with_priority(0);
        assert_eq!(info.priority, 0);
    }

    #[test]
    fn test_with_metadata() {
        let info = DeploymentInfo::new().with_metadata("version", serde_json::json!("1.0"));
        assert_eq!(
            info.metadata.get("version"),
            Some(&serde_json::json!("1.0"))
        );
    }

    #[test]
    fn test_with_metadata_complex_value() {
        let info = DeploymentInfo::new()
            .with_metadata("config", serde_json::json!({"enabled": true, "rate": 100}));
        let config = info.metadata.get("config").unwrap();
        assert_eq!(config["enabled"], true);
        assert_eq!(config["rate"], 100);
    }

    #[test]
    fn test_with_metadata_multiple() {
        let info = DeploymentInfo::new()
            .with_metadata("key1", serde_json::json!("value1"))
            .with_metadata("key2", serde_json::json!(42));

        assert_eq!(info.metadata.len(), 2);
    }

    // ==================== Chaining Tests ====================

    #[test]
    fn test_full_builder_chain() {
        let info = DeploymentInfo::new()
            .with_tag("fast")
            .with_tags(["reliable", "cheap"])
            .with_group("production")
            .with_priority(5)
            .with_metadata("region", serde_json::json!("us-east-1"));

        assert_eq!(info.tags.len(), 3);
        assert_eq!(info.model_group, Some("production".to_string()));
        assert_eq!(info.priority, 5);
        assert_eq!(info.metadata.len(), 1);
    }

    // ==================== has_all_tags Tests ====================

    #[test]
    fn test_has_all_tags_true() {
        let info = DeploymentInfo::new().with_tags(["fast", "cheap", "reliable"]);

        let required = vec!["fast".to_string(), "cheap".to_string()];
        assert!(info.has_all_tags(&required));
    }

    #[test]
    fn test_has_all_tags_false() {
        let info = DeploymentInfo::new().with_tags(["fast", "cheap"]);

        let required = vec!["fast".to_string(), "expensive".to_string()];
        assert!(!info.has_all_tags(&required));
    }

    #[test]
    fn test_has_all_tags_empty_required() {
        let info = DeploymentInfo::new().with_tag("fast");

        let required: Vec<String> = vec![];
        assert!(info.has_all_tags(&required)); // Empty requirement is always satisfied
    }

    #[test]
    fn test_has_all_tags_empty_deployment() {
        let info = DeploymentInfo::new();

        let required = vec!["fast".to_string()];
        assert!(!info.has_all_tags(&required));
    }

    #[test]
    fn test_has_all_tags_exact_match() {
        let info = DeploymentInfo::new().with_tags(["a", "b"]);

        let required = vec!["a".to_string(), "b".to_string()];
        assert!(info.has_all_tags(&required));
    }

    // ==================== has_any_tag Tests ====================

    #[test]
    fn test_has_any_tag_true() {
        let info = DeploymentInfo::new().with_tags(["fast", "cheap"]);

        let tags = vec!["fast".to_string(), "expensive".to_string()];
        assert!(info.has_any_tag(&tags));
    }

    #[test]
    fn test_has_any_tag_false() {
        let info = DeploymentInfo::new().with_tags(["fast", "cheap"]);

        let tags = vec!["slow".to_string(), "expensive".to_string()];
        assert!(!info.has_any_tag(&tags));
    }

    #[test]
    fn test_has_any_tag_empty_tags() {
        let info = DeploymentInfo::new().with_tag("fast");

        let tags: Vec<String> = vec![];
        assert!(!info.has_any_tag(&tags)); // Empty list has no matches
    }

    #[test]
    fn test_has_any_tag_empty_deployment() {
        let info = DeploymentInfo::new();

        let tags = vec!["fast".to_string()];
        assert!(!info.has_any_tag(&tags));
    }

    // ==================== Serialization Tests ====================

    #[test]
    fn test_deployment_info_serialization() {
        let info = DeploymentInfo::new()
            .with_tag("fast")
            .with_group("prod")
            .with_priority(3);

        let json = serde_json::to_value(&info).unwrap();
        assert!(json["tags"].is_array());
        assert_eq!(json["model_group"], "prod");
        assert_eq!(json["priority"], 3);
    }

    #[test]
    fn test_deployment_info_deserialization() {
        let json = r#"{
            "tags": ["fast", "cheap"],
            "model_group": "test-group",
            "priority": 5,
            "metadata": {"key": "value"}
        }"#;

        let info: DeploymentInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.tags.len(), 2);
        assert_eq!(info.model_group, Some("test-group".to_string()));
        assert_eq!(info.priority, 5);
    }

    #[test]
    fn test_deployment_info_deserialization_missing_fields() {
        let json = r#"{}"#;
        let info: DeploymentInfo = serde_json::from_str(json).unwrap();

        assert!(info.tags.is_empty());
        assert!(info.model_group.is_none());
        assert_eq!(info.priority, 0);
    }

    #[test]
    fn test_deployment_info_roundtrip() {
        let original = DeploymentInfo::new()
            .with_tags(["a", "b"])
            .with_group("group")
            .with_priority(10)
            .with_metadata("key", serde_json::json!("value"));

        let json = serde_json::to_string(&original).unwrap();
        let restored: DeploymentInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(original.tags, restored.tags);
        assert_eq!(original.model_group, restored.model_group);
        assert_eq!(original.priority, restored.priority);
    }

    // ==================== Clone Tests ====================

    #[test]
    fn test_deployment_info_clone() {
        let original = DeploymentInfo::new().with_tag("fast").with_group("prod");

        let cloned = original.clone();
        assert_eq!(original.tags, cloned.tags);
        assert_eq!(original.model_group, cloned.model_group);
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_duplicate_tags() {
        let info = DeploymentInfo::new().with_tag("fast").with_tag("fast");

        assert_eq!(info.tags.len(), 2); // Duplicates are allowed
    }

    #[test]
    fn test_empty_tag() {
        let info = DeploymentInfo::new().with_tag("");
        assert!(info.tags.contains(&"".to_string()));
    }

    #[test]
    fn test_high_priority_value() {
        let info = DeploymentInfo::new().with_priority(u32::MAX);
        assert_eq!(info.priority, u32::MAX);
    }
}
