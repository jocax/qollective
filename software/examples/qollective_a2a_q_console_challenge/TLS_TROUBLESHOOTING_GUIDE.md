# TLS Troubleshooting Guide

> Enterprise A2A NATS Example - TLS Issue Resolution
> Last Updated: 2025-07-29

## Overview

This troubleshooting guide provides solutions for common TLS configuration and connection issues encountered in the Qollective Enterprise A2A NATS example. It covers problem diagnosis, resolution steps, and preventive measures.

## Table of Contents

1. [Quick Diagnosis](#quick-diagnosis)
2. [Certificate Issues](#certificate-issues)
3. [Connection Problems](#connection-problems)
4. [Configuration Errors](#configuration-errors)
5. [NATS Server Issues](#nats-server-issues)
6. [Agent Registration Problems](#agent-registration-problems)
7. [Performance Issues](#performance-issues)
8. [Security Validation Failures](#security-validation-failures)
9. [Environment-Specific Issues](#environment-specific-issues)
10. [Debugging Tools and Commands](#debugging-tools-and-commands)

## Quick Diagnosis

### Initial Health Check

Run these commands to quickly assess TLS status:

```bash
# 1. Test basic NATS TLS connection
cargo run --bin test_nats_tls_connection

# 2. Verify Enterprise configuration
cargo run --bin enterprise -- --validate-config

# 3. Check certificate validity
openssl x509 -in software/tests/certs/client-cert.pem -noout -dates

# 4. Test complete TLS pipeline
cargo run --bin test_enterprise_tls
```

### Common Symptoms and Quick Fixes

| Symptom | Quick Check | Likely Cause |
|---------|-------------|--------------|
| "Connection refused" | `telnet localhost 4443` | NATS server not running |
| "Certificate not found" | `ls -la $(path-from-config)` | Wrong certificate path |
| "TLS handshake failed" | `openssl verify -CAfile ca.pem cert.pem` | Certificate validation issue |
| "Permission denied" | `ls -la cert-files` | File permission problem |
| "Agent registration failed" | Check Enterprise logs | Certificate validation rejection |

## Certificate Issues

### Problem: Certificate File Not Found

**Symptoms:**
```
Error: Failed to load certificate: No such file or directory (os error 2)
Path: /path/to/cert.pem
```

**Diagnosis:**
```bash
# Check if certificate path exists
ls -la /path/to/cert.pem

# Verify config.toml paths
grep -E "(cert_path|key_path|ca_cert_path)" config.toml

# Check current working directory
pwd
```

**Solutions:**

1. **Absolute Paths** (Recommended):
```toml
[tls]
ca_cert_path = "/absolute/path/to/software/tests/certs/ca-cert.pem"
cert_path = "/absolute/path/to/software/tests/certs/client-cert.pem"
key_path = "/absolute/path/to/software/tests/certs/client-key.pem"
```

2. **Relative Paths**:
```toml
[tls]
ca_cert_path = "../../tests/certs/ca-cert.pem"
cert_path = "../../tests/certs/client-cert.pem"
key_path = "../../tests/certs/client-key.pem"
```

3. **Environment Variable Override**:
```bash
export TLS_CA_CERT_PATH="/absolute/path/to/ca-cert.pem"
export TLS_CERT_PATH="/absolute/path/to/client-cert.pem"
export TLS_KEY_PATH="/absolute/path/to/client-key.pem"
```

### Problem: Certificate Validation Failed

**Symptoms:**
```
Error: Certificate verification failed
Reason: unable to get local issuer certificate
```

**Diagnosis:**
```bash
# Verify certificate chain
openssl verify -CAfile software/tests/certs/ca-cert.pem software/tests/certs/client-cert.pem

# Check certificate details
openssl x509 -in software/tests/certs/client-cert.pem -noout -text

# Verify CA certificate
openssl x509 -in software/tests/certs/ca-cert.pem -noout -text
```

**Solutions:**

1. **Check Certificate Chain**:
```bash
# Certificate should be signed by CA
openssl verify -CAfile ca-cert.pem client-cert.pem
# Should output: client-cert.pem: OK
```

2. **Regenerate Certificates** (if corrupted):
```bash
cd software/tests/certs
# Backup existing certificates
cp *.pem backup/

# Regenerate using provided script (if available)
./generate-test-certs.sh
```

3. **Verify Certificate Validity Period**:
```bash
openssl x509 -in client-cert.pem -noout -dates
# Check that current date is within valid period
```

### Problem: Private Key Permission Errors

**Symptoms:**
```
Error: Permission denied (os error 13)
Failed to load private key: /path/to/client-key.pem
```

**Solutions:**
```bash
# Set correct permissions
chmod 600 software/tests/certs/*-key.pem
chmod 644 software/tests/certs/*-cert.pem

# Verify permissions
ls -la software/tests/certs/
# Expected: -rw------- for keys, -rw-r--r-- for certificates

# Check ownership
chown $USER:$USER software/tests/certs/*
```

### Problem: Certificate Format Issues

**Symptoms:**
```
Error: Invalid certificate format
Could not parse PEM certificate
```

**Diagnosis:**
```bash
# Check file encoding
file software/tests/certs/client-cert.pem

# Verify PEM format
head -n 5 software/tests/certs/client-cert.pem
# Should start with: -----BEGIN CERTIFICATE-----
```

**Solutions:**
```bash
# Convert from other formats if needed
# From DER to PEM:
openssl x509 -inform DER -outform PEM -in cert.der -out cert.pem

# From PKCS12 to PEM:
openssl pkcs12 -in cert.p12 -out cert.pem -nodes
```

## Connection Problems

### Problem: NATS Server Connection Refused

**Symptoms:**
```
Error: Connection refused (os error 61)
Failed to connect to NATS server: nats://localhost:4443
```

**Diagnosis:**
```bash
# Check if NATS server is running
telnet localhost 4443

# Check NATS server logs
docker logs nats-container

# Verify port availability
netstat -an | grep 4443
```

**Solutions:**

1. **Start NATS Server with TLS**:
```bash
# Using Docker
docker run -d --name nats-tls \
  -p 4443:4443 -p 8222:8222 \
  -v $(pwd)/software/tests/certs:/certs:ro \
  nats:2.10-alpine \
  --tls --tlscert=/certs/nats-cert.pem --tlskey=/certs/nats-key.pem \
  --tlsverify --tlscacert=/certs/ca-cert.pem --port=4443

# Using local binary
nats-server \
  --tls --tlscert=software/tests/certs/nats-cert.pem \
  --tlskey=software/tests/certs/nats-key.pem \
  --tlsverify --tlscacert=software/tests/certs/ca-cert.pem \
  --port=4443
```

2. **Check Firewall Settings**:
```bash
# Allow NATS TLS port
sudo ufw allow 4443/tcp

# Check iptables rules
sudo iptables -L | grep 4443
```

### Problem: TLS Handshake Timeout

**Symptoms:**
```
Error: TLS handshake timeout
Connection attempt timed out after 30 seconds
```

**Diagnosis:**
```bash
# Test connectivity with timeout
timeout 10 telnet localhost 4443

# Check network latency
ping localhost

# Verify NATS server TLS configuration
curl -s http://localhost:8222/varz | jq '.tls'
```

**Solutions:**

1. **Increase Timeout Values**:
```toml
[nats]
connect_timeout_ms = 10000  # Increase from default 5000
tls_handshake_timeout_ms = 15000  # Add TLS-specific timeout
```

2. **Network Troubleshooting**:
```bash
# Check for network issues
traceroute localhost
netstat -an | grep LISTEN | grep 4443

# Test with openssl client
openssl s_client -connect localhost:4443 -cert client-cert.pem -key client-key.pem
```

### Problem: Certificate Hostname Mismatch

**Symptoms:**
```
Error: Certificate verification failed
Reason: Hostname verification failed
```

**Diagnosis:**
```bash
# Check certificate Subject Alternative Names
openssl x509 -in software/tests/certs/nats-cert.pem -noout -text | grep -A5 "Subject Alternative Name"

# Check certificate Common Name
openssl x509 -in software/tests/certs/nats-cert.pem -noout -subject
```

**Solutions:**

1. **Use Correct Hostname in Configuration**:
```toml
[nats]
urls = ["nats://localhost:4443"]  # Ensure hostname matches certificate
```

2. **Add Hostname to Certificate** (regenerate if needed):
```bash
# Create new certificate with proper SAN
openssl req -new -x509 -days 365 -key nats-key.pem -out nats-cert.pem \
  -subj "/CN=localhost" \
  -addext "subjectAltName = DNS:localhost,DNS:127.0.0.1,IP:127.0.0.1"
```

## Configuration Errors

### Problem: Invalid TOML Configuration

**Symptoms:**
```
Error: Failed to parse configuration
TOML parse error at line 15, column 1
```

**Diagnosis:**
```bash
# Validate TOML syntax
cargo run --bin enterprise -- --validate-config

# Check specific TOML syntax
toml-cli check config.toml  # If toml-cli is installed
```

**Solutions:**

1. **Common TOML Syntax Issues**:
```toml
# Wrong: Missing quotes around strings with special characters
cert_path = /path/with spaces/cert.pem

# Correct: Quoted strings
cert_path = "/path/with spaces/cert.pem"

# Wrong: Trailing comma in TOML
[tls]
enabled = true,

# Correct: No trailing comma
[tls]
enabled = true
```

2. **Validate Configuration Schema**:
```bash
# Use the enterprise binary to validate
cargo run --bin enterprise -- --check-config config.toml
```

### Problem: Environment Variable Conflicts

**Symptoms:**
```
Warning: Environment variable TLS_CERT_PATH overrides config file
Using: /env/path/cert.pem instead of /config/path/cert.pem
```

**Solutions:**
```bash
# Check all TLS-related environment variables
env | grep -E "(TLS_|NATS_)"

# Clear conflicting environment variables
unset TLS_CERT_PATH TLS_KEY_PATH TLS_CA_CERT_PATH

# Or explicitly set them to desired values
export TLS_CERT_PATH="/correct/path/cert.pem"
```

### Problem: Configuration Path Resolution

**Symptoms:**
```
Error: Configuration file not found: config.toml
Searched in: [current directory, ~/.config/, /etc/]
```

**Solutions:**
```bash
# Create config.toml in current directory
cp config.example.toml config.toml

# Or specify config path explicitly
cargo run --bin enterprise -- --config /path/to/config.toml

# Check current working directory
pwd
ls -la config.toml
```

## NATS Server Issues

### Problem: NATS Server Won't Start with TLS

**Symptoms:**
```
NATS Server Error: Failed to load TLS certificate
Certificate file not found or invalid
```

**Diagnosis:**
```bash
# Check NATS server certificate
openssl x509 -in software/tests/certs/nats-cert.pem -noout -text

# Verify certificate permissions
ls -la software/tests/certs/nats-cert.pem
```

**Solutions:**

1. **Check NATS Certificate Configuration**:
```bash
# Test NATS server manually
nats-server \
  --tls \
  --tlscert=software/tests/certs/nats-cert.pem \
  --tlskey=software/tests/certs/nats-key.pem \
  --tlsverify \
  --tlscacert=software/tests/certs/ca-cert.pem \
  --port=4443 \
  --debug
```

2. **Verify NATS Certificate Chain**:
```bash
# Ensure NATS certificate is signed by same CA
openssl verify -CAfile software/tests/certs/ca-cert.pem software/tests/certs/nats-cert.pem
```

### Problem: NATS Server TLS Configuration Mismatch

**Symptoms:**
```
Client Error: TLS handshake failed
Server Error: TLS: bad certificate
```

**Solutions:**

1. **Ensure Matching TLS Configuration**:
```bash
# Server side: Enable client certificate verification
nats-server --tlsverify --tlscacert=ca-cert.pem

# Client side: Provide client certificate
# (This is handled automatically by the Enterprise example)
```

2. **Check TLS Version Compatibility**:
```bash
# Test with openssl to verify TLS versions
openssl s_client -connect localhost:4443 -tls1_2
openssl s_client -connect localhost:4443 -tls1_3
```

## Agent Registration Problems

### Problem: Enterprise Certificate Validation Failed

**Symptoms:**
```
Warning: Agent registration validation failed
Reason: Agent 'Unauthorized Agent' is not authorized for USS Enterprise operations
```

**Diagnosis:**
```bash
# Test certificate validation directly
cargo run --bin test_secure_registry_operations

# Check which crew members are authorized
grep -r "trusted_subjects" src/enterprise_certificate_validator.rs
```

**Solutions:**

1. **Use Authorized Crew Member Names**:
```rust
// Authorized names in the Enterprise example:
"Captain Jean-Luc Picard"
"Lieutenant Commander Data"
"Commander Spock"
"Chief Engineer Montgomery Scott"
"Q Console"
"Enterprise Log Agent"
```

2. **Check Agent Name Format**:
```rust
// Agent names must match exactly:
let agent_info = AgentInfo {
    name: "Captain Jean-Luc Picard".to_string(),  // Exact match required
    capabilities: vec!["command".to_string()],
    // ...
};
```

### Problem: Security Clearance Validation Failed

**Symptoms:**
```
Error: Security clearance validation failed
Agent does not have required clearance level
```

**Solutions:**

1. **Use Valid Security Clearance Levels**:
```rust
// Valid clearance levels:
metadata: HashMap::from([
    ("security_clearance".to_string(), "Alpha".to_string()),    // Highest
    ("security_clearance".to_string(), "Beta".to_string()),     // High
    ("security_clearance".to_string(), "Gamma".to_string()),    // Medium
    ("security_clearance".to_string(), "Omega".to_string()),    // Standard
]);
```

2. **Check Agent Metadata Format**:
```rust
// Ensure proper metadata structure
let agent_info = AgentInfo {
    metadata: HashMap::from([
        ("position".to_string(), "Captain".to_string()),
        ("department".to_string(), "Command".to_string()),
        ("security_clearance".to_string(), "Alpha".to_string()),
    ]),
    // ...
};
```

## Performance Issues

### Problem: Slow TLS Handshake

**Symptoms:**
```
Warning: TLS handshake taking longer than expected
Average handshake time: 2.5 seconds
```

**Diagnosis:**
```bash
# Measure TLS handshake time
time openssl s_client -connect localhost:4443 -cert client-cert.pem -key client-key.pem < /dev/null

# Check system load
top
iostat 1 5
```

**Solutions:**

1. **Optimize Certificate Chain**:
```bash
# Ensure minimal certificate chain
# Single client certificate signed by root CA
```

2. **System Performance Tuning**:
```bash
# Increase file descriptor limits
ulimit -n 4096

# Check for CPU/memory constraints
free -h
nproc
```

### Problem: High Memory Usage

**Symptoms:**
```
Warning: High memory usage detected
Process memory: 512 MB (expected: < 100 MB)
```

**Solutions:**

1. **Connection Pooling Configuration**:
```toml
[nats]
max_connections = 10  # Limit concurrent connections
connection_pool_size = 5
```

2. **Monitor Resource Usage**:
```bash
# Memory profiling
cargo run --bin enterprise &
PID=$!
sleep 10
ps -o pid,rss,vsz,pcpu $PID
kill $PID
```

## Security Validation Failures

### Problem: Certificate Tampering Detection

**Symptoms:**
```
Error: Certificate integrity check failed
Certificate may have been tampered with
```

**Solutions:**

1. **Verify Certificate Checksums**:
```bash
# Create baseline checksums
sha256sum software/tests/certs/*.pem > cert-checksums.txt

# Verify later
sha256sum -c cert-checksums.txt
```

2. **Regenerate Certificates if Compromised**:
```bash
# Remove potentially compromised certificates
rm software/tests/certs/*

# Regenerate from backup or create new ones
# (Follow certificate generation procedures)
```

### Problem: Replay Attack Detection

**Symptoms:**
```
Warning: Potential replay attack detected
Duplicate request ID or timestamp anomaly
```

**Solutions:**

1. **Check System Time Synchronization**:
```bash
# Verify system time
timedatectl status

# Sync with NTP if needed
sudo ntpdate -s time.nist.gov
```

2. **Ensure Unique Request IDs**:
```rust
// Always generate new UUIDs for requests
let mut meta = Meta::default();
meta.request_id = Some(Uuid::now_v7());  // Always new
meta.timestamp = Some(chrono::Utc::now());
```

## Environment-Specific Issues

### Problem: Docker Container TLS Issues

**Symptoms:**
```
Error: TLS certificate verification failed in container
Host certificate not accessible from container
```

**Solutions:**

1. **Proper Volume Mounting**:
```yaml
# docker-compose.yml
services:
  enterprise:
    volumes:
      - ./software/tests/certs:/certs:ro
    environment:
      - TLS_CERT_PATH=/certs/client-cert.pem
      - TLS_KEY_PATH=/certs/client-key.pem
      - TLS_CA_CERT_PATH=/certs/ca-cert.pem
```

2. **Container Network Configuration**:
```yaml
# Ensure NATS server is accessible
services:
  nats:
    networks:
      - enterprise-network
  enterprise:
    networks:
      - enterprise-network
    depends_on:
      - nats
```

### Problem: macOS/Windows Path Issues

**Symptoms:**
```
Error: Invalid path format
Windows: UNC path not supported
macOS: Permission denied accessing /path
```

**Solutions:**

1. **Cross-Platform Paths**:
```toml
# Use forward slashes on all platforms
[tls]
cert_path = "software/tests/certs/client-cert.pem"

# Or use environment variables
cert_path = "${TLS_CERT_PATH}"
```

2. **Platform-Specific Configuration**:
```bash
# Windows
set TLS_CERT_PATH=C:\path\to\certs\client-cert.pem

# macOS/Linux
export TLS_CERT_PATH="/path/to/certs/client-cert.pem"
```

## Debugging Tools and Commands

### Comprehensive Debug Session

```bash
# 1. Enable verbose logging
export RUST_LOG=debug

# 2. Run with TLS debugging
cargo run --bin enterprise 2>&1 | tee enterprise-debug.log

# 3. Test TLS in separate terminal
cargo run --bin test_nats_tls_connection 2>&1 | tee tls-test-debug.log

# 4. Analyze connection state
curl -s http://localhost:8222/connz | jq '.'

# 5. Monitor system resources
htop  # or top on systems without htop
```

### Certificate Debugging Commands

```bash
# Complete certificate chain analysis
echo "=== CA Certificate ==="
openssl x509 -in software/tests/certs/ca-cert.pem -noout -text

echo "=== Client Certificate ==="
openssl x509 -in software/tests/certs/client-cert.pem -noout -text

echo "=== NATS Server Certificate ==="
openssl x509 -in software/tests/certs/nats-cert.pem -noout -text

echo "=== Certificate Verification ==="
openssl verify -CAfile software/tests/certs/ca-cert.pem software/tests/certs/client-cert.pem
openssl verify -CAfile software/tests/certs/ca-cert.pem software/tests/certs/nats-cert.pem
```

### Network Debugging Commands

```bash
# TLS connection testing
echo "=== TLS Connection Test ==="
openssl s_client -connect localhost:4443 \
  -cert software/tests/certs/client-cert.pem \
  -key software/tests/certs/client-key.pem \
  -CAfile software/tests/certs/ca-cert.pem \
  -verify_return_error \
  -state

# NATS protocol testing
echo "=== NATS Server Info ==="
curl -s http://localhost:8222/varz | jq '{version, tls, max_connections, auth_required}'

# Port and process debugging
echo "=== Port Status ==="
netstat -tlnp | grep :4443
lsof -i :4443
```

### Log Analysis Commands

```bash
# Extract TLS-specific log entries
grep -i "tls\|certificate\|handshake" enterprise-debug.log

# Extract error messages
grep -i "error\|failed\|denied" enterprise-debug.log

# Extract timing information
grep -E "([0-9]{2}:[0-9]{2}:[0-9]{2})" enterprise-debug.log | grep -i "tls"

# Count specific events
grep -c "TLS handshake completed" enterprise-debug.log
grep -c "Certificate validation failed" enterprise-debug.log
```

## Prevention and Best Practices

### Proactive Monitoring

1. **Certificate Expiration Monitoring**:
```bash
#!/bin/bash
# check-cert-expiry.sh
for cert in software/tests/certs/*-cert.pem; do
    days=$(openssl x509 -in "$cert" -noout -checkend $((30*24*60*60)) && echo "OK" || echo "EXPIRING")
    if [ "$days" = "EXPIRING" ]; then
        echo "WARNING: $cert expires within 30 days"
    fi
done
```

2. **Health Check Integration**:
```rust
// Add to your monitoring system
pub async fn tls_health_check() -> HealthStatus {
    match create_tls_connection().await {
        Ok(_) => HealthStatus::Healthy,
        Err(_) => HealthStatus::Unhealthy,
    }
}
```

### Configuration Validation

1. **Pre-deployment Validation**:
```bash
# Validate before deployment
cargo run --bin enterprise -- --validate-config --dry-run
cargo run --bin test_enterprise_tls
```

2. **Automated Testing**:
```bash
# Add to CI/CD pipeline
cargo test
cargo run --bin comprehensive_tls_integration_test
```

---

## Emergency Procedures

### TLS Service Recovery

If TLS services are completely down:

1. **Quick Recovery Steps**:
```bash
# Stop all services
pkill -f "cargo run --bin"

# Verify certificates
./scripts/verify-all-certs.sh

# Restart NATS server
docker restart nats-container

# Start services in order
cargo run --bin enterprise &
sleep 5
cargo run --bin q_console &
cargo run --bin log_agent &
```

2. **Fallback to Non-TLS** (Emergency only):
```toml
# Temporary non-TLS configuration
[tls]
enabled = false

[nats]
urls = ["nats://localhost:4222"]  # Non-TLS port
```

### Contact and Escalation

- **Development Team**: Check application logs and configuration
- **Infrastructure Team**: NATS server and network issues
- **Security Team**: Certificate validation and security concerns

---

*This troubleshooting guide covers the most common TLS issues in the Qollective Enterprise A2A example. For persistent issues, enable debug logging and follow the systematic diagnosis procedures outlined above.*