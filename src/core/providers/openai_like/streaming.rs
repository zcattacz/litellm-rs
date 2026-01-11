//! OpenAI-Like Streaming Response Handler
//!
//! Uses the unified SSE parser for consistent streaming.

use bytes::Bytes;
use futures::Stream;
use std::pin::Pin;

use super::error::OpenAILikeError;
use crate::core::providers::base::sse::{OpenAICompatibleTransformer, UnifiedSSEStream};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::responses::ChatChunk;

/// OpenAI-like uses OpenAI-compatible SSE format
pub type OpenAILikeStream = UnifiedSSEStream<
    Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
    OpenAICompatibleTransformer,
>;

/// Helper function to create OpenAI-like stream
pub fn create_openai_like_stream(
    stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
) -> OpenAILikeStream {
    let transformer = OpenAICompatibleTransformer::new("openai_like");
    UnifiedSSEStream::new(Box::pin(stream), transformer)
}

/// Wrapper stream that converts ProviderError to OpenAILikeError for backward compatibility
pub struct OpenAILikeStreamCompat {
    inner: OpenAILikeStream,
}

impl OpenAILikeStreamCompat {
    pub fn new(stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static) -> Self {
        Self {
            inner: create_openai_like_stream(stream),
        }
    }
}

impl Stream for OpenAILikeStreamCompat {
    type Item = Result<ChatChunk, OpenAILikeError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use std::pin::Pin;
        use std::task::Poll;

        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => Poll::Ready(Some(Ok(chunk))),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(provider_error_to_openai_like(e)))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Convert ProviderError to OpenAILikeError
/// Since OpenAILikeError is now an alias for ProviderError, this is essentially a passthrough
fn provider_error_to_openai_like(e: ProviderError) -> OpenAILikeError {
    // Since OpenAILikeError is a type alias for ProviderError, we can just return it directly
    // But we update the provider field to ensure consistency
    match e {
        ProviderError::ResponseParsing { message, .. } => {
            OpenAILikeError::response_parsing("openai_like", message)
        }
        ProviderError::Network { message, .. } => OpenAILikeError::other("openai_like", message),
        ProviderError::Streaming { message, .. } => {
            OpenAILikeError::streaming_error("openai_like", "chat", None, None, message)
        }
        _ => OpenAILikeError::other("openai_like", e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::providers::base::sse::UnifiedSSEParser;
    use futures::StreamExt;

    #[test]
    fn test_sse_parsing() {
        let transformer = OpenAICompatibleTransformer::new("openai_like");
        let mut parser = UnifiedSSEParser::new(transformer);

        let test_data = b"data: {\"id\":\"chatcmpl-123\",\"object\":\"chat.completion.chunk\",\"created\":1234567890,\"model\":\"custom-model\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hello\"},\"finish_reason\":null}]}\n\n";

        let result = parser.process_bytes(test_data);
        assert!(result.is_ok());

        let chunks = result.unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].id, "chatcmpl-123");
        assert_eq!(chunks[0].model, "custom-model");
    }

    #[test]
    fn test_done_message() {
        let transformer = OpenAICompatibleTransformer::new("openai_like");
        let mut parser = UnifiedSSEParser::new(transformer);

        let done_data = b"data: [DONE]\n\n";
        let result = parser.process_bytes(done_data);

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_stream_wrapper() {
        use futures::stream;

        let data = vec![
            Ok(Bytes::from(
                "data: {\"id\":\"test\",\"object\":\"chat.completion.chunk\",\"created\":123,\"model\":\"local-model\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hi\"},\"finish_reason\":null}]}\n\n",
            )),
            Ok(Bytes::from("data: [DONE]\n\n")),
        ];

        let mock_stream = stream::iter(data);
        let mut openai_like_stream = create_openai_like_stream(mock_stream);

        let first_chunk = openai_like_stream.next().await;
        assert!(first_chunk.is_some());

        if let Some(Ok(chunk)) = first_chunk {
            assert_eq!(chunk.id, "test");
            assert_eq!(chunk.choices[0].delta.content, Some("Hi".to_string()));
        }

        let second_chunk = openai_like_stream.next().await;
        assert!(second_chunk.is_none());
    }

    #[test]
    fn test_provider_error_conversion() {
        let provider_err = ProviderError::ResponseParsing {
            provider: "test",
            message: "Invalid JSON".to_string(),
        };
        let openai_like_err = provider_error_to_openai_like(provider_err);

        assert!(matches!(
            openai_like_err,
            OpenAILikeError::ResponseParsing { .. }
        ));
    }
}
