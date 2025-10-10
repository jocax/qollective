# NATS NKey Authentication

This directory contains NATS NKey authentication credentials for the TaleTrail Content Generator system.

## What are NKeys?

NKeys are a secure authentication mechanism for NATS servers based on Ed25519 public-key cryptography. Instead of using passwords, each service authenticates using a cryptographic key pair:

- **Private Key (.nk)**: Secret credential that proves the service's identity (like a password)
- **Public Key (.pub)**: Used by the NATS server to verify the service's identity

NKeys provide several security advantages:
- **No password transmission**: Authentication happens cryptographically
- **Per-service credentials**: Each service has unique credentials
- **Subject-level authorization**: Fine-grained access control based on NATS subjects
- **Cryptographically secure**: Based on Ed25519 signatures

## Prerequisites

The NATS CLI is required to generate NKeys:

```bash
# macOS (Homebrew)
brew install nats-io/nats-tools/nats

# Go install
go install github.com/nats-io/natscli/nats@latest

# Or download from releases
# https://github.com/nats-io/natscli/releases
```

## Quick Start

After cloning this repository, you must generate NKeys before starting the system:

```bash
cd nkeys
./nkeys-generate.sh
```

This will create:
- Private keys (`.nk`) for each service
- Public keys (`.pub`) for verification
- `users.conf` file with complete authorization configuration

## Generated Files

The script generates credentials for these services:

| Service | Description | Subjects |
|---------|-------------|----------|
| **story-generator** | Generates story content | Pub: `mcp.story.response.>`, `mcp.events.story.>` <br> Sub: `mcp.story.generate`, `mcp.orchestrator.story.>` |
| **quality-control** | Validates story quality | Pub: `mcp.quality.response.>`, `mcp.events.quality.>` <br> Sub: `mcp.quality.validate`, `mcp.orchestrator.quality.>` |
| **constraint-enforcer** | Enforces content constraints | Pub: `mcp.constraint.response.>`, `mcp.events.constraint.>` <br> Sub: `mcp.constraint.enforce`, `mcp.orchestrator.constraint.>` |
| **prompt-helper** | Optimizes prompts | Pub: `mcp.prompt.response.>`, `mcp.events.prompt.>` <br> Sub: `mcp.prompt.helper`, `mcp.orchestrator.prompt.>` |
| **orchestrator** | Coordinates services | Pub: `mcp.orchestrator.>`, `mcp.story.>`, `mcp.quality.>`, `mcp.constraint.>`, `mcp.prompt.>` <br> Sub: `mcp.events.>`, `mcp.*.response.>` |
| **gateway** | API gateway | Pub: `mcp.orchestrator.request`, `mcp.gateway.>` <br> Sub: `mcp.events.>`, `mcp.orchestrator.response.>` |
| **nats-cli** | Testing and debugging | Full access (`>`) |
| **test** | Integration testing | Full access (password-based) |

## File Structure

```
nkeys/
├── nkeys-generate.sh      # Generation script (committed)
├── nkeys-test-setup.sh    # Test script (committed)
├── README.md              # This file (committed)
├── users.conf             # Authorization config (generated, NOT committed)
├── story-generator.nk     # Private key (generated, NOT committed)
├── story-generator.pub    # Public key (generated, NOT committed)
├── quality-control.nk     # Private key (generated, NOT committed)
├── quality-control.pub    # Public key (generated, NOT committed)
├── constraint-enforcer.nk # Private key (generated, NOT committed)
├── constraint-enforcer.pub # Public key (generated, NOT committed)
├── prompt-helper.nk       # Private key (generated, NOT committed)
├── prompt-helper.pub      # Public key (generated, NOT committed)
├── orchestrator.nk        # Private key (generated, NOT committed)
├── orchestrator.pub       # Public key (generated, NOT committed)
├── gateway.nk             # Private key (generated, NOT committed)
├── gateway.pub            # Public key (generated, NOT committed)
├── nats-cli.nk            # Private key (generated, NOT committed)
└── nats-cli.pub           # Public key (generated, NOT committed)
```

## Configuration Integration

The NATS server configuration (`nats-server.conf`) includes the generated `users.conf`:

```conf
# NKey Authorization - User definitions loaded from generated file
include "nkeys/users.conf"
```

This pattern separates:
- **Static configuration**: Server ports, TLS settings, JetStream config (in `nats-server.conf`)
- **Dynamic credentials**: User authorization rules (in generated `users.conf`)

## Security Considerations

### Critical Security Rules

1. **NEVER commit private keys (.nk files) to git**
   - These are equivalent to passwords and must be kept secret
   - The `.gitignore` is configured to prevent accidental commits

2. **Distribute private keys securely**
   - Use secure channels (encrypted storage, secret managers)
   - Each service needs only its own private key

3. **Regenerate keys if compromised**
   - If a private key is exposed, regenerate all keys
   - Update the NATS server configuration and restart

4. **Rotate keys periodically**
   - Consider regenerating keys on a schedule
   - Implement key rotation procedures for production

### Git Protection

The `.gitignore` is configured to:
- **Block**: All `.nk` files (private keys)
- **Block**: All `.pub` files (public keys - generated)
- **Block**: `users.conf` (generated configuration)
- **Allow**: `generate-nkeys.sh` (the generation script)
- **Allow**: `README.md` (this documentation)

## Principle of Least Privilege

Each service has minimal permissions required for its function:

- **Story Generator**: Can only publish story responses and subscribe to story requests
- **Quality Control**: Can only publish quality validation and subscribe to validation requests
- **Constraint Enforcer**: Can only publish constraint results and subscribe to enforcement requests
- **Prompt Helper**: Can only publish prompt optimizations and subscribe to helper requests
- **Orchestrator**: Has broad access for coordination but limited to MCP subjects
- **Gateway**: Can initiate orchestrator requests and listen for responses
- **NATS CLI**: Full access for testing (use carefully)
- **Test User**: Full access (password-based, for integration testing only)

## How NKey Authentication Works

1. **Service starts** with its private key (.nk file)
2. **NATS server** has the public key in `users.conf`
3. **Service signs** a challenge with its private key
4. **NATS server verifies** the signature using the public key
5. **If valid**, service is authenticated and authorized per its permissions

## Troubleshooting

### "nats command not found"

Install the NATS CLI:

```bash
# macOS (Homebrew)
brew install nats-io/nats-tools/nats

# Go install
go install github.com/nats-io/natscli/nats@latest

# Or download binary
# Visit https://github.com/nats-io/natscli/releases
```

### "Permission denied" errors

Ensure private keys have correct permissions:

```bash
chmod 600 *.nk
chmod 644 *.pub
```

### "Authentication failure" from NATS

1. Verify the service is using the correct private key
2. Ensure `users.conf` has the matching public key
3. Restart the NATS server after regenerating keys

### Regenerating keys

Simply re-run the script (it creates backups automatically):

```bash
./nkeys-generate.sh
```

Old keys are backed up to `backup-YYYYMMDD-HHMMSS/`

## References

- [NATS NKeys Documentation](https://docs.nats.io/running-a-nats-service/configuration/securing_nats/auth_intro/nkey_auth)
- [NATS Authorization](https://docs.nats.io/running-a-nats-service/configuration/securing_nats/authorization)
- [NATS CLI Repository](https://github.com/nats-io/natscli)
- [NKeys Repository](https://github.com/nats-io/nkeys)
- [Ed25519 Cryptography](https://ed25519.cr.yp.to/)

## Support

For issues with:
- **NKey generation**: Check that `nats` CLI is installed and in PATH
- **NATS authentication**: Verify keys match between service and `users.conf`
- **Permissions**: Review subject patterns in `users.conf` and service configuration
- **Git commits**: Ensure `.gitignore` is properly configured

---

**Remember**: Private keys are secrets. Treat them like passwords and never commit them to version control.
