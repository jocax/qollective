# A2A Qollective Star Trek Example Components

This directory contains a complete Star Trek-themed demonstration of the Qollective A2A (Agent-to-Agent) framework. Each component represents a different type of agent or service in the distributed system.

## Component Overview

| Filename | Component Type | Role/Description | Command |
|----------|----------------|------------------|---------|
| `enterprise.rs` | A2A Server | USS Enterprise starship and central computer - complete A2A infrastructure | `cargo run --bin enterprise` |
| `picard.rs` | A2A Command Agent | Captain/Bridge command center that coordinates crew responses | `cargo run --bin picard` |
| `spock.rs` | A2A Science Agent | Science officer providing logical analysis and scientific method | `cargo run --bin spock` |
| `data.rs` | A2A Operations Agent | Android operations officer with analytical/computational capabilities | `cargo run --bin data` |
| `scotty.rs` | A2A Engineering Agent | Chief engineer handling technical diagnostics and engineering tasks | `cargo run --bin scotty` |
| `log-agent.rs` | A2A Logging Service | Centralized logging service that receives and displays crew activities | `cargo run --bin log-agent` |
| `q-console.rs` | A2A Test Interface | Interactive command interface (Q entity) for testing the system | `cargo run --bin q-console` |
| `startrek_types.rs` | Data Structures | Shared Star Trek data types for envelope content | (library file) |

## Architecture

- **Server**: `enterprise` serves as the complete A2A server infrastructure
- **Agents**: `picard`, `spock`, `data`, `scotty` are crew member agents with different capabilities
- **Services**: `log-agent` provides centralized logging
- **Testing**: `q-console` provides an interactive interface to test agent communication
- **Types**: `startrek_types` defines shared data structures for rich envelope content

## Prerequisites

- NATS server running on `localhost:4443` with TLS configuration
- TLS certificates available at `/Users/ms/development/docker/nats/certs/server/`

## Usage

1. Start the Enterprise first: `cargo run --bin enterprise`
2. Launch crew member agents: `cargo run --bin picard`, `cargo run --bin spock`, etc.
3. Use the Q console to interact with the system: `cargo run --bin q-console`
4. Monitor logs with: `cargo run --bin log-agent`