# NATS External Example

Demonstrates the new shared connection pattern using `NatsServer::from_client()` and `server.client()` for multi-layer architectures.

## What This Shows

- `NatsServer::from_client()` - Create server from existing connection
- `server.client()` - Access underlying client for direct NATS operations
- **Single connection** serving both envelope-based MCP and raw NATS pub/sub
- 50% connection reduction compared to separate connections

## Use Case: TaleTrails Pattern

```
┌─────────────────────────────────────┐
│         Single NATS Connection       │
├─────────────────────────────────────┤
│  Layer 1: MCP Request/Response      │
│  - Envelope encoding/decoding       │
│  - Queue group load balancing       │
├─────────────────────────────────────┤
│  Layer 2: Application Messaging     │
│  - Service logs                     │
│  - Status events                    │
│  - Metrics                          │
└─────────────────────────────────────┘
```

## Running

```bash
# Start NATS server in Docker (runs on localhost:10222)
docker-compose up -d

# Run the example on your host machine
cargo run

# Stop NATS when done
docker-compose down
```

## Configuration

The example connects to `nats://localhost:10222` by default. You can override with:
```bash
NATS_URL=nats://localhost:4222 cargo run
```

## Expected Output

```
[INFO] Creating shared NATS connection...
[INFO] Creating NatsServer from existing client...
[INFO] Server listening on subject: example.echo
[INFO] Publishing app log via server.client()...
[INFO] Sending envelope request via same connection...
[INFO] Received response: Echo: Hello from shared connection!
[INFO] Connection count: 1 (vs 2 with separate connections)
```
