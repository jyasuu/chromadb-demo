use crate::error::{ChromaError, Result};
use crate::embeddings::EmbeddingClient;
use chromadb::{ChromaClient as OfficialChromaClient, ChromaClientOptions, ChromaCollection};
use futures::TryFutureExt;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, info, warn};

pub struct ChromaDBWrapper {
    client: OfficialChromaClient,
    embedding_client: EmbeddingClient,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub struct QueryResult {
    pub ids: Vec<String>,
    pub documents: Vec<String>,
    pub distances: Vec<f32>,
    pub metadatas: Vec<HashMap<String, String>>,
}

impl ChromaDBWrapper {
    pub async fn new(chroma_url: &str, google_api_key: String) -> Result<Self> {
        let options = ChromaClientOptions::new(chroma_url.to_string());
        let client = OfficialChromaClient::new(options)
            .await
            .map_err(|e| ChromaError::ApiError(format!("Failed to create ChromaDB client: {}", e)))?;

        let embedding_client = EmbeddingClient::new(google_api_key);

        info!("ChromaDB official client initialized with URL: {}", chroma_url);

        Ok(Self {
            client,
            embedding_client,
        })
    }

    pub async fn health_check(&self) -> Result<bool> {
        match self.client.heartbeat().await {
            Ok(_) => {
                debug!("ChromaDB health check passed");
                Ok(true)
            }
            Err(e) => {
                warn!("ChromaDB health check failed: {}", e);
                Ok(false)
            }
        }
    }

    pub async fn create_collection(&self, name: &str) -> Result<()> {
        match self.client.create_collection(name, None, false).await {
            Ok(_) => {
                info!("Successfully created collection: {}", name);
                Ok(())
            }
            Err(e) => Err(ChromaError::CollectionError(format!(
                "Failed to create collection '{}': {}",
                name, e
            ))),
        }
    }

    pub async fn delete_collection(&self, name: &str) -> Result<()> {
        match self.client.delete_collection(name).await {
            Ok(_) => {
                info!("Successfully deleted collection: {}", name);
                Ok(())
            }
            Err(e) => Err(ChromaError::CollectionError(format!(
                "Failed to delete collection '{}': {}",
                name, e
            ))),
        }
    }

    pub async fn get_collection(&self, name: &str) -> Result<ChromaCollection> {
        self.client
            .get_collection(name)
            .await
            .map_err(|e| ChromaError::CollectionError(format!(
                "Failed to get collection '{}': {}",
                name, e
            )))
    }

    pub async fn list_collections(&self) -> Result<Vec<String>> {
        match self.client.list_collections().await {
            Ok(collections) => {
                let names = collections.into_iter().map(|c| c.name()).collect();
                Ok(names)
            }
            Err(e) => Err(ChromaError::ApiError(format!(
                "Failed to list collections: {}",
                e
            ))),
        }
    }

    pub async fn add_documents(&self, collection_name: &str, documents: Vec<Document>) -> Result<()> {
        if documents.is_empty() {
            return Ok(());
        }

        info!("Adding {} documents to collection '{}'", documents.len(), collection_name);

        // Generate embeddings for all documents
        let texts: Vec<&str> = documents.iter().map(|d| d.content.as_str()).collect();
        let embeddings = self.embedding_client.embed_texts(&texts).await?;

        // Get the collection
        let collection = self.get_collection(collection_name).await?;

        // Prepare data for ChromaDB
        let ids: Vec<String> = documents.iter().map(|d| d.id.clone()).collect();
        let doc_texts: Vec<String> = documents.iter().map(|d| d.content.clone()).collect();
        let metadatas: Vec<Value> = documents
            .iter()
            .map(|d| {
                let mut map = serde_json::Map::new();
                for (k, v) in &d.metadata {
                    map.insert(k.clone(), Value::String(v.clone()));
                }
                Value::Object(map)
            })
            .collect();

        // Add to ChromaDB
        match collection
            .add(ids, embeddings, Some(metadatas), Some(doc_texts))
            .await
        {
            Ok(_) => {
                info!("Successfully added {} documents", documents.len());
                Ok(())
            }
            Err(e) => Err(ChromaError::ApiError(format!(
                "Failed to add documents: {}",
                e
            ))),
        }
    }

    pub async fn query(
        &self,
        collection_name: &str,
        query_text: &str,
        n_results: usize,
        filter: Option<Value>,
    ) -> Result<QueryResult> {
        info!("Querying collection '{}' with text: '{}'", collection_name, query_text);

        // Generate embedding for query
        let query_embedding = self.embedding_client.embed_text(query_text).await?;

        // Get the collection
        let collection = self.get_collection(collection_name).await?;

        // Query ChromaDB
        match collection
            .query(
                vec![query_embedding],
                n_results,
                filter,
                None, // include
                None, // where_document
            )
            .await
        {
            Ok(results) => {
                debug!("Query returned {} results", results.ids.len());

                // Extract first query results (we only sent one query)
                let query_results = if !results.ids.is_empty() {
                    QueryResult {
                        ids: results.ids[0].clone(),
                        documents: results
                            .documents
                            .map(|docs| docs[0].clone())
                            .unwrap_or_default(),
                        distances: results.distances.map(|d| d[0].clone()).unwrap_or_default(),
                        metadatas: results
                            .metadatas
                            .map(|metas| {
                                metas[0]
                                    .iter()
                                    .map(|meta| {
                                        let mut map = HashMap::new();
                                        if let Value::Object(obj) = meta {
                                            for (k, v) in obj {
                                                if let Value::String(s) = v {
                                                    map.insert(k.clone(), s.clone());
                                                }
                                            }
                                        }
                                        map
                                    })
                                    .collect()
                            })
                            .unwrap_or_default(),
                    }
                } else {
                    QueryResult {
                        ids: vec![],
                        documents: vec![],
                        distances: vec![],
                        metadatas: vec![],
                    }
                };

                Ok(query_results)
            }
            Err(e) => Err(ChromaError::ApiError(format!("Query failed: {}", e))),
        }
    }

    pub async fn count_documents(&self, collection_name: &str) -> Result<usize> {
        let collection = self.get_collection(collection_name).await?;

        match collection.count().await {
            Ok(count) => Ok(count as usize),
            Err(e) => Err(ChromaError::ApiError(format!(
                "Failed to count documents: {}",
                e
            ))),
        }
    }

    pub async fn delete_documents(&self, collection_name: &str, ids: Vec<String>) -> Result<()> {
        let collection = self.get_collection(collection_name).await?;

        match collection.delete(Some(ids), None, None).await {
            Ok(_) => {
                info!("Successfully deleted documents");
                Ok(())
            }
            Err(e) => Err(ChromaError::ApiError(format!(
                "Failed to delete documents: {}",
                e
            ))),
        }
    }
}