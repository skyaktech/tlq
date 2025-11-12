# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2025-11-12
### Added
- Configuration via environment variables (TLQ_PORT, TLQ_MAX_MESSAGE_SIZE, TLQ_LOG_LEVEL)
- IPv6 support - server now binds to both IPv4 and IPv6 interfaces

### Changed
- Migrated Docker registry from Docker Hub to GitHub Container Registry (ghcr.io)
- Updated all dependencies to latest versions

## [0.2.2] - 2025-09-04
### Security
- Updated dependencies to address security vulnerabilities

## [0.2.0] - 2025-08-16

### Added
- Complete REST API with endpoints for all queue operations
- HTTP server with JSON API support
- Comprehensive documentation and usage examples
- Library support - can be used as both CLI tool and Rust library
- Task automation with justfile for build/test/publish workflows
- Integration tests for all API endpoints
- Health check endpoint (`GET /hello`)

### Changed
- Enhanced project structure with better organization
- Improved test organization and cleanup
- Updated dependencies and development workflow

### Features
- **POST /add** - Add message to queue
- **POST /get** - Retrieve messages from queue
- **POST /delete** - Delete processed messages
- **POST /retry** - Return messages to queue for retry
- **POST /purge** - Clear all messages from queue

## [0.1.0] - 2025-02-16

### Added
- Initial release
- Basic message queue functionality
- In-memory storage implementation
- Message service with CRUD operations
- UUID v7 message IDs (time-ordered)
- Simple retry mechanism
- CLI interface