# Changelog

All notable changes to TaleTrail Desktop CLI are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-11-02 - Phase 1 MVP Release

### Added

#### Core Infrastructure
- Iocraft v0.7 TUI framework integration with React-like component model
- smol async executor for background tasks and NATS subscriptions
- Comprehensive configuration management with TOML and environment variable overrides
- Structured error handling with custom `AppError` type
- State management with `Arc<RwLock<T>>` pattern for thread-safe shared state

#### MCP Testing Interface
- Pre-built request templates for 5 MCP servers (orchestrator, story-generator, quality-control, constraint-enforcer, prompt-helper)
- Real-time JSON validation with inline error messages
- Multi-tab interface (Templates, Editor, Response, History)
- Request history with persistence, pagination, and replay functionality
- Template browser with search and filtering
- Response viewer with formatted JSON and metadata display
- Server selection and auto-switching based on templates

#### Trail Viewer
- Trail list view with scrolling and pagination
- Multi-criteria filtering:
  - Age group (6-8, 9-11, 12-14, 15-17, 18+)
  - Language (en, de, and extensible)
  - Status (Completed, InProgress, Failed)
  - Tenant ID
- Full-text search across titles, descriptions, and themes
- Bookmark system with persistent storage
- Trail detail view with:
  - Complete metadata display
  - Story structure (DAG) visualization
  - Execution trace inspection
- Fast navigation with keyboard shortcuts

#### NATS Monitoring
- Real-time message feed subscribing to `mcp.>` and `taletrail.>` subjects
- Message filtering:
  - By endpoint (orchestrator, story-generator, quality-control, constraint-enforcer, prompt-helper)
  - By type (Request, Event, Response)
  - By text search (subject and payload)
- Connection diagnostics dashboard showing:
  - Subscription status
  - Message rates (messages/sec)
  - Last message timestamp
  - Connected/disconnected status
- Message detail expansion with formatted JSON
- Auto-scroll toggle for manual inspection vs. live feed
- Circular buffer (max 1000 messages) for memory efficiency

#### Settings Management
- Three-section configuration interface:
  - **NATS Connection**: URL, timeout, TLS certificates, NKey authentication
  - **Directories**: Trails, templates, and execution logs paths
  - **UI Preferences**: Color theme, auto-scroll, page size
- Real-time field validation with inline error messages
- Dirty state tracking (unsaved changes indicator)
- Persistent storage to `~/.config/taletrail-cli/config.toml`
- Environment variable overrides for all settings

#### Reusable UI Components
- `List<T>`: Generic scrollable list component with selection and rendering callbacks
- `TextInput`: Single-line text input with validation and error display
- `TextEditor`: Multi-line text editor for JSON editing
- `Select`: Dropdown/cycle selection component
- `Menu`: Main menu component with keyboard navigation

#### Testing & Quality
- **199 unit and integration tests** covering:
  - Config loading and validation (4 tests)
  - Error handling scenarios (comprehensive coverage)
  - Data models: Trail, MCP, preferences, history (13 tests)
  - NATS client and monitoring (21 tests)
  - Utility functions: file loading, template loading, JSON validation (23 tests)
  - State management: app, MCP, trail, monitoring, settings (40+ tests)
  - UI components: list, editor, form (70 tests)
  - Views: MCP tester, trail viewer, monitoring, settings (18 tests)
  - **10 strategic integration tests** for end-to-end workflows
- 100% test pass rate
- Property-based testing foundations (ready for quickcheck/proptest)

#### Performance
- Startup time: < 2 seconds
- Trail loading: 1000+ trails without lag
- NATS monitoring: 100+ messages/sec without performance degradation
- Search: < 50ms for 1500+ trails
- Filtering: < 50ms with multiple criteria
- Navigation: < 10ms response time
- Memory usage: < 50MB typical, < 100MB with large datasets

#### Documentation
- Comprehensive README with installation, configuration, and usage
- USER_GUIDE with step-by-step instructions for all features
- ARCHITECTURE documentation explaining design decisions
- SHORTCUTS reference for keyboard navigation
- This CHANGELOG documenting features and known issues

### Known Limitations

- Keyboard-only navigation (no mouse support by design)
- Minimum terminal size: 80x24
- ASCII-only UI (no Unicode box drawing for maximum compatibility)
- Read-only trail viewing (no editing from CLI)
- Phase 2 features not implemented: Story Generator, Advanced Search

### Performance Benchmarks

All benchmarks run on Apple M1 Max, 32GB RAM:

| Operation | Dataset | Time | Notes |
|-----------|---------|------|-------|
| Startup | - | 1.8s | Cold start including config load |
| Load trails | 1500 trails | 95ms | Async file I/O |
| Filter trails | 1500 trails, 3 filters | 42ms | In-memory filtering |
| Search trails | 1500 trails | 38ms | Case-insensitive text search |
| Navigate trails | 100 iterations | 8ms | Keyboard navigation |
| Add NATS message | 1000 messages | 120ms | Circular buffer inserts |
| Filter messages | 1000 messages, 2 filters | 15ms | In-memory filtering |

### Dependencies

Core dependencies (see `Cargo.toml` for full list):

- `iocraft = "0.7"` - TUI framework
- `smol = "2.0"` - Async executor
- `async-nats = "0.37"` - NATS client
- `serde = "1.0"` - Serialization
- `serde_json = "1.0"` - JSON handling
- `toml = "0.8"` - Config parsing
- `thiserror = "1.0"` - Error handling
- `anyhow = "1.0"` - Error context
- `chrono = "0.4"` - Date/time handling

## [Unreleased] - Phase 2

### Planned Features

#### Advanced Trail Viewer
- **Linear Reader Mode**: Sequential story reading with automatic progression
- **Interactive Mode**: Navigate story choices and branches interactively
- **Enhanced DAG Visualization**: Collapsible nodes, edge metadata, zoom controls
- **Trail Export**: Export trails to various formats (JSON, Markdown, HTML)

#### Story Generation Dashboard
- **Submit Requests**: Generate stories directly from CLI
- **Progress Tracking**: Real-time generation progress with estimated time
- **Live Results**: View generated trails immediately upon completion
- **Batch Generation**: Generate multiple stories with different parameters

#### Search & Comparison
- **Global Search**: Search across all trails with ranking and relevance scoring
- **Execution Trace Comparison**: Side-by-side comparison of trail executions
- **Diff Highlighting**: Visual diff for trail changes and variations
- **Search History**: Track and replay previous searches

#### Advanced Text Editor
- **Syntax Highlighting**: JSON and TOML syntax highlighting
- **Multi-line Selection**: Select and edit multiple lines at once
- **Undo/Redo**: Full undo/redo support with history
- **Auto-completion**: Context-aware auto-completion for JSON fields
- **Bracket Matching**: Highlight matching brackets and braces

#### User Experience Enhancements
- **Mouse Support**: Optional mouse support for hybrid workflows
- **Unicode Box Drawing**: Improved visual appearance with Unicode characters
- **Custom Themes**: User-defined color schemes and themes
- **Contextual Help**: Inline help hints and tooltips
- **Keyboard Macro Recording**: Record and replay complex key sequences

#### Extensibility
- **Plugin System**: Load external plugins for custom functionality
- **Custom Filters**: User-defined trail and message filters
- **Export Pipelines**: Configurable export formats and transformations
- **Webhook Integration**: Send notifications to external services

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development workflow and guidelines.

## License

MIT License - See [LICENSE](LICENSE) for full text.

## Authors

Built with care by the Qollective Team.

## Acknowledgments

- [Iocraft](https://github.com/ccbrown/iocraft) for the excellent TUI framework
- [async-nats](https://github.com/nats-io/nats.rs) for the robust NATS client
- [smol](https://github.com/smol-rs/smol) for the lightweight async runtime
- TaleTrail community for feedback and testing

---

[0.1.0]: https://github.com/qollective/taletrail/releases/tag/v0.1.0
