//! Cost information models

use serde::{Deserialize, Serialize};

/// Cost information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CostInfo {
    /// Input cost
    pub input_cost: f64,
    /// Output cost
    pub output_cost: f64,
    /// Total cost
    pub total_cost: f64,
    /// Currency
    pub currency: String,
    /// Cost per token rates
    pub rates: CostRates,
}

/// Cost rates
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CostRates {
    /// Input cost per token
    pub input_cost_per_token: f64,
    /// Output cost per token
    pub output_cost_per_token: f64,
    /// Cost per request
    pub cost_per_request: Option<f64>,
}

impl CostInfo {
    /// Create new cost info
    pub fn new(input_cost: f64, output_cost: f64, currency: String) -> Self {
        Self {
            input_cost,
            output_cost,
            total_cost: input_cost + output_cost,
            currency,
            rates: CostRates::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== CostInfo Tests ====================

    #[test]
    fn test_cost_calculation() {
        let cost = CostInfo::new(0.01, 0.02, "USD".to_string());
        assert_eq!(cost.input_cost, 0.01);
        assert_eq!(cost.output_cost, 0.02);
        assert_eq!(cost.total_cost, 0.03);
        assert_eq!(cost.currency, "USD");
    }

    #[test]
    fn test_cost_info_default() {
        let cost = CostInfo::default();
        assert_eq!(cost.input_cost, 0.0);
        assert_eq!(cost.output_cost, 0.0);
        assert_eq!(cost.total_cost, 0.0);
        assert!(cost.currency.is_empty());
    }

    #[test]
    fn test_cost_info_zero_costs() {
        let cost = CostInfo::new(0.0, 0.0, "EUR".to_string());
        assert_eq!(cost.total_cost, 0.0);
        assert_eq!(cost.currency, "EUR");
    }

    #[test]
    fn test_cost_info_large_values() {
        let cost = CostInfo::new(1000.50, 2000.75, "USD".to_string());
        assert_eq!(cost.total_cost, 3001.25);
    }

    #[test]
    fn test_cost_info_small_values() {
        let cost = CostInfo::new(0.000001, 0.000002, "USD".to_string());
        assert!((cost.total_cost - 0.000003).abs() < 1e-10);
    }

    #[test]
    fn test_cost_info_with_rates() {
        let mut cost = CostInfo::new(0.01, 0.02, "USD".to_string());
        cost.rates = CostRates {
            input_cost_per_token: 0.00001,
            output_cost_per_token: 0.00002,
            cost_per_request: Some(0.001),
        };
        assert_eq!(cost.rates.input_cost_per_token, 0.00001);
        assert_eq!(cost.rates.output_cost_per_token, 0.00002);
        assert_eq!(cost.rates.cost_per_request, Some(0.001));
    }

    // ==================== CostRates Tests ====================

    #[test]
    fn test_cost_rates_default() {
        let rates = CostRates::default();
        assert_eq!(rates.input_cost_per_token, 0.0);
        assert_eq!(rates.output_cost_per_token, 0.0);
        assert!(rates.cost_per_request.is_none());
    }

    #[test]
    fn test_cost_rates_with_values() {
        let rates = CostRates {
            input_cost_per_token: 0.00003,
            output_cost_per_token: 0.00006,
            cost_per_request: Some(0.0),
        };
        assert_eq!(rates.input_cost_per_token, 0.00003);
        assert_eq!(rates.output_cost_per_token, 0.00006);
        assert_eq!(rates.cost_per_request, Some(0.0));
    }

    // ==================== Serialization Tests ====================

    #[test]
    fn test_cost_info_serialization() {
        let cost = CostInfo::new(0.01, 0.02, "USD".to_string());
        let json = serde_json::to_value(&cost).unwrap();
        assert_eq!(json["input_cost"], 0.01);
        assert_eq!(json["output_cost"], 0.02);
        assert_eq!(json["total_cost"], 0.03);
        assert_eq!(json["currency"], "USD");
    }

    #[test]
    fn test_cost_info_deserialization() {
        let json = r#"{
            "input_cost": 0.05,
            "output_cost": 0.10,
            "total_cost": 0.15,
            "currency": "GBP",
            "rates": {
                "input_cost_per_token": 0.0001,
                "output_cost_per_token": 0.0002,
                "cost_per_request": null
            }
        }"#;
        let cost: CostInfo = serde_json::from_str(json).unwrap();
        assert_eq!(cost.input_cost, 0.05);
        assert_eq!(cost.output_cost, 0.10);
        assert_eq!(cost.currency, "GBP");
    }

    #[test]
    fn test_cost_rates_serialization() {
        let rates = CostRates {
            input_cost_per_token: 0.00001,
            output_cost_per_token: 0.00002,
            cost_per_request: Some(0.005),
        };
        let json = serde_json::to_value(&rates).unwrap();
        assert_eq!(json["input_cost_per_token"], 0.00001);
        assert_eq!(json["cost_per_request"], 0.005);
    }

    #[test]
    fn test_cost_info_roundtrip() {
        let original = CostInfo::new(1.23, 4.56, "JPY".to_string());
        let json = serde_json::to_string(&original).unwrap();
        let restored: CostInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(original.input_cost, restored.input_cost);
        assert_eq!(original.output_cost, restored.output_cost);
        assert_eq!(original.currency, restored.currency);
    }

    // ==================== Clone Tests ====================

    #[test]
    fn test_cost_info_clone() {
        let original = CostInfo::new(0.01, 0.02, "USD".to_string());
        let cloned = original.clone();
        assert_eq!(original.input_cost, cloned.input_cost);
        assert_eq!(original.output_cost, cloned.output_cost);
        assert_eq!(original.total_cost, cloned.total_cost);
    }

    #[test]
    fn test_cost_rates_clone() {
        let original = CostRates {
            input_cost_per_token: 0.001,
            output_cost_per_token: 0.002,
            cost_per_request: Some(0.01),
        };
        let cloned = original.clone();
        assert_eq!(original.input_cost_per_token, cloned.input_cost_per_token);
        assert_eq!(original.cost_per_request, cloned.cost_per_request);
    }
}
