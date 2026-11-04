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

# Stage 2: Minimal runtime container using distroless
FROM gcr.io/distroless/cc-debian12:nonroot

WORKDIR /app

# Copy compiled binary from builder
COPY --from=builder /app/target/release/demo /app/demo

# Run as non-root user provided by distroless (UID 65532)
USER 65532:65532

EXPOSE 8080

CMD ["/app/demo"]
