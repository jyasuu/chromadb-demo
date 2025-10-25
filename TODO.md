# ChromaDB with Rust: Complete Guide


### Start ChromaDB
```bash
docker-compose up -d
# Verify it's running
curl http://localhost:8000/api/v2/heartbeat
```

---

---

## 3. Environment Configuration

### .env
```
CHROMA_HOST=http://localhost:8000
GOOGLE_API_KEY=your_google_api_key_here
COLLECTION_NAME=documents
```

---


---

## 5. Best Practices

### Connection Management
- Use connection pooling with `reqwest::Client`
- Reuse HTTP clients across requests
- Implement retry logic for transient failures

### Error Handling
- Create custom error types with `thiserror`
- Use `Result<T>` type alias consistently
- Log errors for debugging

### Embeddings
- Batch embedding requests when possible
- Cache embeddings to avoid redundant API calls
- Use appropriate vector dimensions (Gemini: 768-dim)

### ChromaDB Optimization
- Use metadata filtering to reduce query scope
- Set appropriate `n_results` values
- Use collection names strategically
- Consider persistence strategy

### Production Considerations
```rust
// Add retry logic
pub async fn with_retries<F, T>(
    mut f: F,
    max_retries: u32,
) -> Result<T>
where
    F: FnMut() -> futures::future::BoxFuture<'static, Result<T>>,
{
    let mut retries = 0;
    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if retries < max_retries => {
                retries += 1;
                tokio::time::sleep(
                    tokio::time::Duration::from_secs(2_u64.pow(retries))
                ).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

---

## 6. Running the Application

```bash
# Set up environment
cp .env.example .env
# Edit .env with your Google API key

# Start ChromaDB
docker-compose up -d

# Run the client
cargo run --bin chroma_client

# Check logs
docker-compose logs -f chromadb
```

---

## 7. Testing

```bash
# Run tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test health_check
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Connection refused | Ensure ChromaDB is running: `docker-compose up -d` |
| Invalid embeddings | Check embedding dimension matches ChromaDB expectation |
| Collection exists error | Delete collection first or handle gracefully |
| Slow queries | Add metadata filters or increase n_results selectively |