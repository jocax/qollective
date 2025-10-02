// ABOUTME: A2A Qollective Example Dispatcher - Star Trek Enterprise crew simulation
// ABOUTME: Runs different Enterprise crew members based on command line arguments

//! A2A Qollective Nats  Example - Star Trek Enterprise Crew
//!
//! This example demonstrates the Qollective framework through a Star Trek Enterprise
//! crew simulation. Each crew member has unique capabilities and responds to Q's
//! cosmic challenges through the A2A (Agent-to-Agent) protocol.
//!
//! ## Usage
//! 
//! Run different crew members:
//! ```bash
//! cargo run --bin enterprise      # USS Enterprise bridge
//! cargo run --bin picard         # Captain Picard
//! cargo run --bin scotty         # Chief Engineer Scott
//! cargo run --bin data           # Lt. Commander Data
//! cargo run --bin spock          # Science Officer Spock
//! cargo run --bin log_agent      # Enterprise log agent
//! cargo run --bin q_console      # Q's challenge console
//! ```
//! 
//! ## Architecture
//! 
//! - **Enterprise**: Central bridge command that coordinates crew responses
//! - **Crew Members**: Individual agents with specialized capabilities
//! - **Q Console**: Generates cosmic challenges for the crew to solve
//! - **Log Agent**: Records all crew activities and performance metrics
//!
//! Each agent demonstrates different aspects of the Qollective framework:
//! - Envelope-based messaging with metadata and context
//! - Agent registration and capability discovery
//! - Hybrid message handling (envelope + raw NATS)
//! - Configuration-driven transport selection
//! - Real-time challenge-response workflows

use qollective::error::Result;
use qollective_a2a_nats_enterprise::enterprise;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting USS Enterprise Bridge...");
    enterprise::main().await
}