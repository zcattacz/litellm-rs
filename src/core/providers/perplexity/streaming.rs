//! Perplexity Streaming Support
//!
//! Uses the unified SSE parser for consistent streaming across providers.

use bytes::Bytes;
use futures::Stream;
use std::pin::Pin;

use crate::core::providers::base::sse::{OpenAICompatibleTransformer, UnifiedSSEStream};

/// Perplexity uses OpenAI-compatible SSE format
pub type PerplexityStream = UnifiedSSEStream<
    Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
    OpenAICompatibleTransformer,
>;

/// Helper function to create Perplexity stream
pub fn create_perplexity_stream(
    stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
) -> PerplexityStream {
    let transformer = OpenAICompatibleTransformer::new("perplexity");
    UnifiedSSEStream::new(Box::pin(stream), transformer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::providers::base::sse::UnifiedSSEParser;
    use futures::StreamExt;

    #[test]
    fn test_sse_parsing() {
        let transformer = OpenAICompatibleTransformer::new("perplexity");
        let mut parser = UnifiedSSEParser::new(transformer);

        let test_data = b"data: {\"id\":\"chatcmpl-test\",\"object\":\"chat.completion.chunk\",\"created\":1640995200,\"model\":\"sonar\",\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":\"Hello\"},\"finish_reason\":null}]}\n\n";

        let result = parser.process_bytes(test_data).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].choices[0].delta.content,
            Some("Hello".to_string())
        );
    }

    #[test]
    fn test_done_message() {
        let transformer = OpenAICompatibleTransformer::new("perplexity");
        let mut parser = UnifiedSSEParser::new(transformer);

        let test_data = b"data: [DONE]\n\n";
        let result = parser.process_bytes(test_data).unwrap();
        // [DONE] should not produce any chunks
        assert!(result.is_empty());
    }

    #[test]
    fn test_sse_parsing_with_citations() {
        let transformer = OpenAICompatibleTransformer::new("perplexity");
        let mut parser = UnifiedSSEParser::new(transformer);

        // Perplexity may include citations in response - the SSE format is still OpenAI-compatible
        let test_data = b"data: {\"id\":\"chatcmpl-123\",\"object\":\"chat.completion.chunk\",\"created\":1640995200,\"model\":\"sonar\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"According to [1]\"},\"finish_reason\":null}]}\n\n";

        let result = parser.process_bytes(test_data).unwrap();
        assert_eq!(result.len(), 1);
        assert!(
            result[0].choices[0]
                .delta
                .content
                .as_ref()
                .unwrap()
                .contains("[1]")
        );
    }

    #[tokio::test]
    async fn test_perplexity_stream() {
        use futures::stream;

        let test_data = vec![
            Ok(Bytes::from(
                "data: {\"id\":\"test-1\",\"object\":\"chat.completion.chunk\",\"created\":1234567890,\"model\":\"sonar\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hello\"},\"finish_reason\":null}]}\n\n",
            )),
            Ok(Bytes::from("data: [DONE]\n\n")),
        ];

        let mock_stream = stream::iter(test_data);
        let mut perplexity_stream = create_perplexity_stream(mock_stream);

        // First chunk
        let chunk1 = perplexity_stream.next().await;
        assert!(chunk1.is_some());
        let chunk1 = chunk1.unwrap().unwrap();
        assert_eq!(chunk1.choices[0].delta.content.as_ref().unwrap(), "Hello");

        // Stream should end after [DONE]
        let end = perplexity_stream.next().await;
        assert!(end.is_none());
    }

    #[tokio::test]
    async fn test_perplexity_stream_multiple_chunks() {
        use futures::stream;

        let test_data = vec![
            Ok(Bytes::from(
                "data: {\"id\":\"test-1\",\"object\":\"chat.completion.chunk\",\"created\":1234567890,\"model\":\"sonar\",\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\"},\"finish_reason\":null}]}\n\n",
            )),
            Ok(Bytes::from(
                "data: {\"id\":\"test-1\",\"object\":\"chat.completion.chunk\",\"created\":1234567890,\"model\":\"sonar\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"The answer is\"},\"finish_reason\":null}]}\n\n",
            )),
            Ok(Bytes::from(
                "data: {\"id\":\"test-1\",\"object\":\"chat.completion.chunk\",\"created\":1234567890,\"model\":\"sonar\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\" 42.\"},\"finish_reason\":\"stop\"}]}\n\n",
            )),
            Ok(Bytes::from("data: [DONE]\n\n")),
        ];

        let mock_stream = stream::iter(test_data);
        let mut perplexity_stream = create_perplexity_stream(mock_stream);

        // Collect all chunks
        let mut contents = Vec::new();
        while let Some(result) = perplexity_stream.next().await {
            if let Ok(chunk) = result {
                if let Some(content) = &chunk.choices[0].delta.content {
                    contents.push(content.clone());
                }
            }
        }

        assert_eq!(contents.len(), 2);
        assert_eq!(contents[0], "The answer is");
        assert_eq!(contents[1], " 42.");
    }
}
