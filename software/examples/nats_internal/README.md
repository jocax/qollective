# NATS Internal Example

Demonstrates the standard Qollective NATS pattern using internal client/server with envelope-based requests.

## What This Shows

- `NatsServer::new()` - Create server with config
- `NatsClient::new()` - Create client with config
- Envelope-based request/response over NATS
- Handler registration for subjects

## Running

```bash
# Start NATS and run example
docker-compose up --build

# Or run locally (requires NATS on localhost:4222)
NATS_URL=nats://localhost:4222 cargo run
```

## Expected Output

```
[INFO] Starting NATS server...
[INFO] Server listening on subject: example.echo
[INFO] Sending envelope request...
[INFO] Received response: Echo: Hello from internal client!
```
