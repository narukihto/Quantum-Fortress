# Stage 1: Building the binary
FROM rust:1.75-slim as builder

# Install system dependencies for PQC libraries
RUN apt-get update && apt-get install -y cmake clang llvm libclang-dev

WORKDIR /usr/src/quantum_fortress
COPY . .

# Build for release
RUN cargo build --release

# Stage 2: Final Runtime Image
FROM debian:bookworm-slim

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder
COPY --from=builder /usr/src/quantum_fortress/target/release/quantum_fortress .
# Copy the .env file for configuration
COPY .env .

# Expose the port from your config
EXPOSE 3000

# Run the gateway
CMD ["./quantum_fortress"]
