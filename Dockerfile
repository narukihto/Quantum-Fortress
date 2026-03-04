# STAGE 1: Planning - Determine dependencies to optimize Docker layer caching
FROM rust:1.75-slim as planner
WORKDIR /usr/src/quantum_fortress
# Install cargo-chef to speed up subsequent builds
RUN cargo install cargo-chef 
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# STAGE 2: Builder - Compile the application and dependencies
FROM rust:1.75-slim as builder
WORKDIR /usr/src/quantum_fortress

# Install necessary system build tools for PQC and OpenSSL
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Reuse the dependency recipe from Stage 1
RUN cargo install cargo-chef
COPY --from=planner /usr/src/quantum_fortress/recipe.json recipe.json

# Build only the dependencies (this layer is cached)
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source code and build the final release binary
COPY . .
RUN cargo build --release

# STAGE 3: Runtime - Create a minimal, secure execution environment
# Using Google's Distroless (cc-debian12) for maximum security (No shell, no tools)
FROM gcr.io/distroless/cc-debian12

WORKDIR /app

# Copy only the necessary files for runtime from the builder stage
COPY --from=builder /usr/src/quantum_fortress/target/release/quantum_fortress .
COPY --from=builder /usr/src/quantum_fortress/dashboard.html .

# Standard Environment Variables
ENV SERVER_PORT=3000
ENV RUST_LOG=info

# Expose the API and Dashboard port
EXPOSE 3000

# Execute the binary as the entry point
CMD ["./quantum_fortress"]
