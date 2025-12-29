//! Team billing models

use serde::{Deserialize, Serialize};

/// Team billing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamBilling {
    /// Billing plan
    pub plan: BillingPlan,
    /// Billing status
    pub status: BillingStatus,
    /// Monthly budget limit
    pub monthly_budget: Option<f64>,
    /// Current month usage
    pub current_usage: f64,
    /// Billing cycle start
    pub cycle_start: chrono::DateTime<chrono::Utc>,
    /// Billing cycle end
    pub cycle_end: chrono::DateTime<chrono::Utc>,
    /// Payment method
    pub payment_method: Option<PaymentMethod>,
    /// Billing address
    pub billing_address: Option<BillingAddress>,
}

/// Billing plan
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingPlan {
    /// Free plan
    Free,
    /// Starter plan
    Starter,
    /// Professional plan
    Professional,
    /// Enterprise plan
    Enterprise,
    /// Custom plan
    Custom,
}

/// Billing status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingStatus {
    /// Active billing status
    Active,
    /// Past due billing status
    PastDue,
    /// Cancelled billing status
    Cancelled,
    /// Suspended billing status
    Suspended,
    /// Trial billing status
    Trial,
}

/// Payment method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMethod {
    /// Payment method type
    pub method_type: PaymentMethodType,
    /// Last 4 digits (for cards)
    pub last_four: Option<String>,
    /// Expiry month (for cards)
    pub expiry_month: Option<u32>,
    /// Expiry year (for cards)
    pub expiry_year: Option<u32>,
    /// Brand (for cards)
    pub brand: Option<String>,
}

/// Payment method type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethodType {
    /// Credit card payment
    CreditCard,
    /// Debit card payment
    DebitCard,
    /// Bank transfer payment
    BankTransfer,
    /// PayPal payment
    PayPal,
    /// Stripe payment
    Stripe,
}

/// Billing address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingAddress {
    /// Company name
    pub company: Option<String>,
    /// Address line 1
    pub line1: String,
    /// Address line 2
    pub line2: Option<String>,
    /// City
    pub city: String,
    /// State/Province
    pub state: Option<String>,
    /// Postal code
    pub postal_code: String,
    /// Country
    pub country: String,
    /// Tax ID
    pub tax_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    // ==================== BillingPlan Tests ====================

    #[test]
    fn test_billing_plan_free() {
        let plan = BillingPlan::Free;
        let json = serde_json::to_string(&plan).unwrap();
        assert_eq!(json, "\"free\"");
    }

    #[test]
    fn test_billing_plan_starter() {
        let plan = BillingPlan::Starter;
        let json = serde_json::to_string(&plan).unwrap();
        assert_eq!(json, "\"starter\"");
    }

    #[test]
    fn test_billing_plan_professional() {
        let plan = BillingPlan::Professional;
        let json = serde_json::to_string(&plan).unwrap();
        assert_eq!(json, "\"professional\"");
    }

    #[test]
    fn test_billing_plan_enterprise() {
        let plan = BillingPlan::Enterprise;
        let json = serde_json::to_string(&plan).unwrap();
        assert_eq!(json, "\"enterprise\"");
    }

    #[test]
    fn test_billing_plan_custom() {
        let plan = BillingPlan::Custom;
        let json = serde_json::to_string(&plan).unwrap();
        assert_eq!(json, "\"custom\"");
    }

    #[test]
    fn test_billing_plan_deserialize() {
        let plan: BillingPlan = serde_json::from_str("\"enterprise\"").unwrap();
        assert!(matches!(plan, BillingPlan::Enterprise));
    }

    #[test]
    fn test_billing_plan_clone() {
        let original = BillingPlan::Professional;
        let cloned = original.clone();
        let json1 = serde_json::to_string(&original).unwrap();
        let json2 = serde_json::to_string(&cloned).unwrap();
        assert_eq!(json1, json2);
    }

    // ==================== BillingStatus Tests ====================

    #[test]
    fn test_billing_status_active() {
        let status = BillingStatus::Active;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"active\"");
    }

    #[test]
    fn test_billing_status_past_due() {
        let status = BillingStatus::PastDue;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"past_due\"");
    }

    #[test]
    fn test_billing_status_cancelled() {
        let status = BillingStatus::Cancelled;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"cancelled\"");
    }

    #[test]
    fn test_billing_status_suspended() {
        let status = BillingStatus::Suspended;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"suspended\"");
    }

    #[test]
    fn test_billing_status_trial() {
        let status = BillingStatus::Trial;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"trial\"");
    }

    #[test]
    fn test_billing_status_deserialize() {
        let status: BillingStatus = serde_json::from_str("\"past_due\"").unwrap();
        assert!(matches!(status, BillingStatus::PastDue));
    }

    // ==================== PaymentMethodType Tests ====================

    #[test]
    fn test_payment_method_type_credit_card() {
        let t = PaymentMethodType::CreditCard;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"credit_card\"");
    }

    #[test]
    fn test_payment_method_type_debit_card() {
        let t = PaymentMethodType::DebitCard;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"debit_card\"");
    }

    #[test]
    fn test_payment_method_type_bank_transfer() {
        let t = PaymentMethodType::BankTransfer;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"bank_transfer\"");
    }

    #[test]
    fn test_payment_method_type_paypal() {
        let t = PaymentMethodType::PayPal;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"pay_pal\"");
    }

    #[test]
    fn test_payment_method_type_stripe() {
        let t = PaymentMethodType::Stripe;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"stripe\"");
    }

    // ==================== PaymentMethod Tests ====================

    #[test]
    fn test_payment_method_credit_card() {
        let pm = PaymentMethod {
            method_type: PaymentMethodType::CreditCard,
            last_four: Some("4242".to_string()),
            expiry_month: Some(12),
            expiry_year: Some(2025),
            brand: Some("Visa".to_string()),
        };

        assert_eq!(pm.last_four, Some("4242".to_string()));
        assert_eq!(pm.brand, Some("Visa".to_string()));
    }

    #[test]
    fn test_payment_method_bank_transfer() {
        let pm = PaymentMethod {
            method_type: PaymentMethodType::BankTransfer,
            last_four: None,
            expiry_month: None,
            expiry_year: None,
            brand: None,
        };

        assert!(matches!(pm.method_type, PaymentMethodType::BankTransfer));
        assert!(pm.last_four.is_none());
    }

    #[test]
    fn test_payment_method_serialize() {
        let pm = PaymentMethod {
            method_type: PaymentMethodType::CreditCard,
            last_four: Some("1234".to_string()),
            expiry_month: Some(6),
            expiry_year: Some(2026),
            brand: Some("MasterCard".to_string()),
        };

        let json = serde_json::to_string(&pm).unwrap();
        assert!(json.contains("credit_card"));
        assert!(json.contains("1234"));
        assert!(json.contains("MasterCard"));
    }

    #[test]
    fn test_payment_method_clone() {
        let original = PaymentMethod {
            method_type: PaymentMethodType::Stripe,
            last_four: Some("9999".to_string()),
            expiry_month: None,
            expiry_year: None,
            brand: None,
        };

        let cloned = original.clone();
        assert_eq!(original.last_four, cloned.last_four);
    }

    // ==================== BillingAddress Tests ====================

    #[test]
    fn test_billing_address_full() {
        let addr = BillingAddress {
            company: Some("Acme Corp".to_string()),
            line1: "123 Main St".to_string(),
            line2: Some("Suite 100".to_string()),
            city: "San Francisco".to_string(),
            state: Some("CA".to_string()),
            postal_code: "94102".to_string(),
            country: "US".to_string(),
            tax_id: Some("US123456789".to_string()),
        };

        assert_eq!(addr.company, Some("Acme Corp".to_string()));
        assert_eq!(addr.city, "San Francisco");
        assert_eq!(addr.country, "US");
    }

    #[test]
    fn test_billing_address_minimal() {
        let addr = BillingAddress {
            company: None,
            line1: "456 Oak Ave".to_string(),
            line2: None,
            city: "New York".to_string(),
            state: None,
            postal_code: "10001".to_string(),
            country: "US".to_string(),
            tax_id: None,
        };

        assert!(addr.company.is_none());
        assert!(addr.line2.is_none());
    }

    #[test]
    fn test_billing_address_serialize() {
        let addr = BillingAddress {
            company: Some("Test Inc".to_string()),
            line1: "789 Elm St".to_string(),
            line2: None,
            city: "Chicago".to_string(),
            state: Some("IL".to_string()),
            postal_code: "60601".to_string(),
            country: "US".to_string(),
            tax_id: None,
        };

        let json = serde_json::to_string(&addr).unwrap();
        assert!(json.contains("Test Inc"));
        assert!(json.contains("Chicago"));
    }

    #[test]
    fn test_billing_address_clone() {
        let original = BillingAddress {
            company: None,
            line1: "Test St".to_string(),
            line2: None,
            city: "Boston".to_string(),
            state: Some("MA".to_string()),
            postal_code: "02101".to_string(),
            country: "US".to_string(),
            tax_id: None,
        };

        let cloned = original.clone();
        assert_eq!(original.city, cloned.city);
    }

    // ==================== TeamBilling Tests ====================

    #[test]
    fn test_team_billing_creation() {
        let now = Utc::now();
        let billing = TeamBilling {
            plan: BillingPlan::Professional,
            status: BillingStatus::Active,
            monthly_budget: Some(1000.0),
            current_usage: 250.0,
            cycle_start: now,
            cycle_end: now + Duration::days(30),
            payment_method: None,
            billing_address: None,
        };

        assert!(matches!(billing.plan, BillingPlan::Professional));
        assert!(matches!(billing.status, BillingStatus::Active));
        assert_eq!(billing.monthly_budget, Some(1000.0));
        assert_eq!(billing.current_usage, 250.0);
    }

    #[test]
    fn test_team_billing_with_payment_method() {
        let billing = TeamBilling {
            plan: BillingPlan::Enterprise,
            status: BillingStatus::Active,
            monthly_budget: Some(10000.0),
            current_usage: 0.0,
            cycle_start: Utc::now(),
            cycle_end: Utc::now() + Duration::days(30),
            payment_method: Some(PaymentMethod {
                method_type: PaymentMethodType::CreditCard,
                last_four: Some("4242".to_string()),
                expiry_month: Some(12),
                expiry_year: Some(2025),
                brand: Some("Visa".to_string()),
            }),
            billing_address: None,
        };

        assert!(billing.payment_method.is_some());
    }

    #[test]
    fn test_team_billing_with_address() {
        let billing = TeamBilling {
            plan: BillingPlan::Starter,
            status: BillingStatus::Trial,
            monthly_budget: None,
            current_usage: 0.0,
            cycle_start: Utc::now(),
            cycle_end: Utc::now() + Duration::days(14),
            payment_method: None,
            billing_address: Some(BillingAddress {
                company: Some("Startup Inc".to_string()),
                line1: "100 Tech St".to_string(),
                line2: None,
                city: "Austin".to_string(),
                state: Some("TX".to_string()),
                postal_code: "78701".to_string(),
                country: "US".to_string(),
                tax_id: None,
            }),
        };

        assert!(billing.billing_address.is_some());
    }

    #[test]
    fn test_team_billing_serialize() {
        let billing = TeamBilling {
            plan: BillingPlan::Free,
            status: BillingStatus::Active,
            monthly_budget: None,
            current_usage: 0.0,
            cycle_start: Utc::now(),
            cycle_end: Utc::now() + Duration::days(30),
            payment_method: None,
            billing_address: None,
        };

        let json = serde_json::to_string(&billing).unwrap();
        assert!(json.contains("\"plan\":\"free\""));
        assert!(json.contains("\"status\":\"active\""));
    }

    #[test]
    fn test_team_billing_clone() {
        let billing = TeamBilling {
            plan: BillingPlan::Professional,
            status: BillingStatus::Active,
            monthly_budget: Some(500.0),
            current_usage: 100.0,
            cycle_start: Utc::now(),
            cycle_end: Utc::now() + Duration::days(30),
            payment_method: None,
            billing_address: None,
        };

        let cloned = billing.clone();
        assert_eq!(billing.monthly_budget, cloned.monthly_budget);
        assert_eq!(billing.current_usage, cloned.current_usage);
    }

    #[test]
    fn test_team_billing_debug() {
        let billing = TeamBilling {
            plan: BillingPlan::Custom,
            status: BillingStatus::Suspended,
            monthly_budget: None,
            current_usage: 0.0,
            cycle_start: Utc::now(),
            cycle_end: Utc::now() + Duration::days(30),
            payment_method: None,
            billing_address: None,
        };

        let debug_str = format!("{:?}", billing);
        assert!(debug_str.contains("TeamBilling"));
        assert!(debug_str.contains("Custom"));
    }
}
