use chromadb_demo::{ChromaDBWrapper, OfficialDocument};
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
    
    let google_api_key = match std::env::var("GOOGLE_API_KEY") {
        Ok(key) if !key.is_empty() && key != "your_google_api_key_here" => key,
        _ => {
            println!("❌ GOOGLE_API_KEY not found or invalid");
            println!("Please set a valid Google API key in your .env file");
            println!("Example: GOOGLE_API_KEY=your_actual_api_key_here");
            return Ok(());
        }
    };

    println!("🚀 Official ChromaDB Client Demo");
    println!("================================");

    // Initialize the wrapper client
    println!("\n1. Initializing ChromaDB Client");
    let client = match ChromaDBWrapper::new(&chroma_host, google_api_key).await {
        Ok(client) => {
            println!("✓ ChromaDB client initialized successfully");
            client
        }
        Err(e) => {
            println!("❌ Failed to initialize ChromaDB client: {}", e);
            println!("Make sure ChromaDB is running: docker-compose up -d");
            return Ok(());
        }
    };

    // Health check
    println!("\n2. Health Check");
    match client.health_check().await {
        Ok(true) => println!("✓ ChromaDB is healthy and accessible"),
        Ok(false) => {
            println!("❌ ChromaDB health check failed");
            return Ok(());
        }
        Err(e) => {
            println!("❌ Health check error: {}", e);
            return Ok(());
        }
    }

    // List existing collections
    println!("\n3. Listing Collections");
    match client.list_collections().await {
        Ok(collections) => {
            if collections.is_empty() {
                println!("ℹ No collections found");
            } else {
                println!("Found {} collections:", collections.len());
                for collection in &collections {
                    println!("  - {}", collection);
                }
            }
        }
        Err(e) => println!("⚠ Could not list collections: {}", e),
    }

    // Collection management
    let collection_name = "official_demo";
    
    println!("\n4. Collection Management");
    
    // Clean up existing collection if any
    match client.delete_collection(collection_name).await {
        Ok(_) => println!("✓ Deleted existing collection"),
        Err(_) => println!("ℹ No existing collection to delete"),
    }

    // Create new collection
    match client.create_collection(collection_name).await {
        Ok(_) => println!("✓ Created collection: {}", collection_name),
        Err(e) => {
            println!("❌ Failed to create collection: {}", e);
            return Ok(());
        }
    }

    // Prepare sample documents
    println!("\n5. Preparing Documents with Real Embeddings");
    
    let docs = vec![
        OfficialDocument {
            id: Uuid::new_v4().to_string(),
            content: "Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.".to_string(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("category".to_string(), "programming".to_string());
                m.insert("language".to_string(), "rust".to_string());
                m.insert("difficulty".to_string(), "intermediate".to_string());
                m
            },
        },
        OfficialDocument {
            id: Uuid::new_v4().to_string(),
            content: "Python is a high-level programming language known for its simplicity and readability.".to_string(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("category".to_string(), "programming".to_string());
                m.insert("language".to_string(), "python".to_string());
                m.insert("difficulty".to_string(), "beginner".to_string());
                m
            },
        },
        OfficialDocument {
            id: Uuid::new_v4().to_string(),
            content: "ChromaDB is an open-source embedding database that makes it easy to build LLM applications.".to_string(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("category".to_string(), "database".to_string());
                m.insert("type".to_string(), "vector".to_string());
                m.insert("difficulty".to_string(), "intermediate".to_string());
                m
            },
        },
        OfficialDocument {
            id: Uuid::new_v4().to_string(),
            content: "Machine learning algorithms can learn patterns from data without being explicitly programmed.".to_string(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("category".to_string(), "ai".to_string());
                m.insert("field".to_string(), "machine_learning".to_string());
                m.insert("difficulty".to_string(), "advanced".to_string());
                m
            },
        },
        OfficialDocument {
            id: Uuid::new_v4().to_string(),
            content: "Docker containers provide a lightweight way to package and deploy applications.".to_string(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("category".to_string(), "devops".to_string());
                m.insert("tool".to_string(), "docker".to_string());
                m.insert("difficulty".to_string(), "intermediate".to_string());
                m
            },
        },
    ];

    // Add documents to collection
    println!("📄 Adding {} documents to collection...", docs.len());
    println!("⏳ This will generate real embeddings using Gemini API (may take a moment)...");
    
    match client.add_documents(collection_name, docs.clone()).await {
        Ok(_) => println!("✓ Successfully added all documents with real embeddings"),
        Err(e) => {
            println!("❌ Failed to add documents: {}", e);
            return Ok(());
        }
    }

    // Count documents
    match client.count_documents(collection_name).await {
        Ok(count) => println!("✓ Collection now contains {} documents", count),
        Err(e) => println!("⚠ Failed to count documents: {}", e),
    }

    // Demonstrate queries
    println!("\n6. Querying with Real Embeddings");

    let queries = vec![
        "Tell me about programming languages",
        "What is machine learning?",
        "How to deploy applications with containers?",
        "Which language is good for beginners?",
    ];

    for query in queries {
        println!("\n🔍 Query: '{}'", query);
        
        match client.query(collection_name, query, 3, None).await {
            Ok(results) => {
                if results.ids.is_empty() {
                    println!("  No results found");
                } else {
                    println!("  Top {} results:", results.ids.len());
                    for (i, (id, doc, distance)) in results.ids
                        .iter()
                        .zip(results.documents.iter())
                        .zip(results.distances.iter())
                        .enumerate()
                    {
                        println!("    {}. [distance: {:.4}] {}", i + 1, distance, doc);
                    }
                }
            }
            Err(e) => println!("  ❌ Query failed: {}", e),
        }
    }

    // Demonstrate filtered search
    println!("\n7. Filtered Search");
    
    let filter = json!({
        "category": "programming"
    });
    
    println!("🔍 Query with filter (category=programming): 'easy to learn'");
    match client.query(collection_name, "easy to learn", 5, Some(filter)).await {
        Ok(results) => {
            println!("  Found {} filtered results:", results.ids.len());
            for (i, (id, doc, distance)) in results.ids
                .iter()
                .zip(results.documents.iter())
                .zip(results.distances.iter())
                .enumerate()
            {
                println!("    {}. [distance: {:.4}] {}", i + 1, distance, doc);
            }
        }
        Err(e) => println!("  ❌ Filtered query failed: {}", e),
    }

    // Cleanup
    println!("\n8. Cleanup");
    match client.delete_collection(collection_name).await {
        Ok(_) => println!("✓ Deleted test collection"),
        Err(e) => println!("⚠ Failed to delete collection: {}", e),
    }

    println!("\n🎉 Official ChromaDB Demo Completed!");
    println!("\nWhat was demonstrated:");
    println!("  ✓ Official ChromaDB client integration");
    println!("  ✓ Real Gemini embeddings generation");
    println!("  ✓ Collection management (create/delete/list)");
    println!("  ✓ Document operations (add/query/count)");
    println!("  ✓ Similarity search with real vector embeddings");
    println!("  ✓ Metadata filtering");
    println!("  ✓ Production-ready error handling");
    println!("  ✓ Comprehensive logging");

    Ok(())
}