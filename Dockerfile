FROM rust:1.88 as builder
WORKDIR /app

# Install protoc (Protocol Buffer compiler)
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

# Install sqlx-cli for running migrations
RUN cargo install sqlx-cli --no-default-features --features postgres

# Copy build files
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./
COPY proto ./proto
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx

# Build with SQLx offline mode
ENV SQLX_OFFLINE=true
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

# Install postgresql-client for wait-for-db and sqlx needs
RUN apt-get update && apt-get install -y postgresql-client && rm -rf /var/lib/apt/lists/*

# Copy from builder
COPY --from=builder /app/target/release/player /app/
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/
COPY --from=builder /app/migrations /app/migrations

# Copy entrypoint script
COPY entrypoint.sh /app/
CMD ["./player"]
