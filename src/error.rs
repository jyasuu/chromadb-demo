use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChromaError {
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    SerializeError(#[from] serde_json::Error),
    
    #[error("API error: {0}")]
    ApiError(String),
    
    #[error("Embedding error: {0}")]
    EmbeddingError(String),
    
    #[error("Collection error: {0}")]
    CollectionError(String),
}

pub type Result<T> = std::result::Result<T, ChromaError>;
