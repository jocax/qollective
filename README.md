# Qollective

> **Cross-Protocol Data Harmonization Framework written in Rust**

## ⚠️ IMPORTANT NOTICE: PRE-ALPHA SOFTWARE ⚠️

**THIS PROJECT IS IN PRE-ALPHA STAGE AND HAS NOT BEEN RELEASED**

- 🚧 **Status**: Early Development / Proof of Concept
- ⛔ **Production Use**: NOT RECOMMENDED
- 🔬 **Stability**: APIs, schemas, and protocols WILL change
- 📝 **No Guarantees**: This code comes with NO WARRANTIES of any kind
- 🐛 **Expect Bugs**: This is experimental software under active development

### Current Version: 0.0.1 (Unreleased)

---

## What is Qollective?

Qollective is an experimental framework for building distributed systems that communicate seamlessly across multiple transport protocols (REST, gRPC, WebSocket, NATS, JSON-RPC, MCP, A2A) using a unified envelope-first architecture.

## 📖 Documentation

For detailed information about the framework, architecture, and usage:

**[→ View Full Documentation](software/README.md)**

## 🚀 Quick Links

- [Software Documentation](software/README.md) - Main framework documentation
- [Schema System](software/schemas/README.md) - JSON Schema definitions
- [Generator](software/generator/README.md) - Code generation tool
- [Examples](software/examples/) - Different example implementations

## 🏗️ Project Structure

```
qollective/
├── concept/           # Concept
│   ├── capstone/      # Ignore
│   ├── design/        # Diagrams and markdown files describing various aspects of the framework
├── software/          # Main framework implementation
│   ├── src/           # Rust source code
│   ├── schemas/       # JSON Schema definitions
│   ├── generator/     # Code generation tool
│   ├── examples/      # Example implementations
│   └── README.md      # Detailed documentation
└── README.md          # This file
```

## ⚡ Current Status

- ✅ Core envelope architecture implemented
- ✅ Multi-protocol transport layer functional
- ✅ Basic code generation working
- ✅ Example systems demonstrating capabilities
- 🚧 API stabilization in progress
- 🚧 Documentation being expanded
- ❌ Not ready for production use

## 🛠️ Development

This project is under active development. Breaking changes should be expected.

For development setup and contribution guidelines, see the [main documentation](software/README.md).

## 📄 License

This project is licensed under the **Fair Code License v1.0** - see the [LICENSE](LICENSE) file for details.

- ✅ **Permitted**: Personal use, educational use, non-profit use, open source projects
- ❌ **Restricted**: Commercial use without a license
- 📧 **Commercial License**: Contact for commercial licensing terms

---

**Remember**: This is experimental pre-alpha software. Use at your own risk. No support is provided at this time.
