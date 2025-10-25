use crate::error::{ChromaError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use tracing::{debug, info, warn};

const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta";
const EMBEDDING_MODEL: &str = "models/gemini-embedding-exp-03-07";
const MAX_BATCH_SIZE: usize = 100; // Conservative batch limit  // 10
const EMBEDDING_DIMENSION: usize = 3072; // Updated based on actual Gemini response

#[derive(Debug, Serialize)]
struct EmbedRequest {
    requests: Vec<EmbedContentRequest>,
}

#[derive(Debug, Serialize)]
struct EmbedContentRequest {
    model: String,
    content: Content,
    #[serde(skip_serializing_if = "Option::is_none")]
    task_type: Option<String>,
}

#[derive(Debug, Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Debug, Serialize)]
struct Part {
    text: String,
}

#[derive(Debug, Deserialize)]
struct EmbedResponse {
    embeddings: Vec<ContentEmbedding>,
}

#[derive(Debug, Deserialize)]
struct ContentEmbedding {
    values: Vec<f32>,
}

pub struct EmbeddingClient {
    client: Client,
    api_key: String,
    max_retries: u32,
    retry_delay: Duration,
}

impl EmbeddingClient {
    pub fn new(api_key: String) -> Self {
        let timeout = Duration::from_millis(
            std::env::var("REQUEST_TIMEOUT_MS")
                .unwrap_or_else(|_| "60000".to_string())
                .parse()
                .unwrap_or(60000)
        );

        let client = Client::builder()
            .timeout(timeout)
            .connection_verbose(true)
            .build()
            .expect("Failed to create HTTP client");

        let max_retries = std::env::var("MAX_RETRIES")
            .unwrap_or_else(|_| "3".to_string())
            .parse()
            .unwrap_or(3);

        let retry_delay = Duration::from_millis(
            std::env::var("RETRY_DELAY_MS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000)
        );

        Self {
            client,
            api_key,
            max_retries,
            retry_delay,
        }
    }

    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        self.embed_texts(&[text])
            .await?
            .into_iter()
            .next()
            .ok_or_else(|| ChromaError::EmbeddingError("No embedding returned".to_string()))
    }

    pub async fn embed_texts(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        info!("Generating embeddings for {} texts", texts.len());
        
        // Process in batches to respect API limits
        let mut all_embeddings = Vec::new();
        
        for chunk in texts.chunks(MAX_BATCH_SIZE) {
            let batch_embeddings = self.embed_batch(chunk).await?;
            all_embeddings.extend(batch_embeddings);
        }

        info!("Successfully generated {} embeddings", all_embeddings.len());
        Ok(all_embeddings)
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let requests: Vec<EmbedContentRequest> = texts
            .iter()
            .map(|text| EmbedContentRequest {
                model: EMBEDDING_MODEL.to_string(),
                content: Content {
                    parts: vec![Part {
                        text: text.to_string(),
                    }],
                },
                task_type: Some("RETRIEVAL_DOCUMENT".to_string()),
            })
            .collect();

        let request_body = EmbedRequest { requests };

        let mut retries = 0;
        loop {
            match self.call_embedding_api(&request_body).await {
                Ok(embeddings) => {
                    debug!("Successfully generated {} embeddings", embeddings.len());
                    return Ok(embeddings);
                }
                Err(e) if retries < self.max_retries => {
                    retries += 1;
                    warn!(
                        "Embedding request failed (attempt {}/{}): {}. Retrying in {:?}",
                        retries, self.max_retries + 1, e, self.retry_delay
                    );
                    tokio::time::sleep(self.retry_delay * retries).await;
                }
                Err(e) => {
                    return Err(ChromaError::EmbeddingError(format!(
                        "Failed to generate embeddings after {} retries: {}",
                        self.max_retries, e
                    )));
                }
            }
        }
    }

    async fn call_embedding_api(&self, request: &EmbedRequest) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();
        
        // Process each request individually (following working rag.rs pattern)
        for embed_request in &request.requests {
            let url = format!("{}:embedContent", embed_request.model);
            let full_url = format!("{}/{}?key={}", GEMINI_API_BASE, url, self.api_key);
            
            let request_body = serde_json::json!({
                "content": embed_request.content
            });

            let response = self
                .client
                .post(&full_url)
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await?;

            // Add delay between requests to avoid rate limiting (from rag.rs)
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                return Err(ChromaError::EmbeddingError(format!(
                    "Gemini API error {}: {}",
                    status, error_text
                )));
            }

            let response_json: serde_json::Value = response.json().await?;
            
            let embedding_values = response_json["embedding"]["values"]
                .as_array()
                .ok_or_else(|| ChromaError::EmbeddingError("Invalid embedding response format".to_string()))?
                .iter()
                .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                .collect::<Vec<f32>>();

            if embedding_values.len() != EMBEDDING_DIMENSION {
                warn!(
                    "Unexpected embedding dimension: {} (expected {})",
                    embedding_values.len(),
                    EMBEDDING_DIMENSION
                );
            }
            
            embeddings.push(embedding_values);
        }

        Ok(embeddings)
    }

    pub fn get_embedding_dimension() -> usize {
        EMBEDDING_DIMENSION
    }
}
