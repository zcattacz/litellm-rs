//! Tests for the embedding module

use super::*;

// ==================== Integration Tests ====================

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_embedding_input_from_str() {
        let input: EmbeddingInput = "hello".into();
        assert_eq!(input.len(), 1);
    }

    #[test]
    fn test_embedding_input_from_vec_str() {
        let input: EmbeddingInput = vec!["a", "b", "c"].into();
        assert_eq!(input.len(), 3);
    }

    #[test]
    fn test_embedding_options_builder_chain() {
        let opts = EmbeddingOptions::new()
            .with_user("user-123")
            .with_dimensions(1536)
            .with_encoding_format("float")
            .with_api_key("sk-test")
            .with_api_base("https://api.example.com")
            .with_timeout(60)
            .with_task_type("RETRIEVAL_QUERY");

        assert_eq!(opts.user, Some("user-123".to_string()));
        assert_eq!(opts.dimensions, Some(1536));
        assert_eq!(opts.encoding_format, Some("float".to_string()));
        assert_eq!(opts.api_key, Some("sk-test".to_string()));
        assert_eq!(opts.api_base, Some("https://api.example.com".to_string()));
        assert_eq!(opts.timeout, Some(60));
        assert_eq!(opts.task_type, Some("RETRIEVAL_QUERY".to_string()));
    }

    #[test]
    fn test_embedding_options_with_headers() {
        let opts = EmbeddingOptions::new()
            .with_header("X-Custom", "value1")
            .with_header("X-Another", "value2");

        let headers = opts.headers.unwrap();
        assert_eq!(headers.len(), 2);
        assert_eq!(headers.get("X-Custom"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_embedding_options_serialization() {
        let opts = EmbeddingOptions::new()
            .with_dimensions(256)
            .with_user("test");

        let json = serde_json::to_value(&opts).unwrap();
        assert_eq!(json["dimensions"], 256);
        assert_eq!(json["user"], "test");
        // None values should be skipped
        assert!(!json.as_object().unwrap().contains_key("api_key"));
    }
}

// ==================== Helper Function Tests ====================

#[cfg(test)]
mod helper_tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical_vectors() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal_vectors() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_opposite_vectors() {
        let a = vec![1.0, 1.0];
        let b = vec![-1.0, -1.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_mismatched_lengths() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0];
        let sim = cosine_similarity(&a, &b);
        assert_eq!(sim, 0.0);
    }

    #[test]
    fn test_euclidean_distance_same_point() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        let dist = euclidean_distance(&a, &b);
        assert!((dist - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_euclidean_distance_3_4_5_triangle() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        let dist = euclidean_distance(&a, &b);
        assert!((dist - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_dot_product_basic() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let result = dot_product(&a, &b);
        // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
        assert!((result - 32.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_vector() {
        let v = vec![3.0, 4.0];
        let normalized = normalize(&v);
        // Should be [0.6, 0.8]
        assert!((normalized[0] - 0.6).abs() < 1e-6);
        assert!((normalized[1] - 0.8).abs() < 1e-6);

        // Magnitude should be 1
        let mag: f32 = normalized.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((mag - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_zero_vector() {
        let v = vec![0.0, 0.0, 0.0];
        let normalized = normalize(&v);
        assert_eq!(normalized, vec![0.0, 0.0, 0.0]);
    }
}

// ==================== Type Tests ====================

#[cfg(test)]
mod type_tests {
    use super::*;

    #[test]
    fn test_embedding_input_text() {
        let input = EmbeddingInput::text("hello world");
        assert_eq!(input.len(), 1);
        assert!(!input.is_empty());
    }

    #[test]
    fn test_embedding_input_texts() {
        let input = EmbeddingInput::texts(["a", "b", "c"]);
        assert_eq!(input.len(), 3);
    }

    #[test]
    fn test_embedding_input_to_vec() {
        let input = EmbeddingInput::texts(["x", "y"]);
        let vec = input.to_vec();
        assert_eq!(vec.len(), 2);
        assert_eq!(vec[0], "x");
        assert_eq!(vec[1], "y");
    }

    #[test]
    fn test_embedding_input_iter() {
        let input = EmbeddingInput::texts(["a", "b", "c"]);
        let items: Vec<_> = input.iter().collect();
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_embedding_input_empty() {
        let empty_text = EmbeddingInput::text("");
        assert!(empty_text.is_empty());

        let empty_array = EmbeddingInput::texts(Vec::<String>::new());
        assert!(empty_array.is_empty());
    }

    #[test]
    fn test_embedding_input_serialization() {
        let single = EmbeddingInput::text("hello");
        let json = serde_json::to_value(&single).unwrap();
        assert_eq!(json, "hello");

        let multiple = EmbeddingInput::texts(["a", "b"]);
        let json = serde_json::to_value(&multiple).unwrap();
        assert!(json.is_array());
    }
}

// ==================== Router Tests ====================

#[cfg(test)]
mod router_tests {
    use super::router::EmbeddingRouter;

    #[test]
    fn test_parse_model_with_provider_prefix() {
        let (provider, model) = EmbeddingRouter::parse_model("openai/text-embedding-ada-002");
        assert_eq!(provider, "openai");
        assert_eq!(model, "text-embedding-ada-002");
    }

    #[test]
    fn test_parse_model_without_provider() {
        let (provider, model) = EmbeddingRouter::parse_model("text-embedding-ada-002");
        assert_eq!(provider, "openai");
        assert_eq!(model, "text-embedding-ada-002");
    }

    #[test]
    fn test_parse_model_azure() {
        let (provider, model) = EmbeddingRouter::parse_model("azure/text-embedding-3-small");
        assert_eq!(provider, "azure");
        assert_eq!(model, "text-embedding-3-small");
    }

    #[test]
    fn test_parse_model_voyage() {
        let (provider, model) = EmbeddingRouter::parse_model("voyage/voyage-3");
        assert_eq!(provider, "voyage");
        assert_eq!(model, "voyage-3");
    }

    #[test]
    fn test_parse_model_cohere() {
        let (provider, model) = EmbeddingRouter::parse_model("cohere/embed-english-v3.0");
        assert_eq!(provider, "cohere");
        assert_eq!(model, "embed-english-v3.0");
    }
}

// ==================== Similarity Computation Tests ====================

#[cfg(test)]
mod similarity_tests {
    use super::*;

    #[test]
    fn test_semantic_similarity_ranking() {
        // Simulate embeddings for semantic similarity testing
        // In practice, these would come from an actual embedding model

        // "Hello world" embedding (simulated)
        let hello = vec![0.5, 0.5, 0.0, 0.0];
        // "Hi there" embedding - should be similar to hello
        let hi = vec![0.45, 0.55, 0.05, 0.0];
        // "Goodbye world" embedding - somewhat different
        let goodbye = vec![-0.3, 0.4, 0.3, 0.0];
        // "Programming in Rust" - completely different topic
        let rust = vec![0.0, 0.0, 0.6, 0.4];

        let hello_hi = cosine_similarity(&hello, &hi);
        let hello_goodbye = cosine_similarity(&hello, &goodbye);
        let hello_rust = cosine_similarity(&hello, &rust);

        // "Hi there" should be more similar to "Hello world" than "Goodbye world"
        assert!(hello_hi > hello_goodbye);
        // "Goodbye world" should be more similar to "Hello world" than "Programming in Rust"
        assert!(hello_goodbye > hello_rust);
    }

    #[test]
    fn test_normalized_vectors_cosine() {
        let a = normalize(&[1.0, 2.0, 3.0]);
        let b = normalize(&[4.0, 5.0, 6.0]);

        let sim = cosine_similarity(&a, &b);
        // For normalized vectors, cosine similarity equals dot product
        let dot = dot_product(&a, &b);
        assert!((sim - dot).abs() < 1e-6);
    }

    #[test]
    fn test_distance_similarity_inverse() {
        // For normalized vectors, smaller distance = higher similarity
        let a = normalize(&[1.0, 0.0]);
        let b = normalize(&[0.8, 0.6]);
        let c = normalize(&[-1.0, 0.0]);

        let dist_ab = euclidean_distance(&a, &b);
        let dist_ac = euclidean_distance(&a, &c);

        let sim_ab = cosine_similarity(&a, &b);
        let sim_ac = cosine_similarity(&a, &c);

        // Smaller distance should correspond to higher similarity
        assert!(dist_ab < dist_ac);
        assert!(sim_ab > sim_ac);
    }
}

// ==================== Edge Case Tests ====================

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_embedding_options() {
        let opts = EmbeddingOptions::default();
        let json = serde_json::to_value(&opts).unwrap();
        // Should have empty extra_params and stream: false
        assert!(json["extra_params"].as_object().unwrap().is_empty());
    }

    #[test]
    fn test_large_dimension_embedding() {
        // Test with OpenAI ada-002 dimension (1536)
        let v: Vec<f32> = (0..1536).map(|i| (i as f32) * 0.001).collect();
        let normalized = normalize(&v);

        // Verify normalization
        let mag: f32 = normalized.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((mag - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_very_small_values() {
        let a = vec![1e-10, 1e-10, 1e-10];
        let b = vec![1e-10, 1e-10, 1e-10];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_negative_embedding_values() {
        let a = vec![-0.5, 0.5, -0.5, 0.5];
        let b = vec![-0.5, 0.5, -0.5, 0.5];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_single_dimension_embedding() {
        let a = vec![1.0];
        let b = vec![1.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-6);

        let c = vec![-1.0];
        let sim_opposite = cosine_similarity(&a, &c);
        assert!((sim_opposite - (-1.0)).abs() < 1e-6);
    }
}
