# Run all checks before commit
all: fmt clippy build test

# Run pre-commit checks
pre-commit: fmt clippy test
    @echo "✅ All pre-commit checks passed!"

# Format code
fmt:
    cargo fmt --all

# Check formatting without making changes
fmt-check:
    cargo fmt --all -- --check

# Run clippy lints
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Run tests
test:
    cargo test

# Build the project
build:
    cargo build

# Check compilation without producing binaries
check:
    cargo check

# Clean build artifacts
clean:
    cargo clean

# Publish to crates.io (dry-run first)
publish: all
    @echo "Running dry-run first..."
    cargo publish --dry-run
    @echo "Dry-run successful! Publishing to crates.io..."
    cargo publish

# Dry-run publish without actually publishing
publish-dry-run: all
    cargo publish --dry-run

# Build single-architecture Docker image
docker-build:
    docker build -t tlq:latest .

# Build and push multi-architecture Docker image to Docker Hub
docker-publish version="latest": all
    @echo "Building and pushing multi-architecture Docker image..."
    ./build-multiarch.sh {{version}}

# Build multi-architecture Docker image locally (no push)
docker-build-multiarch:
    @echo "Building multi-architecture Docker image locally..."
    ./build-multiarch.sh --local

# Run Docker container locally
docker-run port="1337":
    docker run --rm -p {{port}}:1337 tlq:latest

# Test Docker image health
docker-test:
    @echo "Testing Docker image..."
    docker run --rm --name tlq-test -d -p 8337:1337 tlq:latest
    sleep 3
    curl -s http://localhost:8337/hello || echo "Health check failed"
    docker stop tlq-test


# Show available recipes
help:
    @just --list