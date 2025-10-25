# ChromaDB with Rust - Production Ready Implementation

A comprehensive Rust implementation for ChromaDB with Google Gemini embeddings, featuring production-ready patterns including connection pooling, retry logic, and proper error handling.

## Features

- 🚀 **Production-ready ChromaDB client** with connection pooling and retry logic
- 🧠 **Google Gemini embeddings integration** with batch processing
- 🐳 **Docker-based ChromaDB setup** with persistent storage
- 🔧 **Comprehensive error handling** with custom error types
- 📊 **Structured logging** with tracing
- ⚙️ **Configurable timeouts and retry policies**
- 🔄 **Automatic retries** for transient failures

## Quick Start

### 1. Clone and Setup

```bash
git clone <repository-url>
cd chromadb-demo
cp .env.example .env
# Edit .env with your Google API key
```

### 2. Start ChromaDB

```bash
docker-compose up -d
# Verify it's running
curl http://localhost:8000/api/v2/heartbeat
```

### 3. Run the Application

```bash
cargo run --bin chroma_client
```

## Configuration

### Environment Variables

Create a `.env` file with the following variables:

```env
# ChromaDB Configuration
CHROMA_HOST=http://localhost:8000
COLLECTION_NAME=documents

# Google Gemini API Configuration
GOOGLE_API_KEY=your_google_api_key_here

# Application Configuration
RUST_LOG=info
MAX_RETRIES=3
RETRY_DELAY_MS=1000
CONNECTION_TIMEOUT_MS=30000
REQUEST_TIMEOUT_MS=60000
```

## Architecture

### Core Components

- **ChromaClient**: Production-ready client with connection pooling and retry logic
- **EmbeddingClient**: Google Gemini integration with batch processing
- **Error handling**: Comprehensive error types with proper propagation
- **Models**: Type-safe data structures for all API interactions

### Best Practices Implemented

1. **Connection Management**
   - HTTP connection pooling with configurable limits
   - Keep-alive connections for better performance
   - Proper timeout handling

2. **Retry Logic**
   - Exponential backoff for failed requests
   - Configurable retry attempts and delays
   - Smart retry decisions based on error types

3. **Error Handling**
   - Custom error types with `thiserror`
   - Proper error propagation and logging
   - Graceful degradation

4. **Embeddings Optimization**
   - Batch processing for multiple texts
   - Configurable batch sizes
   - Proper dimension validation

## Usage Examples

### Basic Document Operations

```rust
use chromadb_demo::{ChromaClient, EmbeddingClient, Document};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize clients
    let chroma = ChromaClient::new("http://localhost:8000".to_string());
    let embeddings = EmbeddingClient::new("your_api_key".to_string());

    // Health check
    if !chroma.health_check().await? {
        panic!("ChromaDB is not accessible");
    }

    // Create collection
    chroma.create_collection("my_docs").await?;

    // Add documents
    let docs = vec![
        Document {
            id: "doc1".to_string(),
            content: "Your document content".to_string(),
            metadata: HashMap::new(),
        }
    ];

    let embeddings_vec = embeddings.embed_texts(&["Your document content"]).await?;
    chroma.add_documents("my_docs", docs, embeddings_vec).await?;

    // Query similar documents
    let query_embedding = embeddings.embed_text("search query").await?;
    let results = chroma.query("my_docs", vec![query_embedding], 5).await?;

    Ok(())
}
```

## Docker Configuration

The included `docker-compose.yml` provides:

- ChromaDB with persistent storage
- Health checks
- Optimized configuration for development and production

```yaml
version: '3.8'
services:
  chromadb:
    image: chromadb/chroma:latest
    ports:
      - "8000:8000"
    environment:
      - CHROMA_DB_IMPL=duckdb
      - PERSIST_DIRECTORY=/chroma/data
      - ANONYMIZED_TELEMETRY=false
    volumes:
      - chroma_data:/chroma/data
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/api/v2/heartbeat"]
      interval: 5s
      timeout: 10s
      retries: 5
```

## Testing

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test health_check
```

## Production Deployment

### Performance Considerations

1. **Connection Pooling**: The client maintains a pool of HTTP connections
2. **Batch Processing**: Embeddings are processed in configurable batches
3. **Retry Logic**: Automatic retries for transient failures
4. **Timeout Configuration**: Proper timeouts for all operations

### Monitoring

The application uses structured logging with tracing. Set `RUST_LOG=debug` for detailed logs.

### Security

- API keys are loaded from environment variables
- HTTPS support for production deployments
- Proper error message sanitization

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Connection refused | Ensure ChromaDB is running: `docker-compose up -d` |
| Invalid API key | Check your Google API key in `.env` |
| Timeout errors | Increase timeout values in configuration |
| Embedding dimension mismatch | Ensure consistent use of 768-dimensional vectors |

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

[Your License Here]