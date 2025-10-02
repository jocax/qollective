# TLS Setup and Configuration Guide

> Enterprise A2A NATS Example with TLS Security
> Last Updated: 2025-07-29

## Overview

This guide provides comprehensive instructions for setting up TLS (Transport Layer Security) in the Qollective Enterprise A2A NATS example. The implementation uses mutual TLS (mTLS) authentication to secure all agent-to-agent communication channels.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [TLS Certificate Setup](#tls-certificate-setup)
3. [Configuration](#configuration)
4. [Enterprise Example Setup](#enterprise-example-setup)
5. [Running with TLS](#running-with-tls)
6. [Verification](#verification)
7. [Certificate Management](#certificate-management)
8. [Security Best Practices](#security-best-practices)

## Prerequisites

### System Requirements

- **Rust**: Version 1.75.0 or later
- **NATS Server**: Version 2.10+ with TLS support
- **OpenSSL/rustls**: For certificate operations
- **Docker** (optional): For containerized NATS deployment

### Environment Setup

```bash
# Ensure Rust is installed and up to date
rustup update

# Verify cargo and rustc versions
cargo --version
rustc --version

# Install required tools (if not already available)
cargo install cargo-watch  # For development
```

## TLS Certificate Setup

The Enterprise example uses pre-generated certificates located in `/software/tests/certs/` for development and testing.

### Certificate Structure

```
/software/tests/certs/
├── ca-cert.pem       # Certificate Authority (CA) certificate
├── ca-key.pem        # CA private key
├── nats-cert.pem     # NATS server certificate
├── nats-key.pem      # NATS server private key
├── client-cert.pem   # Client certificate for mTLS
└── client-key.pem    # Client private key for mTLS
```

### Development Certificates

**⚠️ Important**: The certificates in `/software/tests/certs/` are for development and testing only. **Never use these certificates in production environments.**

#### Certificate Details

- **CA Certificate**: Self-signed root CA for the test environment
- **Server Certificate**: Issued for `localhost` and common NATS server hostnames
- **Client Certificate**: Used for mutual TLS authentication between agents
- **Validity**: Development certificates have extended validity periods
- **Security Level**: 2048-bit RSA keys with SHA-256 signatures

### Generating Production Certificates

For production deployments, generate new certificates specific to your environment:

#### 1. Create Certificate Authority (CA)

```bash
# Generate CA private key
openssl genrsa -out ca-key.pem 4096

# Generate CA certificate
openssl req -new -x509 -days 365 -key ca-key.pem -out ca-cert.pem \
  -subj "/C=US/ST=YourState/L=YourCity/O=YourOrganization/CN=YourCA"
```

#### 2. Generate Server Certificate

```bash
# Generate server private key
openssl genrsa -out nats-server-key.pem 4096

# Create certificate signing request
openssl req -new -key nats-server-key.pem -out nats-server.csr \
  -subj "/C=US/ST=YourState/L=YourCity/O=YourOrganization/CN=your-nats-server.example.com"

# Generate server certificate signed by CA
openssl x509 -req -days 365 -in nats-server.csr -CA ca-cert.pem -CAkey ca-key.pem \
  -CAcreateserial -out nats-server-cert.pem

# Clean up CSR
rm nats-server.csr
```

#### 3. Generate Client Certificates

```bash
# Generate client private key
openssl genrsa -out client-key.pem 4096

# Create client certificate signing request
openssl req -new -key client-key.pem -out client.csr \
  -subj "/C=US/ST=YourState/L=YourCity/O=YourOrganization/CN=enterprise-client"

# Generate client certificate signed by CA
openssl x509 -req -days 365 -in client.csr -CA ca-cert.pem -CAkey ca-key.pem \
  -CAcreateserial -out client-cert.pem

# Clean up CSR
rm client.csr
```

#### 4. Set Proper Permissions

```bash
# Secure private keys
chmod 600 *-key.pem
chmod 644 *-cert.pem
```

## Configuration

### TOML Configuration File

The Enterprise example uses `config.toml` for centralized configuration:

```toml
# config.toml - Enterprise TLS Configuration

[nats]
urls = ["nats://localhost:4443"]  # TLS-enabled NATS server
reconnect_wait_ms = 2000
max_reconnect_attempts = 5
ping_interval_ms = 30000
max_outstanding_pings = 2

[tls]
enabled = true
verification_mode = "MutualTls"
ca_cert_path = "/path/to/software/tests/certs/ca-cert.pem"
cert_path = "/path/to/software/tests/certs/client-cert.pem"
key_path = "/path/to/software/tests/certs/client-key.pem"

[agents]
# Enterprise agent configuration
timeout_ms = 10000
health_check_interval_ms = 30000
registration_retry_interval_ms = 5000
```

### Environment Variable Overrides

You can override certificate paths using environment variables:

```bash
# Override certificate paths
export TLS_CA_CERT_PATH="/custom/path/ca-cert.pem"
export TLS_CERT_PATH="/custom/path/client-cert.pem"
export TLS_KEY_PATH="/custom/path/client-key.pem"

# Override NATS URLs
export NATS_URLS="nats://production-server:4443"
```

### Configuration Validation

The example includes automatic configuration validation:

- ✅ Certificate file existence checks
- ✅ Certificate validity verification
- ✅ Path resolution (relative/absolute)
- ✅ TLS configuration consistency
- ✅ NATS URL format validation

## Enterprise Example Setup

### 1. Clone and Build

```bash
# Navigate to the Enterprise example
cd software/examples/qollective_a2a_q_console_challenge

# Build all components
cargo build --release

# Verify all binaries are built
ls target/release/ | grep -E "(enterprise|picard|data|spock|scotty|q_console|log_agent)"
```

### 2. NATS Server Configuration

#### Option A: Docker Compose (Recommended)

```yaml
# docker-compose.yml
version: '3.8'
services:
  nats:
    image: nats:2.10-alpine
    ports:
      - "4443:4443"
      - "8222:8222"  # Monitoring
    volumes:
      - ./software/tests/certs:/certs:ro
    command: [
      "--tls",
      "--tlscert=/certs/nats-cert.pem",
      "--tlskey=/certs/nats-key.pem",
      "--tlsverify",
      "--tlscacert=/certs/ca-cert.pem",
      "--port=4443",
      "--http_port=8222"
    ]
```

```bash
# Start NATS with TLS
docker-compose up -d nats

# Verify NATS is running with TLS
curl -k https://localhost:8222/varz
```

#### Option B: Local NATS Server

```bash
# Install NATS server
go install github.com/nats-io/nats-server/v2@latest

# Run with TLS configuration
nats-server \
  --tls \
  --tlscert=/path/to/software/tests/certs/nats-cert.pem \
  --tlskey=/path/to/software/tests/certs/nats-key.pem \
  --tlsverify \
  --tlscacert=/path/to/software/tests/certs/ca-cert.pem \
  --port=4443 \
  --http_port=8222
```

### 3. Configuration File Setup

```bash
# Copy example configuration
cp config.example.toml config.toml

# Edit configuration with your paths
nano config.toml

# Validate configuration
cargo run --bin enterprise -- --validate-config
```

## Running with TLS

### 1. Start Core Services

```bash
# Terminal 1: Start Enterprise Bridge (coordination service)
cargo run --bin enterprise

# Terminal 2: Start Q Console (command interface)
cargo run --bin q_console

# Terminal 3: Start Log Agent (monitoring)
cargo run --bin log_agent
```

### 2. Start Crew Member Agents

```bash
# Terminal 4: Captain Picard
cargo run --bin picard

# Terminal 5: Lieutenant Commander Data
cargo run --bin data

# Terminal 6: Commander Spock
cargo run --bin spock

# Terminal 7: Chief Engineer Scotty
cargo run --bin scotty
```

### 3. Verify TLS Connections

Check the logs for TLS connection confirmations:

```
[INFO] TLS crypto provider initialized successfully
[INFO] Successfully connected to NATS: nats://localhost:4443
[INFO] TLS handshake completed successfully
[INFO] Certificate validation: mutual TLS established
[INFO] Agent registered with Enterprise registry via TLS
```

## Verification

### TLS Connection Tests

The example includes comprehensive test suites:

#### 1. Basic TLS Connectivity Test

```bash
# Test NATS TLS connection
cargo run --bin test_nats_tls_connection

# Expected output:
# ✅ NATS TLS connection test passed
# ✅ TLS handshake completed successfully
# ✅ Certificate validation successful
```

#### 2. Enterprise TLS Integration Test

```bash
# Test Enterprise-specific TLS features
cargo run --bin test_enterprise_tls

# Expected output:
# ✅ Enterprise TLS configuration loaded
# ✅ All crew member agents connected via TLS
# ✅ Certificate validation working
```

#### 3. Comprehensive TLS Test Suite

```bash
# Run full test suite covering all TLS scenarios
cargo run --bin comprehensive_tls_integration_test

# Expected output:
# 🔐 Comprehensive TLS Integration Test Suite
# ✅ Test Suite 1: TLS Configuration and Setup - PASSED
# ✅ Test Suite 2: Basic TLS Connectivity - PASSED
# ✅ Test Suite 3: Agent Registration and Discovery - PASSED
# ✅ Test Suite 4: Certificate Validation and Security - PASSED
# ✅ Test Suite 5: Error Handling and Edge Cases - PASSED
# ✅ Test Suite 6: Performance and Resilience - PASSED
# ✅ Test Suite 7: Multi-Agent Scenarios - PASSED
# ✅ Test Suite 8: Production Readiness - PASSED
```

### Security Verification

#### Certificate Validation Test

```bash
# Test certificate validation logic
cargo run --bin test_secure_registry_operations

# Verify output includes:
# ✅ Valid crew members accepted
# ❌ Unauthorized agents rejected
# ✅ Security clearance levels validated
```

#### TLS Resilience Test

```bash
# Test TLS connection resilience
cargo run --bin test_tls_resilience

# Verify recovery capabilities:
# ✅ Connection recovery after interruption
# ✅ Automatic reconnection with TLS
# ✅ Certificate re-validation
```

### Monitoring TLS Status

#### NATS Server Monitoring

```bash
# Check NATS server TLS status
curl -s http://localhost:8222/varz | jq '.tls'

# View active TLS connections
curl -s http://localhost:8222/connz | jq '.connections[] | select(.tls != null)'
```

#### Agent TLS Status

```bash
# Check agent health with TLS info
cargo run --bin test_q_console_log_agent_tls

# Monitor enterprise bridge status
cargo run --bin enterprise -- --status
```

## Certificate Management

### Certificate Rotation

For production environments, implement certificate rotation:

#### 1. Automated Rotation Script

```bash
#!/bin/bash
# rotate-certs.sh

set -e

CERT_DIR="/path/to/certificates"
BACKUP_DIR="/path/to/certificate-backups"

# Create backup
timestamp=$(date +%Y%m%d_%H%M%S)
mkdir -p "$BACKUP_DIR/$timestamp"
cp "$CERT_DIR"/*.pem "$BACKUP_DIR/$timestamp/"

# Generate new certificates
# (Use the certificate generation commands from above)

# Restart services with new certificates
systemctl restart nats-server
# Restart application services...

echo "Certificate rotation completed: $timestamp"
```

#### 2. Certificate Monitoring

```bash
# Check certificate expiration
openssl x509 -in /path/to/cert.pem -noout -dates

# Automated expiration check
#!/bin/bash
# check-cert-expiry.sh

cert_file="$1"
days_until_expiry=$(openssl x509 -in "$cert_file" -noout -checkend $((30*24*60*60)) && echo "OK" || echo "EXPIRING")

if [ "$days_until_expiry" = "EXPIRING" ]; then
    echo "WARNING: Certificate $cert_file expires within 30 days"
    # Send alert notification
fi
```

### Certificate Storage

#### Secure Storage Best Practices

1. **File Permissions**: Private keys should be readable only by the application user
2. **Backup Strategy**: Regular encrypted backups of CA and certificates
3. **Access Control**: Limit certificate access to necessary services only
4. **Audit Trail**: Log all certificate operations and access

```bash
# Set secure permissions
find /path/to/certs -name "*-key.pem" -exec chmod 600 {} \;
find /path/to/certs -name "*-cert.pem" -exec chmod 644 {} \;
chown -R app-user:app-group /path/to/certs
```

## Security Best Practices

### TLS Configuration

#### 1. Strong Cipher Suites

The implementation uses rustls with secure defaults:

- **TLS 1.2/1.3**: Only modern TLS versions
- **Strong Ciphers**: AES-GCM, ChaCha20-Poly1305
- **Perfect Forward Secrecy**: ECDHE key exchange
- **Certificate Validation**: Full chain verification

#### 2. Certificate Validation

```rust
// Example: Certificate validation configuration
let tls_config = TlsConfig {
    enabled: true,
    verification_mode: VerificationMode::MutualTls,
    ca_cert_path: Some("ca-cert.pem".into()),
    cert_path: Some("client-cert.pem".into()),
    key_path: Some("client-key.pem".into()),
};
```

#### 3. Connection Security

- **mTLS**: Mutual authentication ensures both client and server identity
- **Certificate Pinning**: CA certificate validation prevents MITM attacks
- **Connection Integrity**: All data encrypted in transit
- **Replay Protection**: TLS sequence numbers prevent replay attacks

### Network Security

#### 1. Firewall Configuration

```bash
# Allow only necessary TLS ports
ufw allow 4443/tcp  # NATS TLS port
ufw allow 8222/tcp  # NATS monitoring (internal only)
ufw deny 4222/tcp   # Block non-TLS NATS port
```

#### 2. Network Segmentation

- **Internal Network**: Keep NATS server on internal network
- **DMZ Considerations**: If external access needed, use reverse proxy
- **VPN Access**: Consider VPN for administrative access

### Application Security

#### 1. Agent Authentication

The Enterprise example implements multi-layer authentication:

- **Certificate-based**: X.509 certificates for transport security
- **Application-level**: Crew roster validation
- **Capability-based**: Role-based access control
- **Temporal**: Time-limited authentication tokens

#### 2. Secure Configuration

```rust
// Example: Secure configuration patterns
pub struct SecureConfig {
    // Always validate certificates
    verify_certificates: true,
    // Use strong ciphers only
    min_tls_version: TlsVersion::V1_2,
    // Enable perfect forward secrecy
    require_ecdhe: true,
    // Certificate transparency
    require_sct: false,  // Set true for public CAs
}
```

### Monitoring and Alerting

#### 1. Security Events

Monitor for these security-relevant events:

- Certificate validation failures
- TLS handshake failures
- Unauthorized agent registration attempts
- Connection anomalies
- Certificate expiration warnings

#### 2. Logging Configuration

```rust
// Example: Security-focused logging
use tracing::{info, warn, error};

// Log successful TLS connections
info!("TLS connection established: peer={}, cipher={}", peer_addr, cipher_suite);

// Log authentication events
warn!("Certificate validation failed: agent={}, reason={}", agent_name, reason);

// Log security violations
error!("Unauthorized access attempt: source={}, agent={}", source_ip, agent_name);
```

## Troubleshooting

### Common Issues

#### 1. Certificate Path Issues

```bash
# Error: Certificate file not found
# Solution: Verify paths in config.toml
ls -la $(grep cert_path config.toml | cut -d'"' -f2)

# Error: Permission denied
# Solution: Check file permissions
chmod 644 /path/to/cert.pem
chmod 600 /path/to/key.pem
```

#### 2. TLS Handshake Failures

```bash
# Error: TLS handshake failed
# Check certificate validity
openssl x509 -in cert.pem -noout -text | grep -A2 "Validity"

# Check certificate chain
openssl verify -CAfile ca-cert.pem client-cert.pem
```

#### 3. NATS Connection Issues

```bash
# Error: Connection refused
# Check NATS server status
docker logs nats-container

# Test NATS connectivity
telnet localhost 4443
```

### Debug Commands

```bash
# Enable debug logging
RUST_LOG=debug cargo run --bin enterprise

# Test TLS configuration
cargo run --bin enterprise -- --test-tls

# Validate certificates
cargo run --bin enterprise -- --validate-certs
```

## Production Deployment

### High Availability Setup

#### 1. NATS Cluster with TLS

```yaml
# nats-cluster.yml
version: '3.8'
services:
  nats-1:
    image: nats:2.10-alpine
    ports:
      - "4443:4443"
    volumes:
      - ./certs:/certs:ro
    command: [
      "--cluster_name=enterprise-cluster",
      "--cluster=nats://0.0.0.0:6222",
      "--routes=nats://nats-2:6222,nats://nats-3:6222",
      "--tls", "--tlscert=/certs/nats-cert.pem", "--tlskey=/certs/nats-key.pem",
      "--tlsverify", "--tlscacert=/certs/ca-cert.pem"
    ]
  
  nats-2:
    image: nats:2.10-alpine
    # Similar configuration...
  
  nats-3:
    image: nats:2.10-alpine
    # Similar configuration...
```

#### 2. Load Balancer Configuration

```nginx
# nginx.conf - TLS termination
upstream nats_backend {
    server nats-1:4443;
    server nats-2:4443;
    server nats-3:4443;
}

server {
    listen 443 ssl;
    ssl_certificate /path/to/loadbalancer-cert.pem;
    ssl_certificate_key /path/to/loadbalancer-key.pem;
    
    location / {
        proxy_pass https://nats_backend;
        proxy_ssl_verify on;
        proxy_ssl_trusted_certificate /path/to/ca-cert.pem;
    }
}
```

### Deployment Checklist

- [ ] Production certificates generated and secured
- [ ] NATS cluster configured with TLS
- [ ] Application configuration updated for production
- [ ] Monitoring and alerting configured
- [ ] Certificate rotation procedures established
- [ ] Backup and recovery procedures tested
- [ ] Security audit completed
- [ ] Performance testing completed
- [ ] Documentation updated for operations team

---

## Support and Resources

### Documentation

- [Qollective Framework Documentation](../../README.md)
- [NATS TLS Configuration](https://docs.nats.io/running-a-nats-service/configuration/securing_nats/tls)
- [rustls Documentation](https://docs.rs/rustls/)

### Example Commands Reference

```bash
# Quick start with TLS
cargo run --bin enterprise &
sleep 5
cargo run --bin picard &
cargo run --bin q_console

# Run comprehensive tests
cargo run --bin comprehensive_tls_integration_test

# Monitor TLS connections
cargo run --bin test_q_console_log_agent_tls
```

### Support

For Enterprise TLS setup support:
- Review logs with `RUST_LOG=debug`
- Run diagnostic tests
- Check certificate validity and paths
- Verify NATS server TLS configuration

---

*This guide provides comprehensive TLS setup instructions for the Qollective Enterprise A2A example. For production deployments, always follow your organization's security policies and conduct thorough security reviews.*