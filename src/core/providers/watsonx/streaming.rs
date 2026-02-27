//! Streaming Module for Watsonx
//!
//! Uses the unified SSE parser for consistent streaming across providers.
//! Also provides fake streaming support when needed.

use crate::core::providers::base::sse::{OpenAICompatibleTransformer, UnifiedSSEStream};
use bytes::Bytes;
use futures::Stream;
use std::pin::Pin;

/// Watsonx uses OpenAI-compatible SSE format
pub type WatsonxStreamInner = UnifiedSSEStream<
    Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
    OpenAICompatibleTransformer,
>;

/// Helper function to create Watsonx stream
pub fn create_watsonx_stream(
    stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
) -> WatsonxStreamInner {
    let transformer = OpenAICompatibleTransformer::new("watsonx");
    UnifiedSSEStream::new(Box::pin(stream), transformer)
}
