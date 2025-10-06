#!/usr/bin/env bash
set -e

echo "=== TaleTrail NATS TLS Certificate Setup ==="
echo ""

# Clean up existing certificates
rm -f *.pem *.srl

# Generate CA certificate
echo "1. Generating CA certificate..."
openssl req -x509 -nodes -newkey rsa:4096 -keyout ca-key.pem -out ca.pem \
  -days 365 -subj "/CN=TaleTrail-CA"

# Generate server certificate
echo "2. Generating server certificate..."
openssl req -nodes -newkey rsa:4096 -keyout server-key.pem \
  -out server-req.pem -subj "/CN=taletrail-nats"

# Create server certificate extensions file
cat > server-ext.cnf <<EOF
subjectAltName = DNS:localhost,DNS:taletrail-nats,IP:127.0.0.1
EOF

# Sign server certificate
openssl x509 -req -in server-req.pem -CA ca.pem -CAkey ca-key.pem \
  -CAcreateserial -out server-cert.pem -days 365 -extfile server-ext.cnf

# Generate client certificate
echo "3. Generating client certificate..."
openssl req -nodes -newkey rsa:4096 -keyout client-key.pem \
  -out client-req.pem -subj "/CN=taletrail-client"

# Sign client certificate
openssl x509 -req -in client-req.pem -CA ca.pem -CAkey ca-key.pem \
  -CAcreateserial -out client-cert.pem -days 365

# Generate gateway HTTPS certificate
echo "4. Generating gateway HTTPS certificate..."
openssl req -nodes -newkey rsa:4096 -keyout gateway-key.pem \
  -out gateway-req.pem -subj "/CN=taletrail-gateway"

# Create gateway certificate extensions file
cat > gateway-ext.cnf <<EOF
subjectAltName = DNS:localhost,DNS:taletrail-gateway,IP:127.0.0.1,IP:0.0.0.0
extendedKeyUsage = serverAuth
EOF

# Sign gateway certificate
openssl x509 -req -in gateway-req.pem -CA ca.pem -CAkey ca-key.pem \
  -CAcreateserial -out gateway-cert.pem -days 365 -extfile gateway-ext.cnf

# Set proper permissions
chmod 644 *.pem

# Clean up intermediate files
rm -f server-req.pem client-req.pem gateway-req.pem server-ext.cnf gateway-ext.cnf

echo ""
echo "✅ TLS certificates generated successfully!"
echo ""
echo "Generated files:"
echo "  - ca.pem (CA certificate)"
echo "  - ca-key.pem (CA private key)"
echo "  - server-cert.pem (NATS server certificate)"
echo "  - server-key.pem (NATS server private key)"
echo "  - client-cert.pem (MCP services client certificate)"
echo "  - client-key.pem (MCP services client private key)"
echo "  - gateway-cert.pem (Gateway HTTPS certificate)"
echo "  - gateway-key.pem (Gateway HTTPS private key)"
echo ""
echo "Per-Service TLS Architecture:"
echo "  🔒 NATS Server: server-cert.pem (CN=taletrail-nats)"
echo "  🔒 Gateway HTTPS: gateway-cert.pem (CN=taletrail-gateway)"
echo "  🔒 MCP Services: client-cert.pem (CN=taletrail-client)"
echo ""
echo "Ready to start services with TLS support!"
