# MCP Testing UI Audit Report

**Date:** 2025-11-02
**Auditor:** Claude AI Assistant + Manual Testing by ⛵Captain Qollective 💎
**Application:** TaleTrail Desktop - MCP Testing UI
**Version:** Development Build
**Test Environment:** macOS (Darwin 24.6.0)

---

## Executive Summary

This audit evaluates the MCP Testing UI functionality of the TaleTrail Desktop Application. The MCP Testing UI allows users to test Model Context Protocol (MCP) servers via NATS messaging. The UI consists of three main components:

1. **Template Browser** - Select and load template files
2. **Request Editor** - Edit JSON requests and send them to MCP servers
3. **Response Viewer** - Display responses, loading states, and errors

**Total Test Cases:** 22
**Status:** TESTING IN PROGRESS ⚠️

---

## Testing Instructions

⛵Captain Qollective 💎, the Tauri application is now running at **http://0.0.0.0:3004/**

Please execute the following test cases manually and mark each as:
- ✅ **PASS** - Feature works as expected
- ❌ **FAIL** - Feature is broken or doesn't work as expected
- ⚠️ **PARTIAL** - Feature partially works but has issues
- ⏭️ **BLOCKED** - Cannot test due to previous failure

For each failure, please document:
1. What you tried to do
2. What actually happened
3. What should have happened
4. Any error messages or console logs

---

## Section 1: Template Browser Functionality (8 Test Cases)

### MCP-TB-001: Open Template File Picker
**Status:** [ ] Not Tested
**Priority:** P0 (Critical)

**Steps:**
1. Navigate to MCP Tester page (should be already open)
2. Click on the "Orchestrator" tab (first tab)
3. Click on the "Templates" tab (should be selected by default)
4. Click the "Choose Template File" button

**Expected Result:**
- File picker dialog opens
- Dialog should default to the templates directory: `taletrail-data/templates/orchestrator/`
- Dialog should filter for JSON files only

**Actual Result:**
```
[Document what happened here]
```

**Pass/Fail:**
- [ ] PASS
- [ ] FAIL - Reason: ________________
- [ ] PARTIAL - Issue: ________________
- [ ] BLOCKED

---

### MCP-TB-002: Load Template from File Picker
**Status:** [ ] Not Tested
**Priority:** P0 (Critical)

**Steps:**
1. Open file picker (from MCP-TB-001)
2. Navigate to `taletrail-data/templates/orchestrator/`
3. Select any `.json` template file
4. Click Open/Confirm

**Expected Result:**
- Template loads successfully
- Template details display in the browser:
  - Tool name shown
  - File path shown
  - Description shown (if present)
- Store updates with selected template content
- Browser console shows: `[TemplateBrowser] Template selected and loaded: [tool-name]`

**Actual Result:**
```
[Document what happened here]
```

**Pass/Fail:**
- [ ] PASS
- [ ] FAIL - Reason: ________________
- [ ] PARTIAL - Issue: ________________
- [ ] BLOCKED

---

### MCP-TB-003: Initialize Templates from Source
**Status:** [ ] Not Tested
**Priority:** P1 (High)

**Steps:**
1. Navigate to MCP Tester page
2. Click on the "Templates" tab
3. If "First Time Setup" section is visible:
   - Click "Initialize Example Templates" button
4. If not visible:
   - Delete the templates directory manually: `taletrail-data/templates/`
   - Refresh the page and try again

**Expected Result:**
- Loading indicator shows while initializing
- Success message appears: "✓ Templates initialized successfully" (or similar)
- Templates are copied from `desktop/src-tauri/templates/` to `taletrail-data/templates/[server]/`
- The "First Time Setup" section disappears
- Console shows: `[TemplateBrowser] Templates initialized: [result message]`

**Actual Result:**
```
[Document what happened here]
```

**Pass/Fail:**
- [ ] PASS
- [ ] FAIL - Reason: ________________
- [ ] PARTIAL - Issue: ________________
- [ ] BLOCKED

---

### MCP-TB-004: Select Template Updates Store
**Status:** [ ] Not Tested
**Priority:** P0 (Critical)

**Steps:**
1. Load a template using file picker (from MCP-TB-002)
2. Open browser console (Cmd+Option+I on Mac)
3. Type: `$nuxt.$stores.mcpTester.templateContent`
4. Press Enter to view store content

**Expected Result:**
- Store contains the loaded template data
- Template data includes:
  - `subject` field (e.g., "mcp.orchestrator.request")
  - `envelope` object with `meta` and `payload`
  - `payload.tool_call.params.name` contains tool name
  - `payload.tool_call.params.arguments` contains tool arguments
- Console shows template content as a JSON object

**Actual Result:**
```
[Document what happened here]
```

**Pass/Fail:**
- [ ] PASS
- [ ] FAIL - Reason: ________________
- [ ] PARTIAL - Issue: ________________
- [ ] BLOCKED

---

### MCP-TB-005: Template Browser Empty State
**Status:** [ ] Not Tested
**Priority:** P2 (Medium)

**Steps:**
1. Ensure no template is selected (refresh page if needed)
2. Navigate to MCP Tester page
3. Click on "Templates" tab

**Expected Result:**
- Empty state displays:
  - Document icon (gray)
  - "No template selected" message
- "Choose Template File" button is visible
- No errors or crashes

**Actual Result:**
```
[Document what happened here]
```

**Pass/Fail:**
- [ ] PASS
- [ ] FAIL - Reason: ________________
- [ ] PARTIAL - Issue: ________________
- [ ] BLOCKED

---

### MCP-TB-006: Template Browser Error Handling
**Status:** [ ] Not Tested
**Priority:** P1 (High)

**Steps:**
1. Open file picker
2. Navigate to a directory without templates (or create an invalid JSON file)
3. Try to select an invalid file or cancel the dialog

**Expected Result:**
- If file picker cancelled: No error, returns to template browser
- If invalid file selected:
  - Error message displays: "Failed to load template: [error details]"
  - Red alert box with error icon
  - Browser console shows error details
- Application does not crash

**Actual Result:**
```
[Document what happened here]
```

**Pass/Fail:**
- [ ] PASS
- [ ] FAIL - Reason: ________________
- [ ] PARTIAL - Issue: ________________
- [ ] BLOCKED

---

### MCP-TB-007: Template Metadata Display
**Status:** [ ] Not Tested
**Priority:** P2 (Medium)

**Steps:**
1. Load a valid template (from MCP-TB-002)
2. Inspect the displayed template details

**Expected Result:**
- Template card shows:
  - "Selected Template:" label
  - Tool name (extracted from envelope.payload.tool_call.params.name)
  - "File Path:" label
  - Full file path in monospace font
  - "Description:" label (if template has description)
  - Description text
- All text is readable and properly formatted

**Actual Result:**
```
[Document what happened here]
```

**Pass/Fail:**
- [ ] PASS
- [ ] FAIL - Reason: ________________
- [ ] PARTIAL - Issue: ________________
- [ ] BLOCKED

---

### MCP-TB-008: Server Tab Switching Clears Template
**Status:** [ ] Not Tested
**Priority:** P1 (High)

**Steps:**
1. Select "Orchestrator" tab
2. Load a template from orchestrator templates
3. Verify template is displayed
4. Switch to "Story Generator" tab
5. Check Templates panel

**Expected Result:**
- When switching server tabs, selected template is cleared
- Template browser shows empty state again
- Console shows: `[TemplateBrowser] Server changed: orchestrator → story-generator`
- Store's `templateContent` should be cleared or reset
- No errors occur

**Actual Result:**
```
[Document what happened here]
```

**Pass/Fail:**
- [ ] PASS
- [ ] FAIL - Reason: ________________
- [ ] PARTIAL - Issue: ________________
- [ ] BLOCKED

---

## Section 2: Request Editor Functionality (8 Test Cases)

### MCP-RE-001: Load Selected Template Content
**Status:** [ ] Not Tested
**Priority:** P0 (Critical)

**Steps:**
1. Navigate to MCP Tester page
2. Select "Orchestrator" tab
3. Load a template in the Template Browser
4. Wait for auto-switch to "Request Editor" tab (should happen automatically)
5. Inspect the Request Editor content

**Expected Result:**
- Request Editor tab automatically becomes active
- JSON editor displays the full template content:
  - `subject` field
  - `envelope.meta` with request_id, tenant, etc.
  - `envelope.payload.tool_call` with method, params (name and arguments)
- JSON is properly formatted with indentation
- Tool name displays above the editor
- No validation errors shown
- Console shows: `[MCP Tester] Template selected, switching to Request Editor`

**Actual Result:**
```
[Document what happened here]
```

**Pass/Fail:**
- [ ] PASS
- [ ] FAIL - Reason: ________________
- [ ] PARTIAL - Issue: ________________
- [ ] BLOCKED

---

### MCP-RE-002: Edit JSON in Request Editor
**Status:** [ ] Not Tested
**Priority:** P0 (Critical)

**Steps:**
1. Load a template (from MCP-RE-001)
2. In the JSON editor, modify some values (e.g., change an argument value)
3. Keep the JSON valid while editing

**Expected Result:**
- Editor allows text editing
- Syntax highlighting works (if available)
- Changes are reflected in real-time
- Store updates with modified template content
- No errors if JSON remains valid

**Actual Result:**
```
[Document what happened here]
```

**Pass/Fail:**
- [ ] PASS
- [ ] FAIL - Reason: ________________
- [ ] PARTIAL - Issue: ________________
- [ ] BLOCKED

---

### MCP-RE-003: JSON Validation - Valid JSON
**Status:** [ ] Not Tested
**Priority:** P0 (Critical)

**Steps:**
1. Load a template
2. Ensure JSON is valid (should be valid by default)
3. Check the "Send Request" button state
4. Look for validation messages

**Expected Result:**
- No validation error message displayed
- "Send Request" button is ENABLED (not greyed out)
- Store's `canSend` property is `true`
- Console shows no validation errors

**Actual Result:**
```
[Document what happened here]
```

**Pass/Fail:**
- [ ] PASS
- [ ] FAIL - Reason: ________________
- [ ] PARTIAL - Issue: ________________
- [ ] BLOCKED

---

### MCP-RE-004: JSON Validation - Invalid JSON
**Status:** [ ] Not Tested
**Priority:** P0 (Critical)

**Steps:**
1. Load a template
2. Break the JSON syntax (e.g., remove a closing brace `}` or add a trailing comma)
3. Observe validation behavior

**Expected Result:**
- Red error box appears below editor
- Error message shows: "JSON Error: [specific syntax error]"
- "Send Request" button is DISABLED (greyed out)
- Store's `canSend` property is `false`
- Validation error updates in real-time as you edit

**Actual Result:**
```
[Document what happened here]
```

**Pass/Fail:**
- [ ] PASS
- [ ] FAIL - Reason: ________________
- [ ] PARTIAL - Issue: ________________
- [ ] BLOCKED

---

### MCP-RE-005: Server Tab Switching Updates Store
**Status:** [ ] Not Tested
**Priority:** P0 (Critical)

**Steps:**
1. Navigate to "Orchestrator" tab
2. Open browser console
3. Type: `$nuxt.$stores.mcpTester.selectedServer`
4. Switch to "Story Generator" tab
5. Check store value again

**Expected Result:**
- When on "Orchestrator" tab: `selectedServer === "orchestrator"`
- When on "Story Generator" tab: `selectedServer === "story-generator"`
- Console shows:
  - `[MCP Tester] selectedServerName changed: orchestrator → story-generator`
  - `[MCP Tester] Calling store.setServer("story-generator")`
  - `[MCP Tester] After setServer, store.selectedServer = story-generator`
- Store updates correctly
- No errors occur

**Actual Result:**
```
[Document what happened here]
```

**Pass/Fail:**
- [ ] PASS
- [ ] FAIL - Reason: ________________
- [ ] PARTIAL - Issue: ________________
- [ ] BLOCKED

---

### MCP-RE-006: Send Request Button States
**Status:** [ ] Not Tested
**Priority:** P0 (Critical)

**Steps:**
1. **Test 1 - Valid JSON:**
   - Load a template with valid JSON
   - Check button state (should be enabled)
2. **Test 2 - Invalid JSON:**
   - Break the JSON
   - Check button state (should be disabled)
3. **Test 3 - Empty Editor:**
   - Clear the editor content
   - Check button state (should be disabled)

**Expected Result:**
- Button enabled ONLY when:
  - Template content exists
  - JSON is valid
  - Store's `canSend === true`
- Button disabled when:
  - No template loaded
  - JSON is invalid
  - Editor is empty

**Actual Result:**
```
Test 1 (Valid JSON): [Result]
Test 2 (Invalid JSON): [Result]
Test 3 (Empty Editor): [Result]
```

**Pass/Fail:**
- [ ] PASS
- [ ] FAIL - Reason: ________________
- [ ] PARTIAL - Issue: ________________
- [ ] BLOCKED

---

### MCP-RE-007: Subject Mismatch Warning
**Status:** [ ] Not Tested
**Priority:** P2 (Medium)

**Steps:**
1. Load a template on "Orchestrator" tab
2. In the JSON editor, change the `subject` field from:
   - `"mcp.orchestrator.request"` to `"mcp.story-generator.request"`
3. Observe the warning banner

**Expected Result:**
- Yellow warning banner appears at top of Request Editor
- Warning message shows:
  - "Subject Mismatch: Template targets **story-generator** but you're on the **orchestrator** tab."
  - "The request will be sent to story-generator and files will be saved there."
- Banner disappears if you fix the subject to match the current tab
- Console shows subject analysis debug logs

**Actual Result:**
```
[Document what happened here]
```

**Pass/Fail:**
- [ ] PASS
- [ ] FAIL - Reason: ________________
- [ ] PARTIAL - Issue: ________________
- [ ] BLOCKED

---

### MCP-RE-008: Request Editor Persistence
**Status:** [ ] Not Tested
**Priority:** P1 (High)

**Steps:**
1. Load a template
2. Edit the JSON content
3. Navigate to "Response" tab
4. Navigate back to "Request Editor" tab
5. Check if edited content is still there

**Expected Result:**
- Edited request content persists when navigating between tabs
- Store maintains the template content
- Editor displays the same content you edited
- No data loss occurs

**Actual Result:**
```
[Document what happened here]
```

**Pass/Fail:**
- [ ] PASS
- [ ] FAIL - Reason: ________________
- [ ] PARTIAL - Issue: ________________
- [ ] BLOCKED

---

## Section 3: Response Viewer Functionality (6 Test Cases)

### MCP-RV-001: Response Viewer Initial State
**Status:** [ ] Not Tested
**Priority:** P2 (Medium)

**Steps:**
1. Navigate to MCP Tester page (fresh load)
2. Click on "Response" tab
3. Observe the empty state

**Expected Result:**
- Empty state displays:
  - Document icon (large, gray, opacity 50%)
  - "No response yet" message
  - "Send a request to see the response here" help text
- No loading indicators
- No error messages

**Actual Result:**
```
[Document what happened here]
```

**Pass/Fail:**
- [ ] PASS
- [ ] FAIL - Reason: ________________
- [ ] PARTIAL - Issue: ________________
- [ ] BLOCKED

---

### MCP-RV-002: Response Viewer Loading State
**Status:** [ ] Not Tested
**Priority:** P0 (Critical)

**WARNING:** This test requires a running NATS server and MCP server to work!

**Prerequisites:**
- NATS server running
- At least one MCP server running (e.g., orchestrator, story-generator)

**Steps:**
1. Load a template
2. Click "Send Request" button
3. Immediately switch to "Response" tab (should auto-switch)
4. Observe loading state while waiting for response

**Expected Result:**
- Loading indicator displays:
  - Spinning circle animation (primary color with transparent top border)
  - "Waiting for response..." text
- Auto-switches to Response tab after clicking Send
- Loading state shows until response arrives
- Console shows: `[MCP Tester] Request successful, switching to Response tab`

**Actual Result:**
```
[Document what happened here]
```

**Pass/Fail:**
- [ ] PASS
- [ ] FAIL - Reason: ________________
- [ ] PARTIAL - Issue: ________________
- [ ] BLOCKED - No NATS server available

---

### MCP-RV-003: Response Viewer Success State
**Status:** [ ] Not Tested
**Priority:** P0 (Critical)

**WARNING:** This test requires a running NATS server and MCP server!

**Steps:**
1. Send a valid request (from MCP-RV-002)
2. Wait for successful response
3. Inspect the response viewer

**Expected Result:**
- Response displays in a read-only textarea
- JSON is properly formatted with indentation
- Response includes envelope structure:
  - `meta` object (with request_id, tenant, tracing, etc.)
  - `payload` object with `tool_response`
  - `tool_response.content` array
  - `tool_response.isError === false`
- Toolbar buttons available:
  - Copy button (clipboard icon)
  - Download button (download icon)
- Console shows success log with duration and trace_id

**Actual Result:**
```
[Document what happened here]
```

**Pass/Fail:**
- [ ] PASS
- [ ] FAIL - Reason: ________________
- [ ] PARTIAL - Issue: ________________
- [ ] BLOCKED - No NATS server available

---

### MCP-RV-004: Response Viewer Error State
**Status:** [ ] Not Tested
**Priority:** P0 (Critical)

**WARNING:** This test requires a running NATS server!

**Steps:**
1. **Option A - No MCP Server:** Stop all MCP servers and send a request (will timeout)
2. **Option B - Invalid Request:** Send a request with invalid tool name
3. Observe error display

**Expected Result:**
- Error banner displays (red/pink background):
  - Exclamation triangle icon
  - "Request Failed" heading
  - Error message in monospace font
- Error is clear and descriptive
- Auto-switches to Response tab to show error
- Console shows error with duration
- No application crash

**Actual Result:**
```
[Document what happened here]
```

**Pass/Fail:**
- [ ] PASS
- [ ] FAIL - Reason: ________________
- [ ] PARTIAL - Issue: ________________
- [ ] BLOCKED - No NATS server available

---

### MCP-RV-005: Response Envelope Structure Preservation
**Status:** [ ] Not Tested
**Priority:** P1 (High)

**WARNING:** This test requires a running NATS server and MCP server!

**Steps:**
1. Send a valid request
2. Wait for response
3. Copy the response JSON (use Copy button or select and copy text)
4. Paste into a JSON validator or text editor
5. Inspect the structure

**Expected Result:**
- Response preserves full envelope structure:
  ```json
  {
    "meta": {
      "request_id": "uuid...",
      "tenant": "1",
      "timestamp": "ISO-8601 timestamp",
      "tracing": { "trace_id": "...", "span_id": "..." },
      ...
    },
    "payload": {
      "tool_response": {
        "content": [ ... ],
        "isError": false
      }
    }
  }
  ```
- All metadata fields are present
- Tracing information included
- Timestamps are valid ISO-8601 format

**Actual Result:**
```
[Document what happened here]
```

**Pass/Fail:**
- [ ] PASS
- [ ] FAIL - Reason: ________________
- [ ] PARTIAL - Issue: ________________
- [ ] BLOCKED - No NATS server available

---

### MCP-RV-006: Response Viewer Action Buttons
**Status:** [ ] Not Tested
**Priority:** P2 (Medium)

**WARNING:** This test requires a running NATS server and MCP server!

**Steps:**
1. Send a request and get a successful response
2. **Test Copy Button:**
   - Click the clipboard icon button
   - Paste into a text editor
3. **Test Download Button:**
   - Click the download icon button
   - Check your Downloads folder

**Expected Result:**
- **Copy Button:**
  - Response JSON copied to clipboard
  - Console shows: "Response copied to clipboard"
  - Can paste formatted JSON elsewhere
- **Download Button:**
  - File downloads as `mcp-response-[timestamp].json`
  - File contains properly formatted JSON
  - File can be opened and is valid JSON
  - Console shows: "Response downloaded"

**Actual Result:**
```
Copy Button: [Result]
Download Button: [Result]
```

**Pass/Fail:**
- [ ] PASS
- [ ] FAIL - Reason: ________________
- [ ] PARTIAL - Issue: ________________
- [ ] BLOCKED - No NATS server available

---

## Test Execution Summary

### Statistics

| Category | Total | Pass | Fail | Partial | Blocked | Not Tested |
|----------|-------|------|------|---------|---------|------------|
| **Template Browser** | 8 | 0 | 0 | 0 | 0 | 8 |
| **Request Editor** | 8 | 0 | 0 | 0 | 0 | 8 |
| **Response Viewer** | 6 | 0 | 0 | 0 | 0 | 6 |
| **TOTAL** | 22 | 0 | 0 | 0 | 0 | 22 |

**Completion:** 0% (0/22 tests executed)

---

## Issues Found

### Critical Issues (P0)

_No issues found yet - testing in progress_

### High Priority Issues (P1)

_No issues found yet - testing in progress_

### Medium Priority Issues (P2)

_No issues found yet - testing in progress_

### Low Priority Issues (P3)

_No issues found yet - testing in progress_

---

## Issue Details

### [ISSUE-ID] - Issue Title
**Severity:** P0/P1/P2/P3
**Component:** Template Browser / Request Editor / Response Viewer
**Test Case:** MCP-XX-XXX

**Description:**
[Detailed description of the issue]

**Steps to Reproduce:**
1. Step 1
2. Step 2
3. Step 3

**Expected Behavior:**
[What should happen]

**Actual Behavior:**
[What actually happens]

**Screenshots:**
[Attach screenshots if applicable]

**Console Logs:**
```
[Paste relevant console logs]
```

**Proposed Fix:**
[Suggestions for fixing, if known]

---

## Recommendations

### Immediate Actions Required
_To be filled after testing completion_

### Future Improvements
_To be filled after testing completion_

---

## Notes

### Testing Environment Details
- **OS:** macOS Darwin 24.6.0
- **Application:** TaleTrail Desktop (Tauri V2)
- **Nuxt:** 4.2.0
- **Vite:** 7.1.12
- **Vue:** 3.5.22
- **Dev Server:** http://0.0.0.0:3004/

### Prerequisites for Full Testing
- NATS server must be running for MCP-RV-002 through MCP-RV-006
- At least one MCP server must be running (orchestrator, story-generator, etc.)
- Templates must be initialized or available in `taletrail-data/templates/`

### Known Limitations
- Some tests (MCP-RV-002 through MCP-RV-006) cannot be executed without external services (NATS + MCP servers)
- Manual testing required - no automated E2E tests available yet

---

## Sign-Off

**Tester:** ⛵Captain Qollective 💎
**Date Completed:** _[To be filled]_
**Status:** IN PROGRESS ⚠️

---

## Next Steps

1. **Complete Manual Testing** - Execute all 22 test cases
2. **Document Failures** - Record detailed reproduction steps for any failures
3. **Capture Screenshots** - Take screenshots of broken features
4. **Prioritize Issues** - Classify issues by severity (P0-P3)
5. **Create Issue List** - Compile all findings into actionable issue tracker
6. **Begin Fixes** - Start with P0 issues in Task Group 9

---

**Report Generated:** 2025-11-02
**Report Version:** 1.0
**Application Version:** Development Build (Tauri V2)
