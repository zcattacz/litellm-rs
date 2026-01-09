//! AWS Sagemaker Provider
//!
//! AWS Sagemaker provides managed infrastructure for deploying machine learning models.
//! This implementation supports Sagemaker endpoints with HuggingFace TGI format
//! and AWS SigV4 authentication.
//!
//! Reference: https://docs.aws.amazon.com/sagemaker/latest/dg/realtime-endpoints.html

// Core modules
mod config;
mod error;
mod provider;
mod sigv4;

// Tests
#[cfg(test)]
mod tests;

// Re-export main types for external use
pub use config::SagemakerConfig;
pub use error::{SagemakerError, SagemakerErrorMapper};
pub use provider::SagemakerProvider;
pub use sigv4::SagemakerSigV4Signer;
