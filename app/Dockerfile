# Stage 1: Build binary with cached dependency layer
FROM rust:1.99.1-slim-bookworm AS builder

WORKDIR /app

# Copy dependency specifications
COPY Cargo.toml Cargo.lock* ./

# Build dependencies with placeholder main.rs for layer caching
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src target/release/deps/demo* target/release/demo*

# Copy application source
COPY src ./src

# Build production binary and strip symbols
RUN cargo build --release && \
    strip target/release/demo

# Stage 2: Minimal runtime container
FROM debian:12.9-slim

# Install ca-certificates and clean apt caches
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Create non-root system user
RUN groupadd -g 10001 -r appuser && \
    useradd -u 10001 -m -r -g appuser appuser

# Copy compiled binary from builder
COPY --from=builder --chown=appuser:appuser /app/target/release/demo /app/demo

USER 10001

EXPOSE 8080

CMD ["/app/demo"]
