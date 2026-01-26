//! Weights & Biases (W&B) Provider
//!
//! This module provides integration with Weights & Biases for LLM call
//! logging, experiment tracking, and observability.
//!
//! # Overview
//!
//! Unlike traditional LLM providers (OpenAI, Anthropic, etc.), W&B is primarily
//! an observability and experiment tracking integration. It doesn't provide
//! LLM capabilities directly but instead:
//!
//! - Logs LLM calls (prompts, responses, token usage, costs, latency)
//! - Tracks experiments and metrics over time
//! - Enables prompt monitoring and analysis
//! - Provides cost tracking and optimization insights
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use litellm_rs::core::providers::wandb::{WandbProvider, WandbConfig, WandbLogger};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a W&B logger
//! let config = WandbConfig::new("your-wandb-api-key")
//!     .with_project("my-llm-project")
//!     .with_entity("my-team");
//!
//! let logger = WandbLogger::new(config)?;
//!
//! // Initialize a run
//! logger.init_run().await?;
//!
//! // Log LLM calls
//! logger.log_success(
//!     "openai",
//!     "gpt-4",
//!     None,  // input (optional)
//!     None,  // output (optional)
//!     100,   // input tokens
//!     50,    // output tokens
//!     Some(0.01),  // cost
//!     200,   // latency ms
//! ).await?;
//!
//! // Get run summary
//! let summary = logger.get_summary().await;
//! println!("Total calls: {}", summary.total_calls);
//! println!("Total cost: ${:.4}", summary.total_cost_usd);
//!
//! // Finish the run
//! logger.finish().await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Configuration
//!
//! Configuration can be provided via:
//! - Direct configuration: `WandbConfig::new("api-key")`
//! - Environment variables: `WANDB_API_KEY`, `WANDB_PROJECT`, `WANDB_ENTITY`
//!
//! # Privacy Controls
//!
//! The logger supports privacy controls to avoid logging sensitive data:
//!
//! ```rust
//! use litellm_rs::core::providers::wandb::WandbConfig;
//!
//! let config = WandbConfig::new("api-key")
//!     .without_prompt_logging()    // Don't log prompts
//!     .without_response_logging(); // Don't log responses
//! ```
//!
//! # Batched Logging
//!
//! For performance optimization, logs are batched before sending to W&B:
//!
//! ```rust
//! use litellm_rs::core::providers::wandb::WandbConfig;
//!
//! let config = WandbConfig::new("api-key")
//!     .with_batch_settings(
//!         20,   // Batch size (send when 20 logs accumulated)
//!         60,   // Flush interval in seconds
//!     );
//! ```

mod config;
mod logger;
mod provider;

// Re-export main types
pub use config::{PROVIDER_NAME, WANDB_API_BASE, WANDB_API_KEY_ENV, WandbConfig};
pub use logger::{LLMCallLog, RunState, RunSummary, WandbLogger, WandbRun};
pub use provider::{WandbError, WandbErrorMapper, WandbProvider};

/// Create a chat log helper function (re-exported for convenience)
pub use logger::create_chat_log;
