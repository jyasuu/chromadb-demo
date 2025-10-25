use crate::error::{ChromaError, Result};
use crate::models::*;
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info, warn, error};
use url::Url;

pub struct ChromaClient {
    base_url: String,
    http_client: Client,
    max_retries: u32,
    retry_delay: Duration,
}

impl ChromaClient {
    pub fn new(base_url: String) -> Self {
        // Validate and normalize URL
        let base_url = Self::validate_url(&base_url)
            .unwrap_or_else(|_| {
                warn!("Invalid URL provided, using default: http://localhost:8000");
                "http://localhost:8000".to_string()
            });

        let connection_timeout = Duration::from_millis(
            std::env::var("CONNECTION_TIMEOUT_MS")
                .unwrap_or_else(|_| "30000".to_string())
                .parse()
                .unwrap_or(30000)
        );

        let request_timeout = Duration::from_millis(
            std::env::var("REQUEST_TIMEOUT_MS")
                .unwrap_or_else(|_| "60000".to_string())
                .parse()
                .unwrap_or(60000)
        );

        let http_client = Client::builder()
            .connect_timeout(connection_timeout)
            .timeout(request_timeout)
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(60))
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

        info!("ChromaClient initialized with base_url: {}", base_url);

        Self {
            base_url,
            http_client,
            max_retries,
            retry_delay,
        }
    }

    fn validate_url(url: &str) -> Result<String> {
        let parsed = Url::parse(url)
            .map_err(|e| ChromaError::ApiError(format!("Invalid URL: {}", e)))?;
        
        if !matches!(parsed.scheme(), "http" | "https") {
            return Err(ChromaError::ApiError("URL must use HTTP or HTTPS".to_string()));
        }
        
        Ok(url.trim_end_matches('/').to_string())
    }

    async fn execute_with_retry<T, F, Fut>(&self, operation_name: &str, mut f: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut retries = 0;
        loop {
            match f().await {
                Ok(result) => {
                    if retries > 0 {
                        info!("{} succeeded after {} retries", operation_name, retries);
                    }
                    return Ok(result);
                }
                Err(e) if retries < self.max_retries && Self::is_retryable_error(&e) => {
                    retries += 1;
                    let delay = self.retry_delay * retries;
                    warn!(
                        "{} failed (attempt {}/{}): {}. Retrying in {:?}",
                        operation_name, retries, self.max_retries + 1, e, delay
                    );
                    tokio::time::sleep(delay).await;
                }
                Err(e) => {
                    error!("{} failed after {} retries: {}", operation_name, retries, e);
                    return Err(e);
                }
            }
        }
    }

    fn is_retryable_error(error: &ChromaError) -> bool {
        match error {
            ChromaError::RequestError(reqwest_error) => {
                reqwest_error.is_timeout() || reqwest_error.is_connect()
            }
            ChromaError::ApiError(msg) => {
                // Retry on 5xx server errors
                msg.contains("500") || msg.contains("502") || msg.contains("503") || msg.contains("504")
            }
            _ => false,
        }
    }

    pub async fn health_check(&self) -> Result<bool> {
        self.execute_with_retry("health_check", || async {
            let response = self.http_client
                .get(&format!("{}/api/v2/heartbeat", self.base_url))
                .send()
                .await?;
            
            if response.status().is_success() {
                debug!("ChromaDB health check passed");
                Ok(true)
            } else {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                Err(ChromaError::ApiError(format!(
                    "Health check failed with status {}: {}", status, error_text
                )))
            }
        }).await
    }

    pub async fn create_collection(&self, name: &str) -> Result<CollectionResponse> {
        let response = self.http_client
            .post(&format!("{}/api/v2/collections", self.base_url))
            .json(&json!({
                "name": name,
                "metadata": {"hnsw:space": "cosine"}
            }))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(ChromaError::CollectionError(
                format!("Failed to create collection: {}", response.status())
            ))
        }
    }

    pub async fn get_collection(&self, name: &str) -> Result<CollectionResponse> {
        let response = self.http_client
            .get(&format!("{}/api/v2/collections/{}", self.base_url, name))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(ChromaError::CollectionError(
                format!("Collection not found: {}", name)
            ))
        }
    }

    pub async fn delete_collection(&self, name: &str) -> Result<()> {
        let response = self.http_client
            .delete(&format!("{}/api/v2/collections/{}", self.base_url, name))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(ChromaError::CollectionError(
                format!("Failed to delete collection: {}", response.status())
            ))
        }
    }

    pub async fn add_documents(
        &self,
        collection_name: &str,
        documents: Vec<Document>,
        embeddings: Vec<Vec<f32>>,
    ) -> Result<()> {
        let ids: Vec<String> = documents.iter().map(|d| d.id.clone()).collect();
        let docs: Vec<String> = documents.iter().map(|d| d.content.clone()).collect();
        let metadatas: Vec<HashMap<String, String>> = 
            documents.iter().map(|d| d.metadata.clone()).collect();

        let request = AddRequest {
            ids,
            embeddings,
            metadatas,
            documents: docs,
        };

        let response = self.http_client
            .post(&format!(
                "{}/api/v2/collections/{}/add",
                self.base_url, collection_name
            ))
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(ChromaError::ApiError(
                format!("Failed to add documents: {}", error_text)
            ))
        }
    }

    pub async fn query(
        &self,
        collection_name: &str,
        query_embeddings: Vec<Vec<f32>>,
        n_results: u32,
    ) -> Result<QueryResponse> {
        self.query_with_filter(collection_name, query_embeddings, n_results, None).await
    }

    pub async fn query_with_filter(
        &self,
        collection_name: &str,
        query_embeddings: Vec<Vec<f32>>,
        n_results: u32,
        where_filter: Option<serde_json::Value>,
    ) -> Result<QueryResponse> {
        self.execute_with_retry("query", || async {
            let request = QueryRequest {
                query_embeddings: query_embeddings.clone(),
                n_results,
                where_filter: where_filter.clone(),
            };

            let response = self.http_client
                .post(&format!(
                    "{}/api/v2/collections/{}/query",
                    self.base_url, collection_name
                ))
                .json(&request)
                .send()
                .await?;

            if response.status().is_success() {
                let query_response: QueryResponse = response.json().await?;
                debug!("Query returned {} results", 
                    query_response.ids.get(0).map(|ids| ids.len()).unwrap_or(0));
                Ok(query_response)
            } else {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                Err(ChromaError::ApiError(
                    format!("Query failed with status {}: {}", status, error_text)
                ))
            }
        }).await
    }

    pub async fn get_documents(
        &self,
        collection_name: &str,
        ids: Option<Vec<String>>,
        where_filter: Option<serde_json::Value>,
        limit: Option<u32>,
    ) -> Result<QueryResponse> {
        self.execute_with_retry("get_documents", || async {
            let mut request = json!({});
            
            if let Some(ids) = &ids {
                request["ids"] = json!(ids);
            }
            
            if let Some(filter) = &where_filter {
                request["where"] = filter.clone();
            }
            
            if let Some(limit) = limit {
                request["limit"] = json!(limit);
            }

            let response = self.http_client
                .post(&format!(
                    "{}/api/v2/collections/{}/get",
                    self.base_url, collection_name
                ))
                .json(&request)
                .send()
                .await?;

            if response.status().is_success() {
                Ok(response.json().await?)
            } else {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                Err(ChromaError::ApiError(
                    format!("Get documents failed with status {}: {}", status, error_text)
                ))
            }
        }).await
    }

    pub async fn update_documents(
        &self,
        collection_name: &str,
        documents: Vec<Document>,
        embeddings: Vec<Vec<f32>>,
    ) -> Result<()> {
        self.execute_with_retry("update_documents", || async {
            let ids: Vec<String> = documents.iter().map(|d| d.id.clone()).collect();
            let docs: Vec<String> = documents.iter().map(|d| d.content.clone()).collect();
            let metadatas: Vec<HashMap<String, String>> = 
                documents.iter().map(|d| d.metadata.clone()).collect();

            let request = json!({
                "ids": ids,
                "embeddings": embeddings,
                "metadatas": metadatas,
                "documents": docs,
            });

            let response = self.http_client
                .post(&format!(
                    "{}/api/v2/collections/{}/update",
                    self.base_url, collection_name
                ))
                .json(&request)
                .send()
                .await?;

            if response.status().is_success() {
                info!("Successfully updated {} documents", documents.len());
                Ok(())
            } else {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                Err(ChromaError::ApiError(
                    format!("Update documents failed with status {}: {}", status, error_text)
                ))
            }
        }).await
    }

    pub async fn delete_documents(
        &self,
        collection_name: &str,
        ids: Vec<String>,
    ) -> Result<()> {
        let response = self.http_client
            .post(&format!(
                "{}/api/v2/collections/{}/delete",
                self.base_url, collection_name
            ))
            .json(&json!({ "ids": ids }))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(ChromaError::ApiError(
                format!("Delete failed: {}", response.status())
            ))
        }
    }

    pub async fn count(&self, collection_name: &str) -> Result<usize> {
        let response = self.http_client
            .get(&format!(
                "{}/api/v2/collections/{}/count",
                self.base_url, collection_name
            ))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(ChromaError::ApiError(
                format!("Count failed: {}", response.status())
            ))
        }
    }
}
