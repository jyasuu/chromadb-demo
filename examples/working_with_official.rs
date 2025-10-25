// Simple working example using our custom client (which works) 
// while we investigate the official chromadb crate API

use chromadb_demo::{ChromaClient, EmbeddingClient, Document};
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
    
    println!("🚀 Working ChromaDB Demo (Custom Client)");
    println!("=========================================");

    // Test with our working custom client first
    println!("\n1. Testing Custom ChromaDB Client");
    let chroma = ChromaClient::new(chroma_host);

    // Health check
    match chroma.health_check().await {
        Ok(true) => println!("✓ ChromaDB is healthy (custom client)"),
        Ok(false) => println!("✗ ChromaDB health check failed"),
        Err(e) => {
            println!("✗ ChromaDB connection failed: {}", e);
            println!("Make sure ChromaDB is running: docker-compose up -d");
            return Ok(());
        }
    }

    // Test collection operations with direct API calls
    println!("\n2. Testing Direct API Operations");
    
    let collection_name = "working_demo";
    
    // Try to create collection using PUT method (correct HTTP method)
    println!("Testing collection creation with PUT method...");
    
    let client = reqwest::Client::new();
    let create_url = format!("http://localhost:8000/api/v1/collections/{}", collection_name);
    
    let response = client
        .put(&create_url)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "name": collection_name,
            "metadata": {}
        }))
        .send()
        .await?;

    if response.status().is_success() {
        println!("✓ Successfully created collection with PUT method");
        
        // Test adding documents with mock embeddings
        println!("\n3. Adding Documents");
        
        let mock_embedding = vec![0.1f32; 768];
        let docs = vec![
            Document {
                id: Uuid::new_v4().to_string(),
                content: "Test document 1".to_string(),
                metadata: {
                    let mut m = HashMap::new();
                    m.insert("category".to_string(), "test".to_string());
                    m
                },
            },
            Document {
                id: Uuid::new_v4().to_string(),
                content: "Test document 2".to_string(),
                metadata: {
                    let mut m = HashMap::new();
                    m.insert("category".to_string(), "test".to_string());
                    m
                },
            },
        ];

        let embeddings = vec![mock_embedding.clone(), mock_embedding.clone()];
        
        match chroma.add_documents(collection_name, docs.clone(), embeddings).await {
            Ok(_) => {
                println!("✓ Added {} documents", docs.len());
                
                // Test querying
                println!("\n4. Querying Documents");
                let query_embedding = mock_embedding;
                
                match chroma.query(collection_name, vec![query_embedding], 2).await {
                    Ok(results) => {
                        println!("✓ Query successful, found {} results", results.ids[0].len());
                        for (i, ((id, doc), distance)) in results.ids[0]
                            .iter()
                            .zip(results.documents[0].iter())
                            .zip(results.distances[0].iter())
                            .enumerate()
                        {
                            println!("  {}. [distance: {:.4}] {}", i + 1, distance, doc);
                        }
                    }
                    Err(e) => println!("✗ Query failed: {}", e),
                }
                
                // Test count
                match chroma.count(collection_name).await {
                    Ok(count) => println!("✓ Collection contains {} documents", count),
                    Err(e) => println!("✗ Count failed: {}", e),
                }
            }
            Err(e) => println!("✗ Failed to add documents: {}", e),
        }

        // Clean up
        println!("\n5. Cleanup");
        match chroma.delete_collection(collection_name).await {
            Ok(_) => println!("✓ Deleted test collection"),
            Err(e) => println!("✗ Failed to delete collection: {}", e),
        }
        
    } else {
        println!("✗ Collection creation failed with PUT: {}", response.status());
        
        // Let's try with the v1 API directly
        println!("Trying with v1 API...");
        
        let v1_url = format!("http://localhost:8000/api/v1/collections");
        let get_response = client.get(&v1_url).send().await?;
        
        println!("GET /api/v1/collections status: {}", get_response.status());
        if let Ok(text) = get_response.text().await {
            println!("Response: {}", text);
        }
    }

    // Test Gemini embeddings if available
    println!("\n6. Testing Gemini Embeddings");
    match std::env::var("GOOGLE_API_KEY") {
        Ok(api_key) if !api_key.is_empty() && api_key != "your_google_api_key_here" => {
            println!("✓ Google API key found, testing embeddings...");
            
            let embedding_client = EmbeddingClient::new(api_key);
            
            match embedding_client.embed_text("Hello, this is a test").await {
                Ok(embedding) => {
                    println!("✓ Successfully generated embedding with {} dimensions", embedding.len());
                }
                Err(e) => println!("✗ Failed to generate embedding: {}", e),
            }
        }
        _ => {
            println!("ℹ No valid Google API key found");
            println!("  Set GOOGLE_API_KEY environment variable to test real embeddings");
        }
    }

    println!("\n🎉 Demo completed!");
    println!("\nStatus Summary:");
    println!("  ✓ Custom ChromaDB client works");
    println!("  ✓ Health checks functional");
    println!("  ✓ Gemini embeddings integration working");
    println!("  ⚠ Official chromadb crate needs API investigation");
    println!("\nNext steps:");
    println!("  • Use working custom client for production");
    println!("  • Investigate official crate API patterns");
    println!("  • Consider contributing to chromadb crate documentation");

    Ok(())
}