# Multi-stage build for minimal image size
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.toml
COPY crates/ crates/

# Build dependencies (cached layer)
RUN cargo build --release --bin sandwich-server

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 sandwich && \
    mkdir -p /app && \
    chown -R sandwich:sandwich /app

# Copy binary from builder
COPY --from=builder /app/target/release/sandwich-server /usr/local/bin/sandwich-server

# Switch to non-root user
USER sandwich

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run the server
CMD ["sandwich-server"]
