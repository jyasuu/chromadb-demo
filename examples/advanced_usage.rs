use chromadb_demo::{ChromaClient, EmbeddingClient, Document};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // Load environment
    dotenv::dotenv().ok();

    let chroma_host = std::env::var("CHROMA_HOST")
        .unwrap_or_else(|_| "http://localhost:8000".to_string());
    let google_api_key = std::env::var("GOOGLE_API_KEY")
        .expect("GOOGLE_API_KEY must be set");

    // Initialize clients
    let chroma = ChromaClient::new(chroma_host);
    let embeddings = EmbeddingClient::new(google_api_key);

    // Collection name
    let collection_name = "advanced_demo";

    println!("🚀 Advanced ChromaDB Usage Demo");
    println!("================================");

    // Health check
    println!("\n1. Health Check");
    if chroma.health_check().await? {
        println!("✓ ChromaDB is healthy");
    } else {
        println!("✗ ChromaDB health check failed");
        return Ok(());
    }

    // Clean up existing collection
    println!("\n2. Collection Management");
    match chroma.delete_collection(collection_name).await {
        Ok(_) => println!("✓ Deleted existing collection"),
        Err(_) => println!("ℹ No existing collection to delete"),
    }

    // Create collection
    let collection = chroma.create_collection(collection_name).await?;
    println!("✓ Created collection: {}", collection.name);

    // Prepare diverse documents with rich metadata
    println!("\n3. Adding Documents with Rich Metadata");
    let docs = vec![
        Document {
            id: Uuid::new_v4().to_string(),
            content: "Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.".to_string(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("category".to_string(), "programming".to_string());
                m.insert("language".to_string(), "rust".to_string());
                m.insert("difficulty".to_string(), "intermediate".to_string());
                m.insert("year".to_string(), "2023".to_string());
                m
            },
        },
        Document {
            id: Uuid::new_v4().to_string(),
            content: "Python is a high-level programming language known for its simplicity and readability.".to_string(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("category".to_string(), "programming".to_string());
                m.insert("language".to_string(), "python".to_string());
                m.insert("difficulty".to_string(), "beginner".to_string());
                m.insert("year".to_string(), "2023".to_string());
                m
            },
        },
        Document {
            id: Uuid::new_v4().to_string(),
            content: "ChromaDB is an open-source embedding database that makes it easy to build LLM applications.".to_string(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("category".to_string(), "database".to_string());
                m.insert("type".to_string(), "vector".to_string());
                m.insert("difficulty".to_string(), "intermediate".to_string());
                m.insert("year".to_string(), "2023".to_string());
                m
            },
        },
        Document {
            id: Uuid::new_v4().to_string(),
            content: "Machine learning algorithms can learn patterns from data without being explicitly programmed.".to_string(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("category".to_string(), "ai".to_string());
                m.insert("field".to_string(), "machine_learning".to_string());
                m.insert("difficulty".to_string(), "advanced".to_string());
                m.insert("year".to_string(), "2023".to_string());
                m
            },
        },
        Document {
            id: Uuid::new_v4().to_string(),
            content: "Docker containers provide a lightweight way to package and deploy applications.".to_string(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("category".to_string(), "devops".to_string());
                m.insert("tool".to_string(), "docker".to_string());
                m.insert("difficulty".to_string(), "intermediate".to_string());
                m.insert("year".to_string(), "2023".to_string());
                m
            },
        },
    ];

    // Generate embeddings for all documents
    let texts: Vec<&str> = docs.iter().map(|d| d.content.as_str()).collect();
    let embeddings_vec = embeddings.embed_texts(&texts).await?;
    println!("✓ Generated {} embeddings", embeddings_vec.len());

    // Add documents to ChromaDB
    chroma.add_documents(collection_name, docs.clone(), embeddings_vec).await?;
    println!("✓ Added {} documents to collection", docs.len());

    // Count documents
    let count = chroma.count(collection_name).await?;
    println!("✓ Collection now contains {} documents", count);

    // Demonstrate various query patterns
    println!("\n4. Advanced Query Patterns");

    // Basic similarity search
    println!("\n4.1 Basic Similarity Search");
    let query_text = "Tell me about programming languages";
    let query_embedding = embeddings.embed_text(query_text).await?;
    let results = chroma.query(collection_name, vec![query_embedding], 3).await?;
    
    println!("Query: '{}'", query_text);
    for (i, (id, doc, distance)) in results.ids[0]
        .iter()
        .zip(results.documents[0].iter())
        .zip(results.distances[0].iter())
        .enumerate()
    {
        println!("  {}. [distance: {:.4}] {}", i + 1, distance, doc);
    }

    // Filtered search by category
    println!("\n4.2 Filtered Search by Category");
    let filter = json!({"category": "programming"});
    let query_embedding = embeddings.embed_text("easy to learn").await?;
    let filtered_results = chroma.query_with_filter(
        collection_name, 
        vec![query_embedding], 
        5, 
        Some(filter)
    ).await?;
    
    println!("Query: 'easy to learn' (filtered by category=programming)");
    for (i, (id, doc, distance)) in filtered_results.ids[0]
        .iter()
        .zip(filtered_results.documents[0].iter())
        .zip(filtered_results.distances[0].iter())
        .enumerate()
    {
        println!("  {}. [distance: {:.4}] {}", i + 1, distance, doc);
    }

    // Complex filter with multiple conditions
    println!("\n4.3 Complex Metadata Filtering");
    let complex_filter = json!({
        "$and": [
            {"difficulty": "intermediate"},
            {"year": "2023"}
        ]
    });
    
    let intermediate_docs = chroma.get_documents(
        collection_name,
        None,
        Some(complex_filter),
        Some(10)
    ).await?;
    
    println!("Documents with difficulty=intermediate AND year=2023:");
    for (i, doc) in intermediate_docs.documents[0].iter().enumerate() {
        println!("  {}. {}", i + 1, doc);
    }

    // Get specific documents by ID
    println!("\n4.4 Get Documents by ID");
    let first_doc_id = docs[0].id.clone();
    let specific_docs = chroma.get_documents(
        collection_name,
        Some(vec![first_doc_id.clone()]),
        None,
        None
    ).await?;
    
    println!("Document with ID {}:", first_doc_id);
    if let Some(doc) = specific_docs.documents[0].first() {
        println!("  Content: {}", doc);
    }

    // Update a document
    println!("\n5. Document Updates");
    let mut updated_doc = docs[0].clone();
    updated_doc.content = "Rust is a blazingly fast systems programming language with memory safety guarantees, updated content!".to_string();
    updated_doc.metadata.insert("updated".to_string(), "true".to_string());
    
    let updated_embedding = embeddings.embed_text(&updated_doc.content).await?;
    chroma.update_documents(
        collection_name,
        vec![updated_doc],
        vec![updated_embedding]
    ).await?;
    println!("✓ Updated document with new content");

    // Verify update
    let updated_docs = chroma.get_documents(
        collection_name,
        Some(vec![first_doc_id]),
        None,
        None
    ).await?;
    
    if let Some(doc) = updated_docs.documents[0].first() {
        println!("Updated content: {}", doc);
    }

    // Performance demonstration with batch operations
    println!("\n6. Batch Operations Performance");
    let batch_texts = vec![
        "Kubernetes orchestrates containerized applications",
        "GraphQL provides a flexible API query language",
        "WebAssembly enables high-performance web applications",
    ];
    
    let batch_embeddings = embeddings.embed_texts(&batch_texts).await?;
    println!("✓ Generated {} embeddings in batch", batch_embeddings.len());
    
    // Clean up
    println!("\n7. Cleanup");
    chroma.delete_collection(collection_name).await?;
    println!("✓ Deleted collection");

    println!("\n🎉 Advanced demo completed successfully!");
    println!("\nKey features demonstrated:");
    println!("  • Production-ready client configuration");
    println!("  • Real Gemini embeddings integration");
    println!("  • Metadata filtering and complex queries");
    println!("  • Document updates and batch operations");
    println!("  • Comprehensive error handling and logging");

    Ok(())
}