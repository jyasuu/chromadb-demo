pub mod chroma_client;
// pub mod chroma_official; // Temporarily disabled while investigating API
pub mod embeddings;
pub mod error;
pub mod models;

pub use chroma_client::ChromaClient;
// pub use chroma_official::{ChromaDBWrapper, Document as OfficialDocument, QueryResult};
pub use embeddings::EmbeddingClient;
pub use error::{ChromaError, Result};
pub use models::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_chroma_client_creation() {
        let client = ChromaClient::new("http://localhost:8000".to_string());
        // Test that client creation doesn't panic
        assert!(true);
    }

    #[tokio::test]
    async fn test_embedding_client_creation() {
        let client = EmbeddingClient::new("test_api_key".to_string());
        // Test that client creation doesn't panic
        assert!(true);
    }

    #[tokio::test]
    async fn test_document_creation() {
        let doc = Document {
            id: Uuid::new_v4().to_string(),
            content: "Test document content".to_string(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("source".to_string(), "test".to_string());
                m
            },
        };
        
        assert!(!doc.id.is_empty());
        assert_eq!(doc.content, "Test document content");
        assert_eq!(doc.metadata.get("source").unwrap(), "test");
    }

    #[tokio::test]
    async fn test_embedding_dimension() {
        let dimension = EmbeddingClient::get_embedding_dimension();
        assert_eq!(dimension, 768);
    }
}