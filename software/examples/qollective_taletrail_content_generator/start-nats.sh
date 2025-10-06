#!/usr/bin/env bash
set -e

echo "=== Starting TaleTrail NATS with TLS ==="
echo ""

# Check if certificates exist
if [ ! -f "certs/ca.pem" ]; then
    echo "⚠️  TLS certificates not found. Generating now..."
    cd certs && ./setup-tls.sh && cd ..
    echo ""
fi

# Create nats-data directory if it doesn't exist
mkdir -p nats-data

# Start docker-compose
echo "Starting NATS container with TLS..."
docker-compose up -d

# Wait for healthcheck
echo "Waiting for NATS to be healthy..."
max_attempts=30
attempt=0

while [ $attempt -lt $max_attempts ]; do
    if docker-compose ps | grep -q "healthy"; then
        echo ""
        echo "✅ NATS is healthy and ready!"
        break
    fi
    echo -n "."
    sleep 1
    attempt=$((attempt + 1))
done

if [ $attempt -eq $max_attempts ]; then
    echo ""
    echo "❌ NATS failed to become healthy after 30 seconds"
    echo "Check logs with: docker-compose logs taletrail-nats"
    exit 1
fi

echo ""
echo "=== NATS Connection Information ==="
echo "  TLS Client Port:    nats://localhost:5222"
echo "  Monitoring Dashboard: http://localhost:9222"
echo "  Health Check:       http://localhost:9222/healthz"
echo ""
echo "TLS Configuration:"
echo "  CA Cert:            ./certs/ca.pem"
echo "  Client Cert:        ./certs/client-cert.pem"
echo "  Client Key:         ./certs/client-key.pem"
echo ""
echo "To stop NATS: docker-compose down"
echo "To view logs: docker-compose logs -f taletrail-nats"
