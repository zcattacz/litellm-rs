//! AI21 Streaming Support
//!
//! Uses the unified SSE parser for consistent streaming across providers.
//! AI21 uses OpenAI-compatible SSE format.

use bytes::Bytes;
use futures::Stream;
use std::pin::Pin;

use crate::core::providers::base::sse::{OpenAICompatibleTransformer, UnifiedSSEStream};

/// AI21 uses OpenAI-compatible SSE format
pub type AI21Stream = UnifiedSSEStream<
    Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
    OpenAICompatibleTransformer,
>;

/// Helper function to create AI21 stream
pub fn create_ai21_stream(
    stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
) -> AI21Stream {
    let transformer = OpenAICompatibleTransformer::new("ai21");
    UnifiedSSEStream::new(Box::pin(stream), transformer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::providers::base::sse::UnifiedSSEParser;
    use futures::StreamExt;

    #[test]
    fn test_sse_parsing() {
        let transformer = OpenAICompatibleTransformer::new("ai21");
        let mut parser = UnifiedSSEParser::new(transformer);

        let test_data = b"data: {\"id\":\"chatcmpl-test\",\"object\":\"chat.completion.chunk\",\"created\":1640995200,\"model\":\"jamba-1.5-large\",\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":\"Hello\"},\"finish_reason\":null}]}\n\n";

        let result = parser.process_bytes(test_data).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].choices[0].delta.content,
            Some("Hello".to_string())
        );
    }

    #[test]
    fn test_done_message() {
        let transformer = OpenAICompatibleTransformer::new("ai21");
        let mut parser = UnifiedSSEParser::new(transformer);

        let test_data = b"data: [DONE]\n\n";
        let result = parser.process_bytes(test_data).unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_ai21_stream() {
        use futures::stream;

        let test_data = vec![
            Ok(Bytes::from(
                "data: {\"id\":\"test-1\",\"object\":\"chat.completion.chunk\",\"created\":1234567890,\"model\":\"jamba-1.5-large\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hello\"},\"finish_reason\":null}]}\n\n",
            )),
            Ok(Bytes::from("data: [DONE]\n\n")),
        ];

        let mock_stream = stream::iter(test_data);
        let mut ai21_stream = create_ai21_stream(mock_stream);

        // First chunk
        let chunk1 = ai21_stream.next().await;
        assert!(chunk1.is_some());
        let chunk1 = chunk1.unwrap().unwrap();
        assert_eq!(chunk1.choices[0].delta.content.as_ref().unwrap(), "Hello");

        // Stream should end after [DONE]
        let end = ai21_stream.next().await;
        assert!(end.is_none());
    }

    #[tokio::test]
    async fn test_ai21_stream_multiple_chunks() {
        use futures::stream;

        let test_data = vec![
            Ok(Bytes::from(
                "data: {\"id\":\"test-1\",\"object\":\"chat.completion.chunk\",\"created\":1234567890,\"model\":\"jamba-1.5-large\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hello\"},\"finish_reason\":null}]}\n\n",
            )),
            Ok(Bytes::from(
                "data: {\"id\":\"test-1\",\"object\":\"chat.completion.chunk\",\"created\":1234567890,\"model\":\"jamba-1.5-large\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\" World\"},\"finish_reason\":null}]}\n\n",
            )),
            Ok(Bytes::from("data: [DONE]\n\n")),
        ];

        let mock_stream = stream::iter(test_data);
        let mut ai21_stream = create_ai21_stream(mock_stream);

        // First chunk
        let chunk1 = ai21_stream.next().await.unwrap().unwrap();
        assert_eq!(chunk1.choices[0].delta.content.as_ref().unwrap(), "Hello");

        // Second chunk
        let chunk2 = ai21_stream.next().await.unwrap().unwrap();
        assert_eq!(chunk2.choices[0].delta.content.as_ref().unwrap(), " World");

        // Stream should end after [DONE]
        let end = ai21_stream.next().await;
        assert!(end.is_none());
    }
}
