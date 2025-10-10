## Project Capstone Tracking 

Proof of concept for TaleTrails Content Engine based on Qollective NATS capability for MCP.

Project TaleTrails is not open source. It is a private project for my new SaaS company.

Project Qollective is planned to be open source and is currently in pre alpha.

## Sprint 7

- Prepared project qollective to be public in pre alpha to support code reviews ✅
- Design target architecture and start prepare project structure ✅
- Capstone project line under branch  [`capstone`](https://github.com/jocax/qollective/tree/capstone) ✅
- Setup NATS server via docker-compose with support for UI + TLS + Pkeys for authentication ✅

## Spring 8 

- Implement Shared Types Generated (extend json schema for taletrail in qollective envelope) ✅
- Implement Shared Types ✅
- Implement Shared Types Llm (LLM Providers + local LLM providers) ✅
   - Evaluated Shimmy (not usable at the moment for local development because llama LLM image support is broken in latest shimmy)
   - Create a public github project (shimmy-goes-apple-silicon)[https://github.com/jocax/shimmy-goes-apple-silicon] to build better on apple silicon 
   - Evaluated LM Studio (usable at the moment)
   - Added Support for OpenAI, Google, Anthrophic and the 2 local providers (Shimmy and LM Studio)
- Implement MCP server "prompt-helper" 🔄
- Implement MCP server "story-generator" 🔄

## Spring 9

- Finish missing MCP Server 📋
- Finish MCP Client (Orchestrator) 📋

## Spring 10

- Finish Gateway (Rest Edge Service as gateway to MCP Client) 📋

## Spring 11

- Final Presentation 📋

### Legend
Done ✅
Progress 🔄
Planned 📋
Canceled ❌
