# TLQ (Tiny Little Queue)

<p align="center">
  <img src="https://tinylittlequeue.app/logo.svg" alt="TLQ Logo" width="150">
</p>

A minimal message queue that just works.

📖 **[Full Documentation →](USAGE.md)**

## Quick Start

### Install

```bash
# Using Cargo
cargo install tlq

# Using Docker
docker run -p 1337:1337 nebojsa/tlq
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
