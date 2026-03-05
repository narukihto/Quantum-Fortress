# STAGE 1: Planning - Generate dependency recipe
FROM rust:1.75-slim as planner
WORKDIR /usr/src/quantum_fortress
RUN cargo install cargo-chef 
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# STAGE 2: Builder - Compile with PQC system dependencies
FROM rust:1.75-slim as builder
WORKDIR /usr/src/quantum_fortress

# Install critical build tools for Dilithium (C-based PQC) and OpenSSL
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    cmake \
    clang \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-chef
COPY --from=planner /usr/src/quantum_fortress/recipe.json recipe.json

# Build only dependencies to maximize Docker layer caching
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source code and perform the final production build
COPY . .
RUN cargo build --release

# STAGE 3: Runtime - Highly secure minimal environment
FROM gcr.io/distroless/cc-debian12

WORKDIR /app

# Copy the compiled binary
COPY --from=builder /usr/src/quantum_fortress/target/release/quantum_fortress .

# Copy essential runtime assets (Dashboard & Blockchain ABI)
COPY --from=builder /usr/src/quantum_fortress/dashboard.html .
COPY --from=builder /usr/src/quantum_fortress/IntegrityLedger.json .

# Default Environment Configuration
ENV SERVER_PORT=3000
ENV RUST_LOG=info
ENV MIN_ENTROPY=3.8

# Expose Service Port
EXPOSE 3000

# Run the fortress gateway
CMD ["./quantum_fortress"]
