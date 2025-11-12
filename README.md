# TLQ (Tiny Little Queue)

<p align="center">
  <img src="https://tinylittlequeue.app/logo.svg" alt="TLQ Logo" width="150">
</p>

A minimal message queue that just works.

📖 **[Full Documentation →](https://github.com/skyaktech/tlq/blob/main/USAGE.md)**

## Quick Start

### Install

```bash
# Using Cargo
cargo install tlq

# Using Docker
docker run -p 1337:1337 ghcr.io/skyaktech/tlq

# Docker with custom configuration
docker run -e TLQ_PORT=8080 -p 8080:8080 ghcr.io/skyaktech/tlq
docker run -e TLQ_MAX_MESSAGE_SIZE=1048576 -e TLQ_LOG_LEVEL=debug -p 1337:1337 ghcr.io/skyaktech/tlq
```

### Use

```bash
# Add a message
curl -X POST localhost:1337/add \
  -H "Content-Type: application/json" \
  -d '{"body":"Hello TLQ!"}'

# Get a message (auto-locks it)
curl -X POST localhost:1337/get \
  -H "Content-Type: application/json" \
  -d '{"count":1}'

# Delete after success
curl -X POST localhost:1337/delete \
  -H "Content-Type: application/json" \
  -d '{"ids":["<message-id>"]}'

# Or retry after failure
curl -X POST localhost:1337/retry \
  -H "Content-Type: application/json" \
  -d '{"ids":["<message-id>"]}'
```

## Features

- **In-memory** - Zero persistence overhead
- **Simple API** - Just add, get, delete, retry
- **Auto-locking** - Messages lock on retrieval
- **Client libraries** - [Rust](https://crates.io/crates/tlq-client), [Node.js](https://www.npmjs.com/package/tlq-client), [Python](https://pypi.org/project/tlq-client/), [Go](https://pkg.go.dev/github.com/skyaktech/tlq-client-go)

## Configuration

You can configure TLQ via environment variables (all optional; defaults shown):
- TLQ_PORT: TCP port to listen on. Default: 1337
- TLQ_MAX_MESSAGE_SIZE: Maximum message body size in bytes. Default: 65536
- TLQ_LOG_LEVEL: Log verbosity (trace, debug, info, warn, error). Default: info

Examples:

```bash
TLQ_PORT=8080 tlq
TLQ_MAX_MESSAGE_SIZE=1048576 TLQ_LOG_LEVEL=debug tlq
```

## Why TLQ?

Perfect for:
- Development & testing
- Lightweight job processing
- Microservice communication
- Any scenario where persistence isn't critical

## License

MIT

## Author

Nebojsa Jakovljevic
