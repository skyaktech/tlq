# TLQ User Guide

## TL;DR

TLQ is an in-memory message queue where you add messages via `/add` (returns a UUID), retrieve them via `/get` (which locks them in "Processing" state, making them invisible to other consumers), then either `/delete` them after successful processing or `/retry` them on failure (which returns them to "Ready" state with an incremented retry count) - all messages are lost on server restart since there's no persistence.

## Overview

TLQ (Tiny Little Queue) is an in-memory message queue that provides simple, reliable message processing with automatic state management. Messages are stored in memory only - there is no persistence across server restarts.

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
- `state` - Current message state
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

Permanently removes successfully processed messages from the queue. Use this after successful message processing.

### Retrying Messages

**POST /retry**
```json
{"ids": ["uuid1", "uuid2"]}
```

Returns messages to the queue when processing fails:
- Changes state back to **Ready**
- Increments `retry_count`
- Makes message available for retrieval again

### Purging Queue

**POST /purge**
```json
{}
```

Removes all messages from the queue, including those being processed.

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

## Example Workflow

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