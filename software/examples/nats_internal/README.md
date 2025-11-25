# NATS Internal Example

Demonstrates the standard Qollective NATS pattern using internal client/server with envelope-based requests.

## What This Shows

- `NatsServer::new()` - Create server with config
- `NatsClient::new()` - Create client with config
- Envelope-based request/response over NATS
- Handler registration for subjects

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

The example demonstrates the envelope structure with:
- **Metadata**: request_id (UUID) and timestamp for request tracking
- **Structured Payloads**: Multi-field JSON objects (not just strings)
- **Pretty-printed JSON**: Clear visibility of the envelope structure

```
[INFO] Starting NATS server...
[INFO] Server listening on subject: example.echo
[INFO] === Sending Envelope Request ===
[INFO] Request Envelope (pretty):
{
  "meta": {
    "timestamp": "2025-11-25T10:30:45.123Z",
    "request_id": "550e8400-e29b-41d4-a716-446655440000"
  },
  "payload": {
    "message": "Hello from internal client!",
    "sender_id": "example-client-001"
  }
}
[INFO] === Server Handler Received Envelope ===
[INFO] === Received Response Envelope ===
[INFO] Response Envelope (pretty):
{
  "meta": {
    "timestamp": "2025-11-25T10:30:45.123Z",
    "request_id": "550e8400-e29b-41d4-a716-446655440000"
  },
  "payload": {
    "echo": "Echo: Hello from internal client!",
    "processed_at": "2025-11-25T10:30:45.456Z"
  }
}
```
