# Multi-stage build for ARM64 (HA Green architecture)
FROM rust:1.75-bookworm AS builder

# Install cross-compilation dependencies for ARM64
RUN apt-get update && apt-get install -y \
    gcc-aarch64-linux-gnu \
    libc6-dev-arm64-cross \
    && rm -rf /var/lib/apt/lists/*

# Set up Rust target for ARM64
RUN rustup target add aarch64-unknown-linux-gnu

# Create a new empty shell project
WORKDIR /usr/src/app

# Copy Cargo files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --target aarch64-unknown-linux-gnu && rm -rf src

# Copy actual source code
COPY src ./src

# Set environment variables for cross-compilation
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
ENV CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
ENV CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++

# Build the actual application
RUN touch src/main.rs && cargo build --release --target aarch64-unknown-linux-gnu

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/aarch64-unknown-linux-gnu/release/custom-ha-service /usr/local/bin/custom-ha-service

# Copy the run script
COPY run.sh /run.sh
RUN chmod +x /run.sh

WORKDIR /

CMD ["/run.sh"]
