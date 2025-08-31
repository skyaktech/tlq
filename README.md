# TLQ (Tiny Little Queue)

A minimal message queue that just works.

## Install

### Using Cargo

```bash
cargo install tlq
```

*Note: Ensure `~/.cargo/bin` is in your PATH, or run with `~/.cargo/bin/tlq`*

### Using Docker

```bash
docker run -p 1337:1337 nebojsa/tlq
```

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

## Client Libraries

Official client libraries are available for multiple languages:

### Rust
```bash
cargo add tlq-client
```

### Node.js
```bash
npm install tlq-client
```

### Python
```bash
pip install tlq-client
```

### Go
```bash
go get github.com/skyaktech/tlq-client-go
```

## Examples

```bash
# Add message
curl -X POST localhost:1337/add \
  -H "Content-Type: application/json" \
  -d '{"body":"Hello TLQ!"}'

# Get messages
curl -X POST localhost:1337/get \
  -H "Content-Type: application/json" \
  -d '{"count":1}'

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

### In-Memory & Fast
- Zero persistence overhead
- Ideal for development environments
- UUID v7 message IDs (time-ordered)

### Simple API
- Just add and get messages
- No complex configurations
- Simple retry mechanism

### Zero Dependencies
- Lightweight standalone binary
- Minimal system footprint
- 64KB message size limit

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

## License

MIT

## Author

Nebojsa Jakovljevic
