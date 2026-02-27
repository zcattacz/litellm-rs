//! Streaming Module for Together AI
//!
//! Uses the unified SSE parser for consistent streaming across providers.
//! Together AI uses OpenAI-compatible SSE format.

use crate::core::providers::base::sse::{OpenAICompatibleTransformer, UnifiedSSEStream};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::responses::{ChatChunk, ChatDelta, ChatResponse, ChatStreamChoice};
use crate::core::types::{message::MessageContent, message::MessageRole};
use bytes::Bytes;
use futures::Stream;
use std::pin::Pin;

/// Together AI uses OpenAI-compatible SSE format
pub type TogetherStreamInner = UnifiedSSEStream<
    Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
    OpenAICompatibleTransformer,
>;

/// Helper function to create Together AI stream
pub fn create_together_stream(
    stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
) -> TogetherStreamInner {
    let transformer = OpenAICompatibleTransformer::new("together");
    UnifiedSSEStream::new(Box::pin(stream), transformer)
}

/// Create a fake stream from a complete response
/// Used when the API doesn't support streaming for certain features
pub async fn create_fake_stream(
    response: ChatResponse,
) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, ProviderError>> + Send>>, ProviderError> {
    // Convert response to chunks
    let chunks = response_to_chunks(response);
    let stream = futures::stream::iter(chunks.into_iter().map(Ok));
    Ok(Box::pin(stream))
}

/// Convert a complete ChatResponse to stream chunks
fn response_to_chunks(response: ChatResponse) -> Vec<ChatChunk> {
    let mut chunks = Vec::new();

    // Create initial chunk with role
    chunks.push(ChatChunk {
        id: response.id.clone(),
        object: "chat.completion.chunk".to_string(),
        created: response.created,
        model: response.model.clone(),
        system_fingerprint: response.system_fingerprint.clone(),
        choices: vec![ChatStreamChoice {
            index: 0,
            delta: ChatDelta {
                role: Some(MessageRole::Assistant),
                content: None,
                thinking: None,
                tool_calls: None,
                function_call: None,
            },
            finish_reason: None,
            logprobs: None,
        }],
        usage: None,
    });

    // Create content chunks
    if let Some(choice) = response.choices.first() {
        if let Some(content) = &choice.message.content {
            let text = match content {
                MessageContent::Text(text) => text.clone(),
                MessageContent::Parts(_) => content.to_string(), // Use Display impl
            };

            // Split content into smaller chunks for more natural streaming
            let words: Vec<&str> = text.split_whitespace().collect();
            let chunk_size = 5; // Words per chunk

            for word_chunk in words.chunks(chunk_size) {
                let chunk_text = word_chunk.join(" ") + " ";
                chunks.push(ChatChunk {
                    id: response.id.clone(),
                    object: "chat.completion.chunk".to_string(),
                    created: response.created,
                    model: response.model.clone(),
                    system_fingerprint: response.system_fingerprint.clone(),
                    choices: vec![ChatStreamChoice {
                        index: 0,
                        delta: ChatDelta {
                            role: None,
                            content: Some(chunk_text),
                            thinking: None,
                            tool_calls: None,
                            function_call: None,
                        },
                        finish_reason: None,
                        logprobs: None,
                    }],
                    usage: None,
                });
            }
        }

        // Add final chunk with finish_reason
        chunks.push(ChatChunk {
            id: response.id.clone(),
            object: "chat.completion.chunk".to_string(),
            created: response.created,
            model: response.model.clone(),
            system_fingerprint: response.system_fingerprint.clone(),
            choices: vec![ChatStreamChoice {
                index: 0,
                delta: ChatDelta {
                    role: None,
                    content: None,
                    thinking: None,
                    tool_calls: None,
                    function_call: None,
                },
                finish_reason: choice.finish_reason.clone(),
                logprobs: None,
            }],
            usage: response.usage.clone(),
        });
    }

    chunks
}
