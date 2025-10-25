mod chroma_client;
mod embeddings;
mod error;
mod models;

use chroma_client::ChromaClient;
use embeddings::EmbeddingClient;
use models::Document;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string())
        )
        .init();

    // Load environment variables
    dotenv::dotenv().ok();
    
    let chroma_host = std::env::var("CHROMA_HOST")
        .unwrap_or_else(|_| "http://localhost:8000".to_string());
    let google_api_key = std::env::var("GOOGLE_API_KEY")
        .expect("GOOGLE_API_KEY must be set");
    let collection_name = std::env::var("COLLECTION_NAME")
        .unwrap_or_else(|_| "documents".to_string());

    // Initialize clients
    let chroma = ChromaClient::new(chroma_host);
    let embeddings = EmbeddingClient::new(google_api_key);

    // Health check
    println!("✓ Checking ChromaDB health...");
    if chroma.health_check().await? {
        println!("✓ ChromaDB is running");
    } else {
        println!("✗ ChromaDB is not accessible");
        return Ok(());
    }

    // Create collection
    println!("\n✓ Creating collection: {}", collection_name);
    match chroma.create_collection(&collection_name).await {
        Ok(col) => println!("✓ Collection created: {:?}", col.name),
        Err(e) => println!("ℹ Collection may already exist: {}", e),
    }

    // Prepare documents
    let docs = vec![
        Document {
            id: Uuid::new_v4().to_string(),
            content: "Rust is a systems programming language".to_string(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("source".to_string(), "documentation".to_string());
                m.insert("language".to_string(), "rust".to_string());
                m
            },
        },
        Document {
            id: Uuid::new_v4().to_string(),
            content: "ChromaDB is a vector database for AI applications".to_string(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("source".to_string(), "documentation".to_string());
                m.insert("database".to_string(), "chromadb".to_string());
                m
            },
        },
        Document {
            id: Uuid::new_v4().to_string(),
            content: "Gemini AI models provide powerful embeddings".to_string(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("source".to_string(), "documentation".to_string());
                m.insert("model".to_string(), "gemini".to_string());
                m
            },
        },
    ];

    // Generate embeddings
    println!("\n✓ Generating embeddings...");
    let texts: Vec<&str> = docs.iter().map(|d| d.content.as_str()).collect();
    let embeddings_vec = embeddings.embed_texts(&texts).await?;
    println!("✓ Generated {} embeddings", embeddings_vec.len());

    // Add documents to collection
    println!("\n✓ Adding documents to ChromaDB...");
    chroma.add_documents(
        &collection_name,
        docs.clone(),
        embeddings_vec,
    )
    .await?;
    println!("✓ Documents added successfully");

    // Count documents
    let count = chroma.count(&collection_name).await?;
    println!("✓ Collection now contains {} documents", count);

    // Query documents
    println!("\n✓ Querying similar documents...");
    let query_text = "Tell me about programming languages";
    let query_embedding = embeddings.embed_text(query_text).await?;
    
    let results = chroma.query(
        &collection_name,
        vec![query_embedding],
        3,
    )
    .await?;

    println!("✓ Query: '{}'", query_text);
    println!("✓ Top {} results:", results.ids[0].len());
    
    for (i, ((id, doc), distance)) in results.ids[0]
        .iter()
        .zip(results.documents[0].iter())
        .zip(results.distances[0].iter())
        .enumerate()
    {
        println!("  {}. [distance: {:.4}] {}", i + 1, distance, doc);
    }

    Ok(())
}
