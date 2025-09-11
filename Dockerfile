# Build stage
FROM rust:1.89-slim AS builder

WORKDIR /app

# Install dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy src/main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --bin tlq
RUN rm -rf src

# Copy actual source code
COPY src ./src

# Build the actual binary
RUN touch src/main.rs && cargo build --release --bin tlq

# Runtime stage
FROM debian:bookworm-slim

# Install curl for healthcheck
RUN apt-get update && apt-get install -y \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false tlq

# Copy binary from builder stage
COPY --from=builder /app/target/release/tlq /usr/local/bin/tlq

# Change ownership and make executable
RUN chown tlq:tlq /usr/local/bin/tlq && chmod +x /usr/local/bin/tlq

# Switch to non-root user
USER tlq

# Set default configuration via environment variables
ENV TLQ_PORT=1337
ENV TLQ_MAX_MESSAGE_SIZE=65536
ENV TLQ_LOG_LEVEL=info

# Expose port (note: if TLQ_PORT is changed, map ports accordingly)
EXPOSE 1337

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/bin/sh", "-c", "curl -f http://localhost:${TLQ_PORT:-1337}/hello || exit 1"]

# Run the binary
CMD ["tlq"]