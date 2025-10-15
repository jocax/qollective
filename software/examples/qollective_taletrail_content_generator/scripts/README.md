# Infrastructure Check Scripts

This directory contains validation scripts for integration testing infrastructure.

## Scripts

### check_nats.sh
Validates that NATS server is accessible on localhost:5222.

**Usage:**
```bash
./check_nats.sh
```

**Exit Codes:**
- `0`: NATS server is accessible
- `1`: NATS server is not accessible

### check_api_keys.sh
Validates that required API keys are set as environment variables.

**Required Keys:**
- `ANTHROPIC_API_KEY`
- `OPENAI_API_KEY`

**Usage:**
```bash
./check_api_keys.sh
```

**Exit Codes:**
- `0`: All API keys are set
- `1`: One or more API keys are missing

### check_certs.sh
Validates that TLS certificate files exist and are readable.

**Required Files** (in `../../../tests/certs/`):
- `ca.pem`
- `client-cert.pem`
- `client-key.pem`

**Usage:**
```bash
./check_certs.sh
```

**Exit Codes:**
- `0`: All certificates are present and readable
- `1`: One or more certificates are missing or unreadable

## Main Integration Script

The parent directory contains `test_integration.sh` which orchestrates all checks and runs integration tests.

**Usage:**
```bash
cd ..
./test_integration.sh
```

This script will:
1. Run all infrastructure checks
2. If any check fails, exit with clear error messages
3. If all checks pass, run integration tests with `cargo nextest` or `cargo test`

## Requirements

- Bash 4.0+
- netcat (nc) or curl for NATS connectivity checks
- Cargo and rust toolchain for running tests
- Optional: cargo-nextest for enhanced test running
