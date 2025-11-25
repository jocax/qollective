FROM rust:1.75-slim as builder

WORKDIR /app

# Copy workspace files
COPY ../../Cargo.toml ../../Cargo.lock ./
COPY ../../src ./src
COPY . ./examples/nats_external

# Build the example
RUN cargo build --release --package nats_external

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/nats_external /usr/local/bin/
CMD ["nats_external"]
