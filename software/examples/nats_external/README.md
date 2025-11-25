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

The example showcases:
- **Shared Connection**: One NATS connection for both MCP envelopes and raw pub/sub
- **Rich Metadata**: request_id and timestamp for distributed tracing
- **Pretty-printed Envelopes**: Full visibility into the envelope structure

```
[INFO] === NATS External (Shared Connection) Example ===
[INFO] Creating shared NATS connection...
[INFO] Creating NatsServer from existing client...
[INFO] Server listening on subject: example.echo
[INFO] [App Layer] Publishing log via server.client()...
[INFO] [App Layer] Received log: Service started successfully
[INFO] [MCP Layer] Sending envelope request via same connection...
[INFO] Request Envelope (pretty):
{
  "meta": {
    "timestamp": "2025-11-25T10:30:45.123Z",
    "request_id": "650e8400-e29b-41d4-a716-446655440000"
  },
  "payload": {
    "message": "Hello from shared connection!",
    "priority": "high"
  }
}
[INFO] [MCP Layer] Handler received envelope
[INFO] [MCP Layer] Response Envelope (pretty):
{
  "meta": {
    "timestamp": "2025-11-25T10:30:45.123Z",
    "request_id": "650e8400-e29b-41d4-a716-446655440000"
  },
  "payload": {
    "echo": "Echo: Hello from shared connection!",
    "server_id": "nats-external-server"
  }
}
[INFO] === Summary ===
[INFO] Connection count: 1 (vs 2 with separate connections)
```
