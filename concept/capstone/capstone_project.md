## Project Name

TaleTrail Content Generator - Multi-Agent MCP System for Adaptive Story Generation

## Project Description

I'm building a production-ready content generation system for TaleTrail that proves two things: first, that agent-based orchestration can reliably produce high-quality educational content, and second, that the pre-alpha Qollective framework is ready for real-world applications. The system generates interactive story graphs (16-node DAGs) where each node contains ~400 words of narrative with three choices leading to different paths. The clever bit is using convergence points to prevent exponential growth - multiple paths merge back together at strategic points. Content adapts based on JSON request parameters like language (starting with German/English), difficulty level, theme, and educational goals.

## Important Project Notice

The Taletrails is another project and is not public. Therefore, parts and code snippets of the Taletrails project are
copied from the Taletrails project to this project. After the capstone and end of the course the Taletrails
project code snipped will be removed from this project.

Example code for capstone project will be put here: (TaleTrail Content Generator)[software/examples/qollective_taletrials_content_generator]


## System Architecture

```
┌──────────────────────────────────────────────────────────┐
│                     HTTP Client Request                  │
│        { "language": "de", "theme": "space", ... }       │
└────────────────────┬─────────────────────────────────────┘
                     │
              ┌──────▼──────┐
              │  HTTP API   │ ← Axum + JWT auth
              │   Gateway   │   Schema validation
              └──────┬──────┘
                     │
              ┌──────▼──────────┐
              │   Orchestrator  │ ← Pipeline coordinator
              │  (MCP Client)   │   Negotiation protocol
              └──────┬──────────┘
                     │
         ┌───────────┼───────────┬──────────────────┬────────────────┐
         │ NATS subjects:        │                  │                │
         │ mcp.story.*           │                  │                │
    ┌────▼────┐          ┌───────▼──────┐   ┌───────▼──────┐    ┌───▼──────┐
    │  Story  │          │   Prompt     │   │   Quality    │    │Constraint│
    │Generator│          │   Heler      │   │   Control    │    │ Enforcer │
    │  Server │          │    Server    │   │    Server    │    │  Server  │
    └────┬────┘          └───────┬──────┘   └───────┬──────┘    └────┬─────┘
         │                       │                  │                │
         └───────────┬───────────┴──────────────────┴────────────────┘
                     │
              ┌──────▼──────┐
              │  LM Studio  │ ← Local LLM (127.0.0.1:1234)
              └─────────────┘
```

## Project Objectives:

1. Understand fundamentals.
* **MCP Protocol Deep Dive**: Figure out how rmcp's tool registration actually works and why the negotiation pattern matters for quality control
* **NATS Subject Design**: Design a proper subject hierarchy that scales - queue groups for load balancing, streams for event monitoring
2. Implement it in Rust.
* **Schema Extension Magic**: Extend Qollective's UnifiedEnvelope to handle our custom TaleTrailEnvelope with 1..n parameters
* **Agent Negotiation Protocol**: Build the correction capability system where agents can say "I can fix this" vs "needs regeneration"
3. Solidify your Rust skills.
* Make Tokio sing with proper concurrent batch processing, get the error handling right with thiserror, and prove that Rust can handle complex distributed systems

## Project Requirements

1. **Schema Extension for Qollective**: Need to prove Qollective can handle custom business logic by extending UnifiedEnvelope with our TaleTrailEnvelope. This isn't just wrapping - we need JSON Schema validation for dynamic parameters where the caller can send whatever combination they want: language, theme, difficulty, age_group, vocabulary_level, etc.

2. **Working Negotiation Protocol**: The cool part - agents don't just validate, they negotiate fixes. Quality Control might say "vocabulary too complex, I can simplify" while Constraint Enforcer says "missing required science fact, needs regeneration". Orchestrator decides the strategy based on capabilities.

## Project Milestones:

### Week 1

1. Understand fundamentals
* **Get the dev environment right**: Spin up NATS in Docker, verify LM Studio is responding at :1234, clone Qollective and understand how HybridTransportClient actually works
* **Design the convergence algorithm**: Work out the math for where paths should merge in a 16-node graph - too early and stories feel repetitive, too late and we lose the benefit
2. Brainstorm
* Sketch out the NATS subject hierarchy (mcp.story.generate.*, mcp.quality.validate.*, etc.) and figure out queue group strategy for load balancing
3. Experiment
* Build a minimal MCP server with rmcp to understand tool registration - just echo back for now
* Test Qollective's envelope extension with a simple custom schema - can we add fields without breaking the transport?
4. Notify your instructor
* Project selected: TaleTrail content generator with focus on Qollective validation

### Week 2

1. **Schema Extension and Core Types**:
* **TaleTrailEnvelope implementation**: Extend UnifiedEnvelope, add JSON Schema validation for our flexible parameters
* **DAG types and messages**: ContentNode, GenerationRequest, ValidationResult with proper serde derives
2. **Build the MCP Servers**:
* **Story Generator**: rmcp tool registration for generate_structure and generate_nodes, integrate with LM Studio client
* **Quality/Constraint servers**: Implement the capability negotiation - each server reports what it can fix vs what needs regeneration

### Week 3

1. **Orchestrator and Integration**:
* **Pipeline orchestration**: Phases (structure → generation → validation → assembly) with proper retry logic and exponential backoff
* **HTTP Gateway**: Axum endpoints that accept our flexible JSON, validate with schema, route through Qollective's transport
2. Showcase progress:
* Demonstrate German story generation end-to-end
* Show how the negotiation protocol handles content issues
* Validate that Qollective handles production workload

## Optional Enhancements:

1. **Streaming generation with SSE**: Instead of waiting 30 seconds for the complete DAG, stream progress updates. "Generating node 5 of 16..." would make the UX way better.

2. **Smart caching layer**: Cache generated nodes by hash of (language + theme + difficulty). If someone asks for a similar story, reuse nodes where possible. Redis would work here.

## Technical Stack

**Why these specific libraries:**
- `qollective` (0.0.1) - My pre-alpha framework that needs real-world validation. If it can handle this, it's production-ready
- `rmcp` (0.7.0) - Latest MCP protocol implementation, chose 0.7 over 0.6 for the improved tool registration macros
- `rig-core` (0.21) - Cleanest LLM abstraction I've found, works great with LM Studio's OpenAI-compatible API
- `axum` (0.8) - Fast, type-safe, and the middleware story is excellent for our schema validation needs
- `async-nats` (0.42) - Rock solid NATS client, need 0.42+ for the JetStream improvements
- `tokio` (1.45) - What else would you use? Need multi-threaded runtime for the concurrent batch processing
- `serde_json` - For our flexible JSON parameters, considering simd-json for performance but premature optimization
- `jsonschema` (0.30) - Validates our 1..n parameters, draft-07 support is good enough

**Dev Environment Setup:**
```bash
# Terminal 1: NATS
docker-compose up nats  # from /Users/ms/development/docker/nats/

# Terminal 2: LM Studio
# Already running at http://127.0.0.1:1234
# Using Mistral-7B for cost-effective local generation

# Terminal 3: Run the beast
cargo run --features="full" --example taletrail-generator
```

**Testing Strategy:**
- Unit tests with mock LLM responses (deterministic)
- Integration tests hitting real LM Studio (expensive but necessary)
- IntelliJ HTTP client files for manual API testing
- Single-threaded test execution because NATS port conflicts are a pain
