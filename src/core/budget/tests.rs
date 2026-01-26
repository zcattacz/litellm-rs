//! Integration tests for the budget management system

use super::*;
use std::sync::Arc;

#[tokio::test]
async fn test_budget_system_initialization() {
    let (manager, alert_manager) = init_budget_system();

    assert_eq!(manager.budget_count(), 0);
    assert!(alert_manager.is_enabled().await);
}

#[tokio::test]
async fn test_budget_system_with_custom_config() {
    let manager_config = BudgetManagerConfig {
        enabled: true,
        default_soft_limit_percentage: 0.75,
        block_on_exceeded: true,
        auto_reset_enabled: false,
        reset_check_interval_secs: 300,
    };

    let alert_config = AlertConfig {
        enabled: true,
        soft_limit_percentage: 0.75,
        warning_thresholds: vec![0.9],
        max_history_size: 500,
        duplicate_suppression_secs: 1800,
    };

    let (manager, alert_manager) = init_budget_system_with_config(manager_config, alert_config);

    assert!(manager.is_enabled().await);

    let config = alert_manager.get_config().await;
    assert_eq!(config.soft_limit_percentage, 0.75);
}

#[tokio::test]
async fn test_end_to_end_budget_workflow() {
    let manager = Arc::new(BudgetManager::new());
    let alert_manager = Arc::new(BudgetAlertManager::new());

    // Create a user budget
    let config = BudgetConfig::new("User 1 Budget", 100.0)
        .with_reset_period(ResetPeriod::Monthly)
        .with_currency(Currency::USD);

    let budget = manager
        .create_budget(BudgetScope::User("user-1".to_string()), config)
        .await
        .unwrap();

    assert_eq!(budget.name, "User 1 Budget");
    assert_eq!(budget.max_budget, 100.0);
    assert_eq!(budget.soft_limit, 80.0);
    assert_eq!(budget.status(), BudgetStatus::Ok);

    // Record some spending
    let result1 = manager
        .record_spend(&BudgetScope::User("user-1".to_string()), 30.0)
        .await
        .unwrap();

    assert_eq!(result1.current_spend, 30.0);
    assert_eq!(result1.new_status, BudgetStatus::Ok);
    assert!(!result1.should_alert_soft_limit);

    // Record more spending to trigger soft limit
    let result2 = manager
        .record_spend(&BudgetScope::User("user-1".to_string()), 51.0)
        .await
        .unwrap();

    assert_eq!(result2.current_spend, 81.0);
    assert_eq!(result2.new_status, BudgetStatus::Warning);
    assert!(result2.should_alert_soft_limit);

    // Process the alert
    let budget_now = manager
        .get_budget(&BudgetScope::User("user-1".to_string()))
        .unwrap();
    alert_manager
        .process_spend_result(&result2, &budget_now)
        .await;

    let alerts = alert_manager.get_all_alerts().await;
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].alert_type, BudgetAlertType::SoftLimitReached);

    // Record more to exceed budget
    let result3 = manager
        .record_spend(&BudgetScope::User("user-1".to_string()), 20.0)
        .await
        .unwrap();

    assert_eq!(result3.current_spend, 101.0);
    assert_eq!(result3.new_status, BudgetStatus::Exceeded);
    assert!(result3.should_alert_exceeded);

    // Check budget should now be blocked
    let check_result = manager
        .check_spend(&BudgetScope::User("user-1".to_string()), 1.0)
        .await;
    assert!(!check_result.allowed);

    // Reset the budget
    manager
        .reset_budget(&BudgetScope::User("user-1".to_string()))
        .await
        .unwrap();

    let after_reset = manager
        .get_budget(&BudgetScope::User("user-1".to_string()))
        .unwrap();
    assert_eq!(after_reset.current_spend, 0.0);
    assert_eq!(after_reset.status(), BudgetStatus::Ok);
}

#[tokio::test]
async fn test_multiple_budget_scopes() {
    let manager = BudgetManager::new();

    // Create budgets for different scopes
    manager
        .create_budget(BudgetScope::Global, BudgetConfig::new("Global", 10000.0))
        .await
        .unwrap();

    manager
        .create_budget(
            BudgetScope::Team("team-1".to_string()),
            BudgetConfig::new("Team 1", 1000.0),
        )
        .await
        .unwrap();

    manager
        .create_budget(
            BudgetScope::User("user-1".to_string()),
            BudgetConfig::new("User 1", 100.0),
        )
        .await
        .unwrap();

    manager
        .create_budget(
            BudgetScope::ApiKey("sk-123...".to_string()),
            BudgetConfig::new("API Key 1", 50.0),
        )
        .await
        .unwrap();

    manager
        .create_budget(
            BudgetScope::Provider("openai".to_string()),
            BudgetConfig::new("OpenAI Provider", 5000.0),
        )
        .await
        .unwrap();

    manager
        .create_budget(
            BudgetScope::Model("gpt-4".to_string()),
            BudgetConfig::new("GPT-4 Model", 2000.0),
        )
        .await
        .unwrap();

    assert_eq!(manager.budget_count(), 6);

    // List by type
    let user_budgets = manager.list_budgets_filtered(Some("user"), None);
    assert_eq!(user_budgets.len(), 1);

    let provider_budgets = manager.list_budgets_filtered(Some("provider"), None);
    assert_eq!(provider_budgets.len(), 1);
}

#[tokio::test]
async fn test_budget_recorder() {
    let manager = Arc::new(BudgetManager::new());

    // Create budgets
    manager
        .create_budget(BudgetScope::Global, BudgetConfig::new("Global", 10000.0))
        .await
        .unwrap();

    manager
        .create_budget(
            BudgetScope::User("user-1".to_string()),
            BudgetConfig::new("User 1", 100.0),
        )
        .await
        .unwrap();

    manager
        .create_budget(
            BudgetScope::Model("gpt-4".to_string()),
            BudgetConfig::new("GPT-4", 1000.0),
        )
        .await
        .unwrap();

    let recorder = BudgetRecorder::new(Arc::clone(&manager));

    // Simulate a request that uses GPT-4
    recorder
        .record_request_spend(Some("user-1"), None, None, Some("gpt-4"), None, 0.05)
        .await;

    // Check all scopes were updated
    assert_eq!(manager.get_current_spend(&BudgetScope::Global), 0.05);
    assert_eq!(
        manager.get_current_spend(&BudgetScope::User("user-1".to_string())),
        0.05
    );
    assert_eq!(
        manager.get_current_spend(&BudgetScope::Model("gpt-4".to_string())),
        0.05
    );
}

#[tokio::test]
async fn test_budget_summary() {
    let manager = BudgetManager::new();

    manager
        .create_budget(
            BudgetScope::User("user-1".to_string()),
            BudgetConfig::new("User 1", 100.0),
        )
        .await
        .unwrap();

    manager
        .create_budget(
            BudgetScope::User("user-2".to_string()),
            BudgetConfig::new("User 2", 100.0),
        )
        .await
        .unwrap();

    manager
        .create_budget(
            BudgetScope::User("user-3".to_string()),
            BudgetConfig::new("User 3", 100.0),
        )
        .await
        .unwrap();

    // Record different spend amounts
    manager
        .record_spend(&BudgetScope::User("user-1".to_string()), 20.0)
        .await;
    manager
        .record_spend(&BudgetScope::User("user-2".to_string()), 85.0)
        .await; // Warning
    manager
        .record_spend(&BudgetScope::User("user-3".to_string()), 110.0)
        .await; // Exceeded

    let summary = manager.get_summary();

    assert_eq!(summary.total_budgets, 3);
    assert_eq!(summary.total_allocated, 300.0);
    assert_eq!(summary.total_spent, 215.0);
    assert_eq!(summary.total_remaining, 85.0);
    assert_eq!(summary.ok_count, 1);
    assert_eq!(summary.warning_count, 1);
    assert_eq!(summary.exceeded_count, 1);
}

#[tokio::test]
async fn test_alert_workflow() {
    let manager = Arc::new(BudgetManager::new());
    let alert_manager = Arc::new(BudgetAlertManager::new());

    manager
        .create_budget(BudgetScope::Global, BudgetConfig::new("Global", 100.0))
        .await
        .unwrap();

    // Record spend that triggers soft limit
    let result = manager
        .record_spend(&BudgetScope::Global, 85.0)
        .await
        .unwrap();

    let budget = manager.get_budget(&BudgetScope::Global).unwrap();
    alert_manager.process_spend_result(&result, &budget).await;

    let stats = alert_manager.get_alert_stats().await;
    assert_eq!(stats.warning_count, 1);
    assert_eq!(stats.soft_limit_alerts, 1);
    assert_eq!(stats.unacknowledged, 1);

    // Acknowledge all alerts
    let alerts = alert_manager.get_all_alerts().await;
    for alert in &alerts {
        alert_manager.acknowledge_alert(&alert.id).await;
    }

    let stats_after = alert_manager.get_alert_stats().await;
    assert_eq!(stats_after.unacknowledged, 0);
}

#[tokio::test]
async fn test_concurrent_budget_operations() {
    use std::sync::Arc;
    use tokio::task;

    let manager = Arc::new(BudgetManager::new());

    manager
        .create_budget(BudgetScope::Global, BudgetConfig::new("Global", 10000.0))
        .await
        .unwrap();

    let mut handles = vec![];

    // Spawn multiple tasks to record spend concurrently
    for _i in 0..10 {
        let manager_clone = Arc::clone(&manager);
        let handle = task::spawn(async move {
            for _j in 0..100 {
                manager_clone.record_spend(&BudgetScope::Global, 1.0).await;
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    // Should have recorded 1000 spends of 1.0 each
    let spend = manager.get_current_spend(&BudgetScope::Global);
    assert_eq!(spend, 1000.0);
}

#[tokio::test]
async fn test_budget_scope_parsing() {
    // Test scope to key and back
    let scopes = vec![
        BudgetScope::User("user-123".to_string()),
        BudgetScope::Team("team-456".to_string()),
        BudgetScope::ApiKey("sk-abc123".to_string()),
        BudgetScope::Provider("openai".to_string()),
        BudgetScope::Model("gpt-4-turbo".to_string()),
        BudgetScope::Global,
    ];

    for scope in scopes {
        let key = scope.to_key();
        let parsed = BudgetScope::from_key(&key);
        assert_eq!(parsed, Some(scope));
    }
}

#[tokio::test]
async fn test_budget_disabled() {
    let config = BudgetManagerConfig {
        enabled: false,
        ..Default::default()
    };

    let manager = BudgetManager::with_config(config);

    manager
        .create_budget(BudgetScope::Global, BudgetConfig::new("Global", 100.0))
        .await
        .unwrap();

    // When disabled, check_spend should always return allowed
    manager.record_spend(&BudgetScope::Global, 150.0).await;

    let result = manager.check_spend(&BudgetScope::Global, 50.0).await;
    assert!(result.allowed); // Allowed because manager is disabled
}

#[tokio::test]
async fn test_budget_without_blocking() {
    let config = BudgetManagerConfig {
        block_on_exceeded: false,
        ..Default::default()
    };

    let manager = BudgetManager::with_config(config);

    manager
        .create_budget(BudgetScope::Global, BudgetConfig::new("Global", 100.0))
        .await
        .unwrap();

    manager.record_spend(&BudgetScope::Global, 150.0).await;

    // Should still be allowed even though exceeded
    let result = manager.check_spend(&BudgetScope::Global, 50.0).await;
    assert!(result.allowed);
    assert_eq!(result.status, BudgetStatus::Exceeded);
}
