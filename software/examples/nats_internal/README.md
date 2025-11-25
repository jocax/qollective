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

```
[INFO] Starting NATS server...
[INFO] Server listening on subject: example.echo
[INFO] Sending envelope request...
[INFO] Received response: Echo: Hello from internal client!
```
