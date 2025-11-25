FROM rust:1.75-slim as builder

WORKDIR /app

# Copy workspace files
COPY ../../Cargo.toml ../../Cargo.lock ./
COPY ../../src ./src
COPY . ./examples/nats_internal

# Build the example
RUN cargo build --release --package nats_internal

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/nats_internal /usr/local/bin/
CMD ["nats_internal"]
