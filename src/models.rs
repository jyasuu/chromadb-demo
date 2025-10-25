use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedResult {
    pub embeddings: Vec<Vec<f32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddRequest {
    pub ids: Vec<String>,
    pub embeddings: Vec<Vec<f32>>,
    pub metadatas: Vec<HashMap<String, String>>,
    pub documents: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryRequest {
    pub query_embeddings: Vec<Vec<f32>>,
    pub n_results: u32,
    pub where_filter: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct QueryResponse {
    pub ids: Vec<Vec<String>>,
    pub embeddings: Option<Vec<Vec<Vec<f32>>>>,
    pub documents: Vec<Vec<String>>,
    pub metadatas: Vec<Vec<serde_json::Value>>,
    pub distances: Vec<Vec<f32>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionResponse {
    pub name: String,
    pub id: String,
    pub metadata: Option<serde_json::Value>,
}
