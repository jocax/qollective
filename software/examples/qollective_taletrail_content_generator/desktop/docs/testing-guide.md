# Testing Guide - TaleTrail Desktop Application

> Comprehensive guide for running tests in the Tauri V2 + Nuxt 4 desktop application

## Table of Contents

- [Overview](#overview)
- [Test Infrastructure](#test-infrastructure)
- [Quick Reference Commands](#quick-reference-commands)
- [Running Tests](#running-tests)
- [Test Patterns and Best Practices](#test-patterns-and-best-practices)
- [Test Utilities](#test-utilities)
- [Writing New Tests](#writing-new-tests)
- [Troubleshooting](#troubleshooting)

---

## Overview

The TaleTrail Desktop Application uses **Vitest** as the test runner with **@nuxt/test-utils** for Nuxt-aware testing. Tests are organized alongside the code they test using the `__tests__` directory pattern.

**Test Stack:**
- **Test Runner:** Vitest 3.2.4
- **Test Utils:** @nuxt/test-utils 3.20.1
- **Component Testing:** @vue/test-utils 2.4.6
- **Environment:** Nuxt environment (includes Vue composition API)
- **Coverage:** Vitest Coverage v8

---

## Test Infrastructure

### Configuration Files

#### `vitest.config.ts`
Main Vitest configuration with Nuxt integration:
- Environment: `nuxt` (provides Vue composition API and Nuxt composables)
- Setup file: `vitest.setup.ts`
- Pool: `forks` (optimized for parallel execution)
- Coverage provider: `v8` with 80% thresholds

#### `vitest.setup.ts`
Global test setup providing:
- Vue composition API functions (ref, reactive, computed, watch, etc.)
- Nuxt composable stubs (useRouter, useRoute, navigateTo, etc.)
- Auto-imported utilities matching runtime behavior

### Test Locations

```
app/
├── components/
│   └── Mcp/
│       └── __tests__/
│           ├── TemplateBrowser.spec.ts
│           ├── RequestEditor.spec.ts
│           └── ResponseViewer.spec.ts
├── config/
│   └── __tests__/
│       └── constants.spec.ts
├── pages/
│   └── __tests__/
│       ├── index.spec.ts
│       ├── monitoring.spec.ts
│       └── settings.spec.ts
├── stores/
│   └── __tests__/
│       └── mcpTester.spec.ts
└── utils/
    └── __tests__/
        └── testHelpers.ts (utilities, not tests)
```

---

## Quick Reference Commands

### Basic Commands

```bash
# Run all tests (single run)
npm run test
# or
bun test

# Run tests in watch mode (recommended during development)
npm run test:watch
# or
bun test:watch

# Run tests with UI (visual test runner)
npm run test:ui
# or
bun test:ui

# Generate coverage report
npm run test:coverage
# or
bun test:coverage
```

### Advanced Commands

```bash
# Run specific test file
npx vitest run app/config/__tests__/constants.spec.ts

# Run tests matching a pattern
npx vitest run --grep "MCP Testing"

# Run tests in a specific directory
npx vitest run app/components/Mcp/__tests__

# Run tests with verbose output
npx vitest run --reporter=verbose

# Run tests with specific timeout
npx vitest run --testTimeout=10000

# Update snapshots (if using snapshot testing)
npx vitest run -u
```

### Watch Mode Commands

When running in watch mode (`npm run test:watch`), you have access to these interactive commands:

- **a** - Run all tests
- **f** - Run only failed tests
- **t** - Filter by test name pattern
- **p** - Filter by filename pattern
- **q** - Quit watch mode
- **Enter** - Trigger re-run
- **c** - Clear console
- **u** - Update snapshots

---

## Running Tests

### During Development (Watch Mode)

**Recommended:** Use watch mode during active development to get instant feedback:

```bash
bun test:watch
```

This will:
- Re-run tests automatically when files change
- Only run tests affected by your changes (smart mode)
- Provide instant feedback in terminal

### Before Committing (Single Run)

Run all tests once to verify everything passes:

```bash
bun test
```

### Coverage Report Generation

Generate a comprehensive coverage report:

```bash
bun test:coverage
```

Coverage reports are generated in:
- **Text format:** Terminal output
- **HTML format:** `coverage/index.html` (open in browser)
- **JSON format:** `coverage/coverage-final.json`

**Coverage Thresholds:**
- Statements: 80%
- Branches: 80%
- Functions: 80%
- Lines: 80%

### Running Specific Test Suites

**For focused testing during bug fixes (Phase 3):**

```bash
# Run only MCP Testing UI tests
npx vitest run app/components/Mcp/__tests__

# Run only Trail Viewer tests
npx vitest run app/pages/__tests__/index.spec.ts

# Run only Monitoring tests
npx vitest run app/pages/__tests__/monitoring.spec.ts

# Run only store tests
npx vitest run app/stores/__tests__
```

### Running Tests with Filters

```bash
# Run tests matching "template" in the test name
npx vitest run --grep template

# Run tests NOT matching "slow" in the test name
npx vitest run --grep template --grep-invert slow

# Run tests in files matching pattern
npx vitest run --name-pattern="MCP"
```

---

## Test Patterns and Best Practices

### Test Organization

Tests follow this structure (from `constants.spec.ts`):

```typescript
import { describe, expect, it } from "vitest";

describe("Feature Name", () => {
	describe("Sub-feature", () => {
		it("should have expected behavior", () => {
			// Arrange: Setup test data
			const expected = "value";

			// Act: Execute code under test
			const result = functionToTest();

			// Assert: Verify expectations
			expect(result).toBe(expected);
		});
	});
});
```

### Common Test Patterns

#### 1. Array Testing
```typescript
it("should export MCP_SERVERS array", () => {
	expect(MCP_SERVERS).toBeDefined();
	expect(Array.isArray(MCP_SERVERS)).toBe(true);
	expect(MCP_SERVERS).toContain("orchestrator");
});
```

#### 2. Object Testing
```typescript
it("should export PATHS object", () => {
	expect(PATHS).toBeDefined();
	expect(PATHS.TEMPLATES_DIR_RELATIVE).toBe("templates");
});
```

#### 3. Type Safety Testing
```typescript
it("should enforce McpServer type safety", () => {
	const validServer: McpServer = "orchestrator";
	expect(MCP_SERVERS).toContain(validServer);

	// TypeScript will prevent this at compile time:
	// const invalidServer: McpServer = "invalid"; // Error
});
```

#### 4. Component Testing (with test helpers)
```typescript
import { mount } from "@vue/test-utils";
import { setupPinia, setupTauriMock, createMockTemplateData } from "@/utils/__tests__/testHelpers";

describe("TemplateBrowser", () => {
	beforeEach(() => {
		setupPinia();
		setupTauriMock();
	});

	it("should display templates when loaded", async () => {
		const mockData = createMockTemplateData();
		const wrapper = mount(TemplateBrowser);

		// Test component behavior
		expect(wrapper.find(".template-list").exists()).toBe(true);
	});
});
```

---

## Test Utilities

### Available Test Helpers

Located in `app/utils/__tests__/testHelpers.ts`:

#### Tauri Mocking
```typescript
// Setup Tauri mock
const { mockInvoke } = setupTauriMock();

// Mock specific command
mockTauriCommand(mockInvoke, "load_templates", mockTemplatesData);

// Mock multiple commands
mockMultipleTauriCommands(mockInvoke, {
	"load_templates": mockTemplatesData,
	"send_request": mockResponseData,
	"get_history": mockHistoryData
});
```

#### Pinia Store Mocking
```typescript
// Setup Pinia
const pinia = setupPinia();

// Create mock store
const mockStore = createMockStore({
	selectedServer: "orchestrator",
	isLoading: false
});
```

#### NATS Mocking
```typescript
// Mock NATS connection
const connection = createMockNatsConnection(true);

// Mock NATS message
const message = createMockNatsMessage(
	"generation.events.completed",
	{ status: "completed" }
);
```

#### Fixtures

**MCP Testing Fixtures:**
```typescript
// Create mock template info
const templateInfo = createMockTemplateInfo({
	tool_name: "generate_story"
});

// Create mock template data with envelope
const templateData = createMockTemplateData();

// Create grouped templates for all servers
const groupedTemplates = createMockGroupedTemplates(3);

// Create mock response envelope
const response = createMockMcpResponseEnvelope();

// Create mock history entry
const historyEntry = createMockHistoryEntry();
```

**Trail Viewer Fixtures:**
```typescript
// Create single mock trail
const trail = createMockTrailListItem({
	theme: "space adventure",
	age_group: "9-11"
});

// Create multiple mock trails
const trails = createMockTrailList(10);
```

#### Utility Functions
```typescript
// Wait for promises to resolve
await flushPromises();

// Wait for specific time (debounce testing)
await wait(300);

// Create mock error
const error = createMockTauriError("Command failed", "CMD_ERROR");

// Mock file dialog
const path = createMockFileDialogResponse(true, "/selected/path");
```

---

## Writing New Tests

### Step 1: Create Test File

Create a test file in the `__tests__` directory next to the code:

```bash
touch app/components/MyComponent/__tests__/MyComponent.spec.ts
```

### Step 2: Import Dependencies

```typescript
import { describe, expect, it, beforeEach, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { setupPinia, setupTauriMock } from "@/utils/__tests__/testHelpers";
import MyComponent from "../MyComponent.vue";
```

### Step 3: Write Test Structure

```typescript
describe("MyComponent", () => {
	beforeEach(() => {
		// Setup runs before each test
		setupPinia();
		setupTauriMock();
	});

	describe("initialization", () => {
		it("should render correctly", () => {
			const wrapper = mount(MyComponent);
			expect(wrapper.exists()).toBe(true);
		});
	});

	describe("user interactions", () => {
		it("should handle button click", async () => {
			const wrapper = mount(MyComponent);
			await wrapper.find("button").trigger("click");
			expect(wrapper.emitted("click")).toBeTruthy();
		});
	});
});
```

### Step 4: Run Your Test

```bash
# Run in watch mode
npx vitest run app/components/MyComponent/__tests__/MyComponent.spec.ts
```

---

## Troubleshooting

### Common Issues

#### 1. "Cannot find module" errors

**Problem:** Test can't import modules
**Solution:** Check that paths in `tsconfig.json` include test files

#### 2. "useRouter is not defined" errors

**Problem:** Nuxt composables not available in tests
**Solution:** Ensure `vitest.setup.ts` is configured correctly and `setupFiles` in `vitest.config.ts` points to it

#### 3. Tauri invoke() not mocked

**Problem:** Tests try to call real Tauri commands
**Solution:** Use `setupTauriMock()` in your test's `beforeEach()`

```typescript
beforeEach(() => {
	const { mockInvoke } = setupTauriMock();
	mockTauriCommand(mockInvoke, "your_command", mockResponse);
});
```

#### 4. Pinia store not initialized

**Problem:** "getActivePinia was called with no active Pinia"
**Solution:** Call `setupPinia()` in your test's `beforeEach()`

```typescript
beforeEach(() => {
	setupPinia();
});
```

#### 5. Tests timing out

**Problem:** Tests never complete (stuck on async operations)
**Solution:**
- Use `await flushPromises()` after async operations
- Increase timeout: `it("test", { timeout: 10000 }, async () => { ... })`
- Check for unmocked Tauri commands causing hangs

#### 6. Coverage not generated

**Problem:** Coverage report is empty or incomplete
**Solution:**
- Ensure source files are not in `coverage.exclude` in `vitest.config.ts`
- Run with `--coverage` flag explicitly: `npx vitest run --coverage`

### Debugging Tests

#### Enable Verbose Output
```bash
npx vitest run --reporter=verbose
```

#### Use console.log in Tests
```typescript
it("should debug", () => {
	console.log("Debug value:", myVariable);
	expect(myVariable).toBe(expected);
});
```

#### Use Vitest UI for Visual Debugging
```bash
bun test:ui
```
Then open the URL shown (usually `http://localhost:51204/__vitest__/`)

---

## Test Execution Strategy for This Project

### Phase 3: During Bug Fixes (2-8 tests per area)

When fixing bugs in a specific area, run ONLY the tests for that area:

```bash
# Fixing MCP Testing UI bugs
npx vitest run app/components/Mcp/__tests__

# Fixing Trail Viewer bugs
npx vitest run app/pages/__tests__/index.spec.ts

# Fixing Monitoring bugs
npx vitest run app/pages/__tests__/monitoring.spec.ts

# Fixing store bugs
npx vitest run app/stores/__tests__
```

### Phase 4: After All Fixes (20-50 feature-specific tests)

Run all feature-specific tests created for this optimization:

```bash
# Run all tests (should be 20-50 tests total)
bun test

# Or run with coverage
bun test:coverage
```

### Final Verification: Manual Testing

Automated tests complement but don't replace manual testing. Use the manual testing checklist in `docs/manual-testing-checklist.md` for final verification.

---

## Additional Resources

- **Vitest Documentation:** https://vitest.dev/
- **Vue Test Utils:** https://test-utils.vuejs.org/
- **Nuxt Test Utils:** https://nuxt.com/docs/getting-started/testing
- **Testing Best Practices:** See `app/config/__tests__/constants.spec.ts` for patterns

---

## Summary of Test Commands

| Command | Purpose | When to Use |
|---------|---------|-------------|
| `bun test` | Run all tests once | Before committing |
| `bun test:watch` | Run tests in watch mode | During development |
| `bun test:ui` | Visual test runner | Debugging |
| `bun test:coverage` | Generate coverage report | Before releasing |
| `npx vitest run <file>` | Run specific test file | Focused testing |
| `npx vitest run --grep <pattern>` | Filter tests by name | Testing specific features |

---

**Last Updated:** 2025-11-02
**Maintained By:** TaleTrail Development Team
