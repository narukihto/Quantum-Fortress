# Stage 1: Build Stage
FROM rust:1.75-slim as builder

# Install system dependencies for PQC libraries
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/quantum_fortress
COPY . .

# Build the release binary
RUN cargo build --release

# Stage 2: Final Runtime Stage
FROM debian:bookworm-slim

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary and the dashboard from the builder stage
COPY --from=builder /usr/src/quantum_fortress/target/release/quantum_fortress .
COPY --from=builder /usr/src/quantum_fortress/dashboard.html .

# Expose the aligned port 3000
EXPOSE 3000

# Start the Quantum Fortress Sentinel
CMD ["./quantum_fortress"]
