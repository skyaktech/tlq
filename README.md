# tlq

Tiny Little Queue - A minimal message queue server in Rust.

## Install

```bash
cargo install tlq
```

*Note: Ensure `~/.cargo/bin` is in your PATH, or run with `~/.cargo/bin/tlq`*

## Run

```bash
tlq  # Starts server on port 1337
```

## API

All endpoints accept JSON via POST (except `/hello`):

- `GET /hello` - Health check
- `POST /add` - Add message to queue
- `POST /get` - Retrieve messages
- `POST /delete` - Delete processed messages
- `POST /retry` - Return messages to queue
- `POST /purge` - Clear all messages

## Examples

```bash
# Add message
curl -X POST localhost:1337/add \
  -H "Content-Type: application/json" \
  -d '{"body":"Hello World"}'

# Get messages
curl -X POST localhost:1337/get \
  -H "Content-Type: application/json" \
  -d '{"count":5}'

# Delete message
curl -X POST localhost:1337/delete \
  -H "Content-Type: application/json" \
  -d '{"ids":["uuid-here"]}'

# Retry message
curl -X POST localhost:1337/retry \
  -H "Content-Type: application/json" \
  -d '{"ids":["uuid-here"]}'

# Purge all
curl -X POST localhost:1337/purge \
  -H "Content-Type: application/json" \
  -d '{}'
```

## Message Structure

```json
{
  "id": "01234567-89ab-cdef-0123-456789abcdef",
  "body": "Your message content",
  "state": "Ready",
  "retry_count": 0
}
```

## Features

- In-memory storage (ephemeral)
- UUID v7 message IDs (time-ordered)
- 64KB message size limit
- Simple retry mechanism
- Zero configuration

## Use Cases

- Development/testing message queues
- Lightweight job processing
- Local event streaming
- Microservice communication
- Task distribution

## Limitations

- No persistence (memory only)
- No authentication
- Single node only
- No dead letter queue
- No message TTL

## Library Usage

### Direct usage (no HTTP server)

```rust
use tlq::services::MessageService;
use tlq::storage::memory::MemoryStorage;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let store = Arc::new(MemoryStorage::new());
    let service = MessageService::new(store);
    
    // Use the queue directly
    let msg = service.add("Hello World".to_string()).await.unwrap();
    let messages = service.get(1).await.unwrap();
    service.delete(vec![msg.id.to_string()]).await.unwrap();
}
```

### With HTTP API

```rust
use tlq::api::create_api;
use tlq::services::MessageService;
use tlq::storage::memory::MemoryStorage;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let store = Arc::new(MemoryStorage::new());
    let service = MessageService::new(store);
    let app = create_api(service);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:1337")
        .await
        .unwrap();
    
    axum::serve(listener, app).await.unwrap();
}
```

## License

MIT

## Author

Nebojsa Jakovljevic
