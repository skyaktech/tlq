# TLQ User Guide

## TL;DR

TLQ is an in-memory message queue where you add messages via `/add` (returns a UUID), retrieve them via `/get` (which locks them in "Processing" state, making them invisible to other consumers), then either `/delete` them after successful processing or `/retry` them on failure (which returns them to "Ready" state with an incremented retry count) - all messages are lost on server restart since there's no persistence.

## Overview

TLQ (Tiny Little Queue) is an in-memory message queue that provides simple, reliable message processing with automatic state management. Messages are stored in memory only - there is no persistence across server restarts.

## Installation

### Using Cargo

Install TLQ directly from crates.io:

```bash
cargo install tlq
```

After installation, ensure `~/.cargo/bin` is in your PATH. You can verify the installation with:

```bash
tlq
```

If the command is not found, you can run it directly with:
```bash
~/.cargo/bin/tlq
```

### Using Docker

Run TLQ using the official Docker image:

```bash
# Default configuration
docker run -p 1337:1337 ghcr.io/skyaktech/tlq

# Custom port (note: port mapping must match TLQ_PORT)
docker run -e TLQ_PORT=8080 -p 8080:8080 ghcr.io/skyaktech/tlq

# Custom message size limit (using k suffix)
docker run -e TLQ_MAX_MESSAGE_SIZE=128k -p 1337:1337 ghcr.io/skyaktech/tlq

# Debug logging
docker run -e TLQ_LOG_LEVEL=debug -p 1337:1337 ghcr.io/skyaktech/tlq

# Multiple options combined
docker run -e TLQ_PORT=9000 -e TLQ_LOG_LEVEL=debug -p 9000:9000 ghcr.io/skyaktech/tlq
```

### Building from Source

```bash
git clone https://github.com/nebjak/tlq.git
cd tlq
cargo build --release
./target/release/tlq
```

## Running the Server

Start TLQ with default settings:

```bash
tlq
```

The server will start on `http://localhost:1337`. You can verify it's running:

```bash
curl http://localhost:1337/hello
# Returns: "Hello World"
```

## Configuration

TLQ can be configured via environment variables. All are optional; defaults are shown.

- TLQ_PORT: TCP port to listen on. Default: 1337
- TLQ_MAX_MESSAGE_SIZE: Maximum message body size in bytes. Supports K/k suffix (e.g., 128K = 131072 bytes). Default: 65536
- TLQ_LOG_LEVEL: Log verbosity (trace, debug, info, warn, error). Default: info

Examples:

```bash
# Change port
TLQ_PORT=8080 tlq

# Increase message size to 1MB and use debug logs
TLQ_MAX_MESSAGE_SIZE=128k TLQ_LOG_LEVEL=debug tlq

# Alternative: specify size in bytes
TLQ_MAX_MESSAGE_SIZE=32768 TLQ_LOG_LEVEL=debug tlq
```

Note: The official Dockerfile exposes and health-checks port 1337 by default; if you change TLQ_PORT inside the container, you may want to adjust your run command and health checks accordingly.

## Client Libraries

Official clients are available for:
- [Rust](https://crates.io/crates/tlq-client)
- [Node.js](https://www.npmjs.com/package/tlq-client)
- [Python](https://pypi.org/project/tlq-client/)
- [Go](https://pkg.go.dev/github.com/skyaktech/tlq-client-go)

Use whatever language your project needs - all clients provide the same functionality with idiomatic APIs for each language.

## Core Concepts

### Message Lifecycle

Messages in TLQ move through distinct states:

- **Ready** - Available for consumers to retrieve
- **Processing** - Locked by a consumer, invisible to others

### Message Structure

Every message contains:
- `id` - UUID v7 (time-ordered unique identifier)
- `body` - Message content (max 64KB)
- `state` - Current message state ("Ready", "Processing")
- `retry_count` - Number of retry attempts

## Operations

### Adding Messages

**POST /add**
```json
{"body": "Your message content"}
```

Returns the complete message object with generated UUID:
```json
{
  "id": "01234567-89ab-cdef-0123-456789abcdef",
  "body": "Your message content",
  "state": "Ready",
  "retry_count": 0
}
```

Messages are immediately available for consumption after being added.

### Retrieving Messages

**POST /get**
```json
{"count": 5}
```
Optional: `count` defaults to 1 if not specified.

Returns an array of messages. Retrieved messages:
- Automatically transition to **Processing** state
- Become invisible to other consumers
- Must be explicitly deleted or retried

```json
[
  {
    "id": "01234567-89ab-cdef-0123-456789abcdef",
    "body": "Message content",
    "state": "Processing",
    "retry_count": 0
  }
]
```

### Deleting Messages

**POST /delete**
```json
{"ids": ["uuid1", "uuid2"]}
```

Permanently removes messages from the queue. Use this after successful message processing. Returns "Success" on completion.

### Retrying Messages

**POST /retry**
```json
{"ids": ["uuid1", "uuid2"]}
```

Returns messages to the queue when processing fails:
- Changes state back to **Ready**
- Increments `retry_count`
- Makes message available for retrieval again
- Returns "Success" on completion

### Purging Queue

**POST /purge**
```json
{}
```

Removes all messages from the queue, including those being processed.

**⚠️ Warning:** This operation:
- Immediately deletes ALL messages in the queue
- Includes messages in "Processing" state
- Cannot be undone
- Returns "Success" on completion

Use cases:
- Clearing test data during development
- Emergency reset when queue is corrupted
- Starting fresh after configuration changes

### Health Check

**GET /hello**

Returns `"Hello World"` to verify server availability.

## Message Processing Pattern

1. **Consumer retrieves** message via `/get`
2. **Message locks** automatically (Processing state)
3. **Consumer processes** the message
4. **On success**: Delete message via `/delete`
5. **On failure**: Return to queue via `/retry`

## Important Notes

- **No persistence** - All messages lost on server restart
- **No TTL** - Messages remain until explicitly deleted
- **No dead letter queue** - Retry count increments but messages retry indefinitely
- **Single node only** - No clustering or replication
- **64KB limit** - Maximum message body size

## Examples

### Basic Workflow

```bash
# Add message
curl -X POST localhost:1337/add \
  -H "Content-Type: application/json" \
  -d '{"body":"Process this task"}'

# Get and process
curl -X POST localhost:1337/get \
  -H "Content-Type: application/json" \
  -d '{"count":1}'
# Returns message with ID

# Success - delete it
curl -X POST localhost:1337/delete \
  -H "Content-Type: application/json" \
  -d '{"ids":["returned-uuid-here"]}'

# OR Failure - retry it
curl -X POST localhost:1337/retry \
  -H "Content-Type: application/json" \
  -d '{"ids":["returned-uuid-here"]}'
```

### Batch Processing

```bash
# Add multiple messages
curl -X POST localhost:1337/add \
  -H "Content-Type: application/json" \
  -d '{"body":"Task 1"}'

curl -X POST localhost:1337/add \
  -H "Content-Type: application/json" \
  -d '{"body":"Task 2"}'

# Get multiple messages at once
curl -X POST localhost:1337/get \
  -H "Content-Type: application/json" \
  -d '{"count":10}'

# Delete multiple messages
curl -X POST localhost:1337/delete \
  -H "Content-Type: application/json" \
  -d '{"ids":["uuid1", "uuid2", "uuid3"]}'
```

### Using Client Libraries

#### Node.js
```javascript
const { TLQClient, AddMessageCommand, GetMessagesCommand, 
        DeleteMessagesCommand, RetryMessagesCommand, 
        PurgeQueueCommand } = require('tlq-client');

const client = new TLQClient({
  host: 'localhost',
  port: 1337
});

// Add message
const message = await client.send(new AddMessageCommand({
  body: JSON.stringify({ task: 'Process this' })
}));
console.log('Added message:', message.id);

// Get and process messages
const result = await client.send(new GetMessagesCommand({ count: 5 }));
for (const msg of result.messages) {
  try {
    // Process message
    await processTask(JSON.parse(msg.body));
    await client.send(new DeleteMessagesCommand({ ids: [msg.id] }));
  } catch (error) {
    await client.send(new RetryMessagesCommand({ ids: [msg.id] }));
  }
}

// Purge queue if needed
await client.send(new PurgeQueueCommand());
```

#### Python
```python
from tlq_client import TLQClient

# Using context manager for automatic cleanup
with TLQClient(host="localhost", port=1337) as client:
    # Add message
    message_id = client.add_message("Process this task")
    print(f'Added message: {message_id}')
    
    # Get and process messages
    messages = client.get_messages(count=5)
    for msg in messages:
        try:
            # Process message
            process_task(msg.body)
            client.delete_messages(msg.id)
        except Exception:
            client.retry_messages(msg.id)
    
    # Purge queue if needed (careful!)
    # client.purge_queue()
```

#### Rust
```rust
use tlq_client::TlqClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = TlqClient::builder()
        .host("localhost")
        .port(1337)
        .build();
    
    // Add message
    let message = client.add_message("Process this task").await?;
    println!("Added message: {}", message.id);
    
    // Get and process messages
    let messages = client.get_messages(5).await?;
    for msg in messages {
        match process_task(&msg.body).await {
            Ok(_) => client.delete_message(msg.id).await?,
            Err(_) => client.retry_message(msg.id).await?,
        }
    }
    
    // Purge queue if needed
    // client.purge_queue().await?;
    
    Ok(())
}
```

#### Go
```go
package main

import (
    "context"
    "fmt"
    "time"
    "github.com/skyaktech/tlq-client-go"
)

func main() {
    // Create configured client
    client := tlq.NewClient(
        tlq.WithHost("localhost"),
        tlq.WithPort(1337),
        tlq.WithTimeout(30 * time.Second),
    )
    
    ctx := context.Background()
    
    // Add message
    message, err := client.AddMessage(ctx, "Process this task")
    if err != nil {
        panic(err)
    }
    fmt.Printf("Added message: %s\n", message.ID)
    
    // Get and process messages
    messages, err := client.GetMessages(ctx, 5)
    if err != nil {
        panic(err)
    }
    
    for _, msg := range messages {
        if err := processTask(msg.Body); err != nil {
            client.RetryMessages(ctx, []string{msg.ID})
        } else {
            client.DeleteMessage(ctx, msg.ID)
        }
    }
    
    // Purge queue if needed
    // err = client.PurgeQueue(ctx)
}
```

### Emergency Operations

```bash
# Check server health
curl http://localhost:1337/hello
# Returns: "Hello World"

# Purge all messages (use with caution!)
curl -X POST localhost:1337/purge \
  -H "Content-Type: application/json" \
  -d '{}'
# Returns: "Success"
```

**When to use purge:**
- Development/testing: Clear test data between runs
- Emergency: Queue contains corrupted or invalid messages
- Reset: Starting fresh after major changes

**⚠️ Never use purge in production unless absolutely necessary** - there's no way to recover purged messages.