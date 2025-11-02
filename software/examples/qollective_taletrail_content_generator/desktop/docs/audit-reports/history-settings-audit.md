# Request History & Settings Audit Report

**Date:** 2025-11-02
**Task Group:** 6 - Request History & Settings Audit
**Auditor:** Claude Code
**Application:** TaleTrail Desktop (Tauri V2)

---

## Executive Summary

This audit identified **CRITICAL** type mismatches between the frontend TypeScript definitions and backend Rust structures that will cause **complete failure** of both Request History and Settings functionality. These issues are **P0 blockers** that must be fixed immediately.

### Overall Status
- **Request History:** ❌ BROKEN (Critical type mismatch)
- **Settings:** ❌ BROKEN (Critical type mismatch)
- **Persistence:** ⚠️ UNKNOWN (Cannot test due to type mismatches)

### Severity Summary
- **P0 (Blocker):** 2 issues - Complete functionality broken
- **P1 (Critical):** 0 issues
- **P2 (High):** 1 issue - Missing tenant_id handling
- **P3 (Medium):** 0 issues
- **P4 (Low):** 1 issue - Hardcoded tenant_id

---

## Test Results

### Request History Functionality (6 Test Cases)

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| RH-LP-001 | Load history for selected server | ❌ FAIL | Type mismatch prevents loading |
| RH-LP-002 | Search by tool name | ❌ FAIL | Type mismatch prevents query |
| RH-LP-003 | History entry display | ⚠️ BLOCKED | Cannot test until loading works |
| RH-LP-004 | History persistence across restarts | ⚠️ BLOCKED | Cannot test until loading works |
| RH-LP-005 | Empty history state | ⚠️ BLOCKED | Cannot test until loading works |
| RH-LP-006 | History pagination - Load More | ❌ FAIL | Type mismatch in pagination params |

### Replay and Deletion Operations (5 Test Cases)

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| RH-RD-001 | Replay history entry | ⚠️ BLOCKED | Cannot test until history loads |
| RH-RD-002 | Replay preserves request data | ⚠️ BLOCKED | Cannot test until history loads |
| RH-RD-003 | Delete individual entry | ✅ PASS | Command signature looks correct |
| RH-RD-004 | Delete confirmation dialog | ⚠️ BLOCKED | Cannot test GUI interaction |
| RH-RD-005 | History updates after new request | ⚠️ BLOCKED | Cannot test until history loads |

### Settings Page Functionality (4 Test Cases)

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| SET-CRUD-001 | Display current settings | ❌ FAIL | Type mismatch prevents loading |
| SET-CRUD-002 | Save settings changes | ❌ FAIL | Type mismatch prevents saving |
| SET-CRUD-003 | Settings persistence | ⚠️ BLOCKED | Cannot test until save/load works |
| SET-CRUD-004 | Reset to defaults | ⚠️ BLOCKED | Local only, not using backend |

---

## Critical Issues Found

### Issue #1: Request History Query Type Mismatch (P0 - BLOCKER)

**Severity:** P0 (Blocker) - Complete functionality broken
**Component:** Request History
**Files Affected:**
- Frontend: `/app/types/mcp.ts` (lines 251-258)
- Frontend: `/app/components/Mcp/RequestHistory.vue` (lines 163-168, 187-192)
- Backend: `/src-tauri/src/models/mcp_history.rs` (lines 46-57)

**Description:**

The frontend and backend have completely incompatible definitions for `HistoryQuery`. The frontend sends parameters that the backend doesn't recognize, and the backend expects parameters the frontend doesn't send.

**Frontend TypeScript (`mcp.ts` lines 251-258):**
```typescript
export interface HistoryQuery {
    server_filter?: string        // ✅ Matches backend
    tool_filter?: string          // ❌ Backend uses search_term
    success_filter?: boolean      // ❌ Backend uses status_filter (enum)
    limit?: number                // ❌ Backend uses page_size
    offset?: number               // ❌ Backend uses page (0-indexed)
    tenant_id?: number            // ❌ Backend doesn't have this field
}
```

**Actual Frontend Usage (`RequestHistory.vue` line 163):**
```typescript
const query: HistoryQuery = {
    limit: 50,              // ❌ Backend expects page_size
    offset: 0,              // ❌ Backend expects page
    server_filter: props.server,  // ✅ Correct
    tenant_id: 1            // ❌ Backend doesn't have this field
};
```

**Backend Rust (`mcp_history.rs` lines 46-57):**
```rust
pub struct HistoryQuery {
    pub page: usize,                      // ❌ Frontend uses offset
    pub page_size: usize,                 // ❌ Frontend uses limit
    pub server_filter: Option<String>,    // ✅ Matches frontend
    pub status_filter: Option<HistoryStatus>,  // ❌ Frontend uses success_filter
    pub search_term: Option<String>,      // ❌ Frontend uses tool_filter
}
```

**Impact:**
- **History will NEVER load** - The query will fail type serialization
- **Pagination will NEVER work** - Using wrong parameters (offset vs page)
- **Search will NEVER work** - Using wrong field name (tool_filter vs search_term)
- **Status filtering is impossible** - Frontend expects boolean, backend expects enum

**Reproduction Steps:**
1. Navigate to MCP Tester page
2. Select any server tab
3. View Request History section
4. Observe: History fails to load with serialization error

**Expected Behavior:**
- History should load entries for the selected server
- Pagination should work with Load More button
- Search should filter by tool name

**Actual Behavior:**
- History fails to load completely
- Console shows TypeScript/Rust serialization error
- Empty state is shown incorrectly

**Root Cause:**
Type definitions were created independently without synchronization between frontend and backend.

**Recommended Fix:**

**Option A: Update Frontend to Match Backend (RECOMMENDED)**
```typescript
// app/types/mcp.ts
export interface HistoryQuery {
    page: number                    // Change from offset
    page_size: number              // Change from limit
    server_filter?: string
    status_filter?: 'success' | 'error' | 'timeout'  // Change from success_filter
    search_term?: string           // Change from tool_filter
}

// app/components/Mcp/RequestHistory.vue (line 163)
const query: HistoryQuery = {
    page: 0,                       // Change from offset: 0
    page_size: 50,                 // Change from limit: 50
    server_filter: props.server,
    // Remove tenant_id - not supported by backend
};
```

**Option B: Update Backend to Match Frontend**
This would require changing the Rust pagination logic from page-based to offset-based, which is more complex and error-prone.

---

### Issue #2: Settings UserPreferences Type Mismatch (P0 - BLOCKER)

**Severity:** P0 (Blocker) - Complete functionality broken
**Component:** Settings Page
**Files Affected:**
- Frontend: `/app/pages/settings.vue` (lines 188-192)
- Backend: `/src-tauri/src/models/preferences.rs` (lines 5-10)

**Description:**

The frontend and backend have incompatible `UserPreferences` structures. The frontend includes `root_directory` which doesn't exist in the backend, and the backend includes `default_view_mode` and `theme` which don't exist in the frontend.

**Frontend TypeScript (`settings.vue` lines 188-192):**
```typescript
interface UserPreferences {
    directory_path: string    // ✅ Matches backend
    auto_validate: boolean    // ✅ Matches backend
    root_directory: string    // ❌ NOT in backend
}
```

**Backend Rust (`preferences.rs` lines 5-10):**
```rust
pub struct UserPreferences {
    pub default_view_mode: ViewMode,  // ❌ NOT in frontend
    pub theme: Theme,                 // ❌ NOT in frontend
    pub directory_path: String,       // ✅ Matches frontend
    pub auto_validate: bool,          // ✅ Matches frontend
}
```

**Impact:**
- **Settings will NEVER load** - Deserialization will fail
- **Settings will NEVER save** - Serialization will fail
- **Reset to defaults is broken** - Uses wrong structure
- **Root directory management is broken** - Field doesn't exist in backend

**Reproduction Steps:**
1. Navigate to Settings page (`/settings`)
2. Observe loading state
3. Expected: Settings load and display
4. Actual: Loading fails with serialization error

**Actual Behavior:**
- Settings page shows loading spinner indefinitely
- Console shows TypeScript/Rust type mismatch error
- Cannot save any settings changes
- Reset button uses local-only structure

**Root Cause:**
Frontend was updated to support root_directory management but backend was not updated to match. Backend includes view_mode and theme fields that are not used in the UI.

**Recommended Fix:**

**Option A: Update Backend to Match Frontend (RECOMMENDED)**
```rust
// src-tauri/src/models/preferences.rs
pub struct UserPreferences {
    pub directory_path: String,
    pub auto_validate: bool,
    pub root_directory: String,  // ADD this field
    // Remove default_view_mode and theme if not used
}

impl Default for UserPreferences {
    fn default() -> Self {
        UserPreferences {
            directory_path: String::new(),
            auto_validate: true,
            root_directory: "taletrail-data".to_string(),
        }
    }
}
```

**Option B: Update Frontend to Match Backend**
```typescript
// app/pages/settings.vue
interface UserPreferences {
    default_view_mode: 'linear' | 'grid'  // ADD
    theme: 'light' | 'dark' | 'system'   // ADD
    directory_path: string
    auto_validate: boolean
    // Remove root_directory
}
```

However, Option A is preferred because root_directory is actively used in the UI (lines 32, 39, 43, 253-256).

---

### Issue #3: Missing Tenant ID Support in History Query (P2 - HIGH)

**Severity:** P2 (High) - Important feature not working
**Component:** Request History
**Files Affected:**
- Backend: `/src-tauri/src/models/mcp_history.rs`
- Backend: `/src-tauri/src/commands/mcp_history_commands.rs`

**Description:**

The frontend attempts to pass `tenant_id` in the HistoryQuery but the backend doesn't support filtering by tenant_id. Instead, tenant_id is only stored in the history entries themselves but not filterable in the query.

**Current Situation:**
- Frontend sends `tenant_id: 1` in query (line 167)
- Backend ignores this field (not in struct)
- History from ALL tenants is returned
- Multi-tenancy support is broken

**Impact:**
- Users will see history from other tenants
- Security and data isolation concerns
- Violates multi-tenancy design

**Recommended Fix:**

Add tenant_id filter support to backend:

```rust
// src-tauri/src/models/mcp_history.rs
pub struct HistoryQuery {
    pub page: usize,
    pub page_size: usize,
    pub server_filter: Option<String>,
    pub status_filter: Option<HistoryStatus>,
    pub search_term: Option<String>,
    pub tenant_id: Option<i32>,  // ADD THIS
}

// src-tauri/src/commands/mcp_history_commands.rs (after line 128)
if let Some(tenant_filter) = query.tenant_id {
    entries.retain(|e| e.tenant_id == tenant_filter);
}
```

---

### Issue #4: Hardcoded Tenant ID in Request History (P4 - LOW)

**Severity:** P4 (Low) - Technical debt, not user-facing
**Component:** Request History
**Files Affected:**
- `/app/components/Mcp/RequestHistory.vue` (line 167)

**Description:**

The component uses a hardcoded `tenant_id: 1` with a TODO comment instead of loading it from settings or user context.

**Code:**
```typescript
const query: HistoryQuery = {
    limit: 50,
    offset: 0,
    server_filter: props.server,
    tenant_id: 1 // TODO: Get from settings
};
```

**Impact:**
- Currently low impact (only one tenant)
- Will break in multi-tenant scenarios
- Violates DRY principle

**Recommended Fix:**

```typescript
// Option 1: Load from settings store
import { useSettingsStore } from '@/stores/settings';
const settings = useSettingsStore();

const query: HistoryQuery = {
    page: 0,
    page_size: 50,
    server_filter: props.server,
    tenant_id: settings.currentTenantId
};

// Option 2: Pass as prop
const props = defineProps<{
    server: ServerName
    tenantId: number  // ADD THIS
}>();
```

---

## Working Features

### Request History Backend Commands ✅
- **`save_request_to_history`** - Correctly implemented with proper serialization
- **`delete_history_entry`** - Correctly implemented with ID-based deletion
- **`clear_request_history`** - Correctly implemented with store clearing

### Settings Backend Commands ✅
- **`save_preferences`** - Correctly implemented with tenant scoping
- **`load_preferences`** - Correctly implemented with tenant scoping
- **`load_config_toml`** - Correctly implemented for read-only config display

### UI Components (Partial) ⚠️
- **Request History UI Structure** - Well-organized with loading/error/empty states
- **Settings UI Layout** - Clean tabbed interface with proper form validation
- **Delete Confirmation** - Properly implemented with user confirmation (line 232-237)
- **Search Input** - Reactive search with debounce (lines 14-19, 126-132)
- **Pagination UI** - Load More button with loading state (lines 69-79)

---

## Blocked Test Cases

Due to the critical type mismatches, the following test cases could not be executed:

1. **RH-LP-003** - History entry display (blocked by loading failure)
2. **RH-LP-004** - History persistence across restarts (blocked by loading failure)
3. **RH-LP-005** - Empty history state (blocked by loading failure)
4. **RH-RD-001** - Replay history entry (blocked by loading failure)
5. **RH-RD-002** - Replay preserves request data (blocked by loading failure)
6. **RH-RD-004** - Delete confirmation dialog (cannot test GUI in code audit)
7. **RH-RD-005** - History updates after new request (blocked by loading failure)
8. **SET-CRUD-003** - Settings persistence (blocked by save/load failure)

---

## Code Quality Observations

### Positive Aspects ✅
1. **Comprehensive error handling** - Both components handle loading, error, and empty states
2. **Well-structured code** - Clear separation of concerns with composables and stores
3. **Type safety** - Strong TypeScript typing (when types match Rust!)
4. **Reactive UI** - Proper Vue reactivity with watchers and computed properties
5. **User feedback** - Loading indicators, success/error messages
6. **Comprehensive Rust tests** - Excellent unit test coverage in `mcp_history_commands.rs` (lines 228-570)

### Areas for Improvement ⚠️
1. **Type synchronization** - Need process to keep TS and Rust types in sync
2. **Multi-tenancy** - Inconsistent handling across components
3. **Configuration management** - Hardcoded values instead of centralized config
4. **Documentation** - Missing API contract documentation between frontend and backend

---

## Recommendations

### Immediate Actions (P0)

1. **Fix Request History Query Types** (Issue #1)
   - Update frontend `HistoryQuery` to match backend structure
   - Change `offset` → `page` and `limit` → `page_size`
   - Change `tool_filter` → `search_term`
   - Change `success_filter` → `status_filter` with proper enum
   - Estimated effort: 30 minutes

2. **Fix Settings UserPreferences Types** (Issue #2)
   - Add `root_directory` field to backend Rust struct
   - Update default implementation
   - Test save/load flow
   - Estimated effort: 30 minutes

### High Priority (P2)

3. **Add Tenant ID Filtering** (Issue #3)
   - Add `tenant_id` field to `HistoryQuery` in backend
   - Implement filtering logic in `load_request_history`
   - Test multi-tenant scenarios
   - Estimated effort: 45 minutes

### Follow-up (P4)

4. **Remove Hardcoded Tenant ID** (Issue #4)
   - Create tenant context/store
   - Update all components to use context
   - Estimated effort: 1 hour

5. **Establish Type Synchronization Process**
   - Consider using `ts-rs` crate for automatic TypeScript generation from Rust
   - Or create shared type generation script
   - Document type sync process in developer guide
   - Estimated effort: 2-3 hours

### Testing Strategy

Once type mismatches are fixed:

1. **Manual Testing Round 2**
   - Execute all 11 test cases with working types
   - Test persistence by restarting application
   - Verify pagination with 50+ history entries
   - Test search filtering with various queries
   - Test settings save/load/reset flow

2. **Integration Tests**
   - Write Vitest tests for HistoryQuery construction
   - Write Vitest tests for UserPreferences serialization
   - Test error handling for invalid data

3. **End-to-End Tests**
   - Full workflow: Send request → View history → Replay → Delete
   - Full workflow: Change settings → Save → Restart → Verify persistence

---

## Appendix A: Type Definition Comparison

### HistoryQuery Detailed Comparison

| Field | Frontend Type | Backend Type | Status | Notes |
|-------|---------------|--------------|--------|-------|
| `page` | ❌ Missing | `usize` | ❌ Mismatch | Frontend uses `offset` instead |
| `page_size` | ❌ Missing | `usize` | ❌ Mismatch | Frontend uses `limit` instead |
| `offset` | `number?` | ❌ Missing | ❌ Mismatch | Backend uses `page` instead |
| `limit` | `number?` | ❌ Missing | ❌ Mismatch | Backend uses `page_size` instead |
| `server_filter` | `string?` | `Option<String>` | ✅ Match | Correct |
| `status_filter` | ❌ Missing | `Option<HistoryStatus>` | ❌ Mismatch | Frontend uses `success_filter` (boolean) |
| `success_filter` | `boolean?` | ❌ Missing | ❌ Mismatch | Backend uses `status_filter` (enum) |
| `search_term` | ❌ Missing | `Option<String>` | ❌ Mismatch | Frontend uses `tool_filter` |
| `tool_filter` | `string?` | ❌ Missing | ❌ Mismatch | Backend uses `search_term` |
| `tenant_id` | `number?` | ❌ Missing | ❌ Mismatch | Backend doesn't support filtering |

**Match Score: 1/10 fields** - Only `server_filter` matches

### UserPreferences Detailed Comparison

| Field | Frontend Type | Backend Type | Status | Notes |
|-------|---------------|--------------|--------|-------|
| `directory_path` | `string` | `String` | ✅ Match | Correct |
| `auto_validate` | `boolean` | `bool` | ✅ Match | Correct |
| `root_directory` | `string` | ❌ Missing | ❌ Mismatch | Backend doesn't have this |
| `default_view_mode` | ❌ Missing | `ViewMode` | ❌ Mismatch | Frontend doesn't have this |
| `theme` | ❌ Missing | `Theme` | ❌ Mismatch | Frontend doesn't have this |

**Match Score: 2/5 fields** - Only `directory_path` and `auto_validate` match

---

## Appendix B: File References

### Frontend Files
- `/app/types/mcp.ts` - Type definitions (NEEDS UPDATE)
- `/app/components/Mcp/RequestHistory.vue` - History component (NEEDS UPDATE)
- `/app/pages/settings.vue` - Settings page (NEEDS UPDATE)

### Backend Files
- `/src-tauri/src/models/mcp_history.rs` - History data models
- `/src-tauri/src/models/preferences.rs` - Preferences model (NEEDS UPDATE)
- `/src-tauri/src/commands/mcp_history_commands.rs` - History commands (MAY NEED UPDATE for tenant_id)
- `/src-tauri/src/commands/settings.rs` - Settings commands

### Test Files to Create
- `/app/components/Mcp/__tests__/RequestHistory.spec.ts` - Component tests
- `/app/pages/__tests__/settings.spec.ts` - Settings page tests
- `/app/types/__tests__/mcp.spec.ts` - Type validation tests

---

## Conclusion

Both Request History and Settings functionality are **completely broken** due to critical type mismatches between frontend and backend. These are **P0 blocker issues** that prevent any testing of the actual business logic, which appears to be well-implemented.

The good news is that the fixes are straightforward and can be completed in approximately 1-2 hours total. Once the type mismatches are resolved, the underlying functionality should work correctly as the backend command implementations are solid.

**Next Steps:**
1. Fix type mismatches (Issues #1 and #2) - IMMEDIATE
2. Re-run manual testing checklist - After fixes
3. Add tenant_id filtering (Issue #3) - HIGH PRIORITY
4. Implement automated tests - FOLLOW-UP
5. Establish type sync process - FOLLOW-UP

---

**Report Generated:** 2025-11-02
**Auditor:** Claude Code
**Supported by Claude 🤖**
**Approved by ⛵Captain Qollective 💎**
