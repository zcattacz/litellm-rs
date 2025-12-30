//! Type definitions for function calling
//!
//! This module contains all the core types used in function calling.

use serde_json::Value;

/// Function definition for AI models
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionDefinition {
    /// Function name
    pub name: String,
    /// Function description
    pub description: Option<String>,
    /// Function parameters schema (JSON Schema)
    pub parameters: Value,
    /// Whether the function is strict (OpenAI specific)
    pub strict: Option<bool>,
}

/// Tool definition for AI models
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolDefinition {
    /// Tool type (currently only "function")
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Function definition
    pub function: FunctionDefinition,
}

/// Tool choice options
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// No tools should be called
    None,
    /// Let the model decide
    Auto,
    /// Force a specific tool
    Required,
    /// Specific tool to use
    Specific {
        /// Tool type identifier
        #[serde(rename = "type")]
        tool_type: String,
        /// Function choice details
        function: FunctionChoice,
    },
}

/// Specific function choice
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionChoice {
    /// Function name to call
    pub name: String,
}

/// Function call in a message
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionCall {
    /// Function name
    pub name: String,
    /// Function arguments (JSON string)
    pub arguments: String,
}

/// Tool call in a message
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolCall {
    /// Tool call ID
    pub id: String,
    /// Tool type
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Function call details
    pub function: FunctionCall,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ==================== FunctionDefinition Tests ====================

    #[test]
    fn test_function_definition_creation() {
        let func = FunctionDefinition {
            name: "get_weather".to_string(),
            description: Some("Get the current weather for a location".to_string()),
            parameters: json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "The city and state"
                    }
                },
                "required": ["location"]
            }),
            strict: Some(true),
        };

        assert_eq!(func.name, "get_weather");
        assert!(func.description.is_some());
        assert!(func.strict.unwrap());
    }

    #[test]
    fn test_function_definition_minimal() {
        let func = FunctionDefinition {
            name: "simple_function".to_string(),
            description: None,
            parameters: json!({}),
            strict: None,
        };

        assert_eq!(func.name, "simple_function");
        assert!(func.description.is_none());
        assert!(func.strict.is_none());
    }

    #[test]
    fn test_function_definition_serialization() {
        let func = FunctionDefinition {
            name: "search".to_string(),
            description: Some("Search for items".to_string()),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string"}
                }
            }),
            strict: None,
        };

        let json = serde_json::to_string(&func).unwrap();
        assert!(json.contains("search"));
        assert!(json.contains("Search for items"));

        let parsed: FunctionDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "search");
    }

    #[test]
    fn test_function_definition_complex_parameters() {
        let func = FunctionDefinition {
            name: "create_order".to_string(),
            description: Some("Create a new order".to_string()),
            parameters: json!({
                "type": "object",
                "properties": {
                    "items": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "product_id": {"type": "string"},
                                "quantity": {"type": "integer"}
                            }
                        }
                    },
                    "customer_id": {"type": "string"},
                    "shipping_address": {
                        "type": "object",
                        "properties": {
                            "street": {"type": "string"},
                            "city": {"type": "string"},
                            "zip": {"type": "string"}
                        }
                    }
                },
                "required": ["items", "customer_id"]
            }),
            strict: Some(true),
        };

        let json = serde_json::to_string(&func).unwrap();
        assert!(json.contains("items"));
        assert!(json.contains("customer_id"));
        assert!(json.contains("shipping_address"));
    }

    #[test]
    fn test_function_definition_clone() {
        let func = FunctionDefinition {
            name: "test".to_string(),
            description: Some("Test function".to_string()),
            parameters: json!({"type": "object"}),
            strict: Some(false),
        };

        let cloned = func.clone();
        assert_eq!(cloned.name, func.name);
        assert_eq!(cloned.description, func.description);
    }

    // ==================== ToolDefinition Tests ====================

    #[test]
    fn test_tool_definition_creation() {
        let tool = ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "get_stock_price".to_string(),
                description: Some("Get current stock price".to_string()),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "symbol": {"type": "string"}
                    }
                }),
                strict: None,
            },
        };

        assert_eq!(tool.tool_type, "function");
        assert_eq!(tool.function.name, "get_stock_price");
    }

    #[test]
    fn test_tool_definition_serialization() {
        let tool = ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "calculator".to_string(),
                description: Some("Perform calculations".to_string()),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "expression": {"type": "string"}
                    }
                }),
                strict: Some(true),
            },
        };

        let json = serde_json::to_string(&tool).unwrap();
        // Should use "type" due to rename
        assert!(json.contains("\"type\":\"function\""));
        assert!(json.contains("calculator"));

        let parsed: ToolDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.tool_type, "function");
    }

    #[test]
    fn test_tool_definition_clone() {
        let tool = ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "test".to_string(),
                description: None,
                parameters: json!({}),
                strict: None,
            },
        };

        let cloned = tool.clone();
        assert_eq!(cloned.tool_type, tool.tool_type);
        assert_eq!(cloned.function.name, tool.function.name);
    }

    // ==================== FunctionChoice Tests ====================

    #[test]
    fn test_function_choice_creation() {
        let choice = FunctionChoice {
            name: "get_weather".to_string(),
        };

        assert_eq!(choice.name, "get_weather");
    }

    #[test]
    fn test_function_choice_serialization() {
        let choice = FunctionChoice {
            name: "search_database".to_string(),
        };

        let json = serde_json::to_string(&choice).unwrap();
        assert!(json.contains("search_database"));

        let parsed: FunctionChoice = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "search_database");
    }

    #[test]
    fn test_function_choice_clone() {
        let choice = FunctionChoice {
            name: "my_function".to_string(),
        };

        let cloned = choice.clone();
        assert_eq!(cloned.name, choice.name);
    }

    // ==================== ToolChoice Tests ====================

    #[test]
    fn test_tool_choice_specific() {
        let choice = ToolChoice::Specific {
            tool_type: "function".to_string(),
            function: FunctionChoice {
                name: "get_weather".to_string(),
            },
        };

        if let ToolChoice::Specific { tool_type, function } = &choice {
            assert_eq!(tool_type, "function");
            assert_eq!(function.name, "get_weather");
        } else {
            panic!("Expected Specific variant");
        }
    }

    #[test]
    fn test_tool_choice_specific_serialization() {
        let choice = ToolChoice::Specific {
            tool_type: "function".to_string(),
            function: FunctionChoice {
                name: "my_tool".to_string(),
            },
        };

        let json = serde_json::to_string(&choice).unwrap();
        assert!(json.contains("function"));
        assert!(json.contains("my_tool"));
    }

    #[test]
    fn test_tool_choice_clone() {
        let choice = ToolChoice::Specific {
            tool_type: "function".to_string(),
            function: FunctionChoice {
                name: "test".to_string(),
            },
        };

        let cloned = choice.clone();
        if let (
            ToolChoice::Specific { tool_type: t1, function: f1 },
            ToolChoice::Specific { tool_type: t2, function: f2 },
        ) = (&choice, &cloned)
        {
            assert_eq!(t1, t2);
            assert_eq!(f1.name, f2.name);
        }
    }

    // ==================== FunctionCall Tests ====================

    #[test]
    fn test_function_call_creation() {
        let call = FunctionCall {
            name: "get_weather".to_string(),
            arguments: r#"{"location": "San Francisco, CA"}"#.to_string(),
        };

        assert_eq!(call.name, "get_weather");
        assert!(call.arguments.contains("San Francisco"));
    }

    #[test]
    fn test_function_call_serialization() {
        let call = FunctionCall {
            name: "search".to_string(),
            arguments: r#"{"query": "rust programming"}"#.to_string(),
        };

        let json = serde_json::to_string(&call).unwrap();
        assert!(json.contains("search"));
        assert!(json.contains("arguments"));

        let parsed: FunctionCall = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "search");
    }

    #[test]
    fn test_function_call_parse_arguments() {
        let call = FunctionCall {
            name: "calculate".to_string(),
            arguments: r#"{"x": 10, "y": 20, "operation": "add"}"#.to_string(),
        };

        // Parse arguments as JSON
        let args: Value = serde_json::from_str(&call.arguments).unwrap();
        assert_eq!(args["x"], 10);
        assert_eq!(args["y"], 20);
        assert_eq!(args["operation"], "add");
    }

    #[test]
    fn test_function_call_empty_arguments() {
        let call = FunctionCall {
            name: "no_args_function".to_string(),
            arguments: "{}".to_string(),
        };

        assert_eq!(call.arguments, "{}");
        let args: Value = serde_json::from_str(&call.arguments).unwrap();
        assert!(args.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_function_call_clone() {
        let call = FunctionCall {
            name: "test".to_string(),
            arguments: r#"{"key": "value"}"#.to_string(),
        };

        let cloned = call.clone();
        assert_eq!(cloned.name, call.name);
        assert_eq!(cloned.arguments, call.arguments);
    }

    // ==================== ToolCall Tests ====================

    #[test]
    fn test_tool_call_creation() {
        let call = ToolCall {
            id: "call_abc123".to_string(),
            tool_type: "function".to_string(),
            function: FunctionCall {
                name: "get_weather".to_string(),
                arguments: r#"{"city": "NYC"}"#.to_string(),
            },
        };

        assert_eq!(call.id, "call_abc123");
        assert_eq!(call.tool_type, "function");
        assert_eq!(call.function.name, "get_weather");
    }

    #[test]
    fn test_tool_call_serialization() {
        let call = ToolCall {
            id: "tool_call_1".to_string(),
            tool_type: "function".to_string(),
            function: FunctionCall {
                name: "send_email".to_string(),
                arguments: r#"{"to": "user@example.com", "subject": "Hello"}"#.to_string(),
            },
        };

        let json = serde_json::to_string(&call).unwrap();
        // Should use "type" due to rename
        assert!(json.contains("\"type\":\"function\""));
        assert!(json.contains("tool_call_1"));
        assert!(json.contains("send_email"));

        let parsed: ToolCall = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "tool_call_1");
        assert_eq!(parsed.tool_type, "function");
    }

    #[test]
    fn test_tool_call_clone() {
        let call = ToolCall {
            id: "id_123".to_string(),
            tool_type: "function".to_string(),
            function: FunctionCall {
                name: "test".to_string(),
                arguments: "{}".to_string(),
            },
        };

        let cloned = call.clone();
        assert_eq!(cloned.id, call.id);
        assert_eq!(cloned.tool_type, call.tool_type);
        assert_eq!(cloned.function.name, call.function.name);
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_complete_tool_workflow() {
        // Define a tool
        let tool = ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "get_current_time".to_string(),
                description: Some("Get the current time in a timezone".to_string()),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "timezone": {
                            "type": "string",
                            "description": "The timezone (e.g., 'America/New_York')"
                        }
                    },
                    "required": ["timezone"]
                }),
                strict: Some(true),
            },
        };

        // Simulate model making a tool call
        let tool_call = ToolCall {
            id: "call_xyz".to_string(),
            tool_type: "function".to_string(),
            function: FunctionCall {
                name: "get_current_time".to_string(),
                arguments: r#"{"timezone": "America/New_York"}"#.to_string(),
            },
        };

        // Verify the tool call matches the definition
        assert_eq!(tool_call.function.name, tool.function.name);
        assert_eq!(tool_call.tool_type, tool.tool_type);
    }

    #[test]
    fn test_multiple_tools() {
        let tools = vec![
            ToolDefinition {
                tool_type: "function".to_string(),
                function: FunctionDefinition {
                    name: "get_weather".to_string(),
                    description: Some("Get weather".to_string()),
                    parameters: json!({"type": "object"}),
                    strict: None,
                },
            },
            ToolDefinition {
                tool_type: "function".to_string(),
                function: FunctionDefinition {
                    name: "search_web".to_string(),
                    description: Some("Search the web".to_string()),
                    parameters: json!({"type": "object"}),
                    strict: None,
                },
            },
            ToolDefinition {
                tool_type: "function".to_string(),
                function: FunctionDefinition {
                    name: "calculate".to_string(),
                    description: Some("Perform math".to_string()),
                    parameters: json!({"type": "object"}),
                    strict: None,
                },
            },
        ];

        assert_eq!(tools.len(), 3);

        let json = serde_json::to_string(&tools).unwrap();
        let parsed: Vec<ToolDefinition> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 3);
    }

    #[test]
    fn test_parallel_tool_calls() {
        // Simulate multiple tool calls in one response
        let tool_calls = vec![
            ToolCall {
                id: "call_1".to_string(),
                tool_type: "function".to_string(),
                function: FunctionCall {
                    name: "get_weather".to_string(),
                    arguments: r#"{"city": "NYC"}"#.to_string(),
                },
            },
            ToolCall {
                id: "call_2".to_string(),
                tool_type: "function".to_string(),
                function: FunctionCall {
                    name: "get_weather".to_string(),
                    arguments: r#"{"city": "LA"}"#.to_string(),
                },
            },
        ];

        assert_eq!(tool_calls.len(), 2);
        assert_ne!(tool_calls[0].id, tool_calls[1].id);

        // Serialize and deserialize
        let json = serde_json::to_string(&tool_calls).unwrap();
        let parsed: Vec<ToolCall> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn test_tool_choice_with_definition() {
        let tool = ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "required_tool".to_string(),
                description: None,
                parameters: json!({}),
                strict: None,
            },
        };

        // Force the model to use a specific tool
        let choice = ToolChoice::Specific {
            tool_type: "function".to_string(),
            function: FunctionChoice {
                name: tool.function.name.clone(),
            },
        };

        if let ToolChoice::Specific { function, .. } = choice {
            assert_eq!(function.name, "required_tool");
        }
    }

    #[test]
    fn test_debug_formatting() {
        let func = FunctionDefinition {
            name: "debug_test".to_string(),
            description: Some("Test".to_string()),
            parameters: json!({}),
            strict: None,
        };

        let debug_str = format!("{:?}", func);
        assert!(debug_str.contains("FunctionDefinition"));
        assert!(debug_str.contains("debug_test"));
    }
}
