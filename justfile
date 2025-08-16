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

# Show available recipes
help:
    @just --list