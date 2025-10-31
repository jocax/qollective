<p align="center">
    <img width="150" src="./public/logo.png" alt="logo">
</p>
<h1 align="center">NUXTOR</h1>
<p align="center">
A spiritual successor of <a href="https://github.com/NicolaSpadari/vitauri">ViTauri</a>, made with <a href="https://nuxt.com">Nuxt 4</a> and <a href="https://v2.tauri.app">Tauri 2</a>
<br>
Build super fast desktop applications!
</p>

<br />

<p float="left">
	<img src="https://img.shields.io/github/package-json/v/NicolaSpadari/nuxtor" />
	<img src="https://img.shields.io/github/license/NicolaSpadari/nuxtor" />
</p>

<br />

<div align="center">
<img src="./public/screenshot.png">
</div>

<p align="center">Powered by Nuxt 4</p>

Check more screenshots at [preview](https://github.com/NicolaSpadari/nuxtor/blob/main/preview.md)

<br />

## Technologies run-down

- Nuxt v4
- Tauri v2
- NuxtUI v4
- TailwindCSS v4
- Typescript
- ESLint
- Auto imports (for Tauri api too!)

## Functionalities

- Run shell commands from the app
- Send custom notifications to the client (remember to turn on/grant notifications in your computer settings)
- Display OS related informations
- Store and retrieve data locally
- Show tray icon
- Support all Nuxt functionalities (routing/layout/middleware/modules/etc...)

## Setup

  - Before running this app, you need to configure your environment with Rust. Take a look at the [Tauri docs](https://tauri.app/start/prerequisites).
  - This project enforces [bun](https://bun.sh). In order to use another package manager you need to update `package.json` and `tauri.conf.json`
  - The frontend runs on the usual port `3030` of Nuxt, the Tauri server uses the port `3031`. This settings are customizable in the `nuxt.config.ts` and `tauri.conf.json`.
  - Once ready, follow these commands:

  ```sh
  # use this template
  $ npx degit NicolaSpadari/nuxtor my-nuxtor-app

  # go into the folder
  $ cd my-nuxtor-app

  # install dependencies
  $ bun install

  # start the project
  $ bun run tauri:dev
  ```

  This will run the Nuxt frontend and will launch the Tauri window.

## Build

  ```sh
  $ bun run tauri:build
  ```

This command will generate the Nuxt static output and bundle the project under `src-tauri/target`.

## Debug

  ```sh
  $ bun run tauri:build:debug
  ```

The same Tauri bundle will generate under `src-tauri/target`, but with the ability to open the console.

## Notes

- Tauri v2 brings some big refactors, such as packages names and permission management. New permissions have to be granted under `src-tauri/capabilities/main.json`
- Tauri functions are auto imported with the help of a custom module, named like `useTauri<LibraryName>`. If another Tauri plugin is added, then the module has to be updated to support its functions under `app/modules/tauri.ts`
- As per [documentation](https://tauri.app/start/frontend/nuxt/#checklist), Nuxt SSR must be disabled in order for Tauri to act as the backend. Still, all Nuxt goodies will be functional.
- NuxtUI is a very powerful UI library that consolidates design over the entire application. While version 4 is still in alpha, it includes old pro components of the v3.

## MCP Testing UI

This desktop application includes a comprehensive **MCP Testing UI** for testing Model Context Protocol (MCP) servers via NATS messaging.

### Features

- **Template Browser** - Browse 24+ pre-built MCP request templates organized by server
- **Dual-Mode Editor** - Edit requests as JSON or interactive forms
- **Response Viewer** - View formatted responses with content type detection (text/image/resource)
- **Request History** - Track, search, filter, and replay previous requests
- **5 MCP Servers Supported:**
  - Orchestrator - Story generation coordination
  - Story Generator - Interactive story content
  - Quality Control - Content validation
  - Constraint Enforcer - Content constraint checks
  - Prompt Helper - AI prompt recommendations

### Quick Start

1. **Start NATS server:**
   ```sh
   $ cd ..
   $ nats-server --config nats-server.conf
   ```

2. **Start MCP backend servers** (in separate terminals):
   ```sh
   $ cd orchestrator && cargo run
   $ cd story-generator && cargo run
   $ cd quality-control && cargo run
   $ cd constraint-enforcer && cargo run
   $ cd prompt-helper && cargo run
   ```

3. **Launch desktop app:**
   ```sh
   $ bun run tauri:dev
   ```

4. **Access MCP Tester:**
   - Navigate to **System** → **MCP Tester** in the app

### Documentation

- **User Guide:** [`MCP_TESTING_UI_USER_GUIDE.md`](./MCP_TESTING_UI_USER_GUIDE.md) - Complete user documentation
- **Integration Tests:** [`.agent-os/specs/2025-10-29-mcp-testing-ui/INTEGRATION_TEST_PLAN.md`](./.agent-os/specs/2025-10-29-mcp-testing-ui/INTEGRATION_TEST_PLAN.md) - Test plan and procedures
- **Templates:** [`src-tauri/templates/`](./src-tauri/templates/) - 24 pre-built request templates

### Architecture

**Backend (Rust/Tauri):**
- Template management commands
- Request execution via NATS
- History persistence with Tauri store
- Full integration with Qollective envelope architecture

**Frontend (Vue/Nuxt):**
- Responsive three-panel layout
- Real-time JSON validation
- Dynamic form generation from schemas
- Content-aware response rendering

### Testing

Run unit tests:
```sh
$ bun run test
```

Run integration tests with real backends (manual):
- Follow the integration test plan in `.agent-os/specs/2025-10-29-mcp-testing-ui/INTEGRATION_TEST_PLAN.md`

## Configuration

TaleTrail Desktop uses hierarchical configuration with the following priority (lowest to highest):

1. **Constants** - Default values in code (`app/config/constants.ts`, `src-tauri/src/constants.rs`)
2. **config.toml** - Structured configuration file (`src-tauri/config.toml`)
3. **.env file** - Optional environment-specific overrides (not committed to git)
4. **System Environment Variables** - Highest priority overrides

### Quick Start

1. **Copy environment template:**
   ```sh
   $ cp .env.example .env
   ```

2. **Customize values for your environment:**
   ```sh
   # Edit .env file
   $ nano .env
   ```

3. **Run the application:**
   ```sh
   $ bun run tauri:dev
   ```

### Configuration Files

- **`src-tauri/config.toml`** - Main configuration file with defaults
- **`.env`** - Environment-specific overrides (optional, create from `.env.example`)
- **`.env.example`** - Template with all available configuration options
- **`src-tauri/src/constants.rs`** - Backend code-level constants
- **`app/config/constants.ts`** - Frontend code-level constants

### Available Environment Variables

#### Backend Variables (TALETRAIL_*)

```bash
# NATS Configuration
TALETRAIL_NATS_URL=nats://localhost:5222
TALETRAIL_NATS_TIMEOUT_MS=5000
TALETRAIL_NATS_REQUEST_TIMEOUT_MS=180000
TALETRAIL_NATS_TLS_ENABLED=true

# Path Configuration (relative to project root)
TALETRAIL_TRAILS_DIR=test-trails
TALETRAIL_TEMPLATES_DIR=templates
TALETRAIL_CERTS_DIR=certs
TALETRAIL_NKEYS_DIR=nkeys

# MCP Configuration
TALETRAIL_MCP_SERVERS=orchestrator,story-generator,quality-control,constraint-enforcer,prompt-helper
TALETRAIL_MCP_TIMEOUT_MS=180000

# Monitoring
TALETRAIL_MAX_EVENT_BUFFER=1000
TALETRAIL_REQUEST_CLEANUP_TIMEOUT=3600
```

#### Frontend Variables (VITE_*)

```bash
# Development Server Ports
VITE_DEV_PORT=3030
VITE_HMR_PORT=3031

# NATS URL (display only, backend handles connection)
VITE_NATS_URL=nats://localhost:5222
```

### Common Configuration Overrides

**Change NATS Server:**
```bash
export TALETRAIL_NATS_URL="nats://production-server:4222"
```

**Change Development Server Port:**
```bash
export VITE_DEV_PORT=4040
bun run dev  # Now runs on port 4040
```

**Change Trails Directory:**
```bash
export TALETRAIL_TRAILS_DIR="/path/to/my/trails"
```

**Enable Debug Logging:**
```bash
export TALETRAIL_DEBUG=true
```

### Configuration Principles

- **Backend resolves all paths** - Frontend never uses absolute paths
- **Constants first** - All hardcoded values must be in constants modules
- **Environment variable override** - System environment variables have highest priority
- **Type-safe constants** - Frontend constants are fully type-safe with TypeScript

See `.env.example` for complete list of available configuration options and detailed documentation.

## License

MIT License © 2024-PRESENT [NicolaSpadari](https://github.com/NicolaSpadari)
