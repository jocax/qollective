# Issue Tracker - TaleTrail Desktop Optimization

**Project:** TaleTrail Desktop Application Optimization
**Date Created:** 2025-11-02
**Last Updated:** 2025-11-02
**Status:** Phase 3 Complete - Pending Final Verification

---

## Overview

This issue tracker consolidates all issues identified during the comprehensive UI audit of the TaleTrail Desktop Application. Issues are prioritized using a P0-P4 scale and tracked through the development lifecycle.

### Priority Definitions

- **P0 (Blocker):** Core functionality completely broken, application unusable
- **P1 (Critical):** Major features not working, workarounds may exist
- **P2 (High):** Important features broken, impacts user experience
- **P3 (Medium):** Minor features broken, low user impact
- **P4 (Low):** Edge cases or cosmetic issues, enhancement opportunities

### Status Definitions

- **Not Started:** Issue identified but not yet addressed
- **In Progress:** Actively being worked on
- **Fixed:** Code changes completed
- **Testing:** In automated test suite
- **Verified:** Manually verified as working
- **Deferred:** Postponed to future iteration
- **Closed:** Resolved and verified

---

## Summary Statistics

### By Priority

| Priority | Total | Fixed | Verified | Deferred | Remaining |
|----------|-------|-------|----------|----------|-----------|
| **P0 (Blocker)** | 2 | 2 | 0 | 0 | 0 |
| **P1 (Critical)** | 0 | 0 | 0 | 0 | 0 |
| **P2 (High)** | 1 | 1 | 0 | 0 | 0 |
| **P3 (Medium)** | 0 | 0 | 0 | 0 | 0 |
| **P4 (Low)** | 4 | 0 | 0 | 4 | 0 |
| **TOTAL** | **7** | **3** | **0** | **4** | **0** |

### By Functional Area

| Functional Area | P0 | P1 | P2 | P3 | P4 | Total |
|----------------|----|----|----|----|----|----|
| MCP Testing UI | 0 | 0 | 0 | 0 | 0 | 0 |
| Trail Viewer | 0 | 0 | 0 | 0 | 3 | 3 |
| Monitoring Page | 0 | 0 | 0 | 0 | 1 | 1 |
| Request History | 1 | 0 | 0 | 0 | 0 | 1 |
| Settings | 1 | 0 | 0 | 0 | 0 | 1 |
| Cross-Cutting | 0 | 0 | 1 | 0 | 0 | 1 |
| **TOTAL** | **2** | **0** | **1** | **0** | **4** | **7** |

### Overall Progress

- **Total Issues:** 7
- **Fixed:** 3 (43%)
- **Verified:** 0 (0%) - ⏳ MANUAL VERIFICATION PENDING
- **Deferred:** 4 (57%)
- **Remaining:** 0 (0%)

---

## P0 (Blocker) Issues

### P0-001: Request History Type Mismatch

**Status:** ✅ FIXED → ⏳ PENDING VERIFICATION
**Priority:** P0 (Blocker)
**Component:** Request History
**Functional Area:** MCP Testing UI

**Description:**
Complete failure of Request History loading due to type mismatch between frontend TypeScript and backend Rust structures.

**Symptoms:**
- History entries fail to load
- Pagination completely broken
- Search by tool name non-functional
- Console shows type deserialization errors

**Root Cause:**
Frontend `HistoryQuery` interface uses different field names than backend Rust struct:
- Frontend: `limit/offset` → Backend: `page_size/page`
- Frontend: `tool_filter` → Backend: `search_term`
- Frontend: `success_filter` (boolean) → Backend: `status_filter` (enum)
- Frontend: `tenant_id` → Backend: no such field

**Impact:**
Request History completely unusable. Users cannot:
- View previous requests
- Replay previous requests
- Search history
- Delete history entries

**Fix Applied:**
1. Updated frontend `HistoryQuery` interface in `app/types/mcp.ts`:
   - Changed `limit` → `page_size`
   - Changed `offset` → `page` (now 0-indexed page number)
   - Changed `tool_filter` → `search_term`
   - Changed `success_filter` → `status_filter` (enum: "all" | "success" | "error")
   - Removed `tenant_id` field

2. Updated `RequestHistory.vue` query construction:
   - Line 163: Uses `page_size` and `page` instead of `limit/offset`
   - Line 187: Uses `search_term` instead of `tool_filter`
   - Removed `tenant_id` from query

3. Updated `HistoryPage` interface:
   - Changed `total` → `total_count` to match backend

**Files Modified:**
- `app/types/mcp.ts` (lines 251-270)
- `app/components/Mcp/RequestHistory.vue` (lines 163-168, 187-192)

**Testing:**
- ✅ Automated: 8 tests passing in `RequestHistory.spec.ts`
- ⏳ Manual: 11 test cases pending (see manual-testing-checklist.md, RH-LP-001 through RH-RD-005)

**Verification Steps:**
1. Navigate to MCP Tester → Select server → Click "Request History" tab
2. Verify history entries load without errors
3. Click "Load More" → Verify pagination works
4. Search by tool name → Verify filtering works
5. Filter by status (success/error) → Verify filtering works
6. Click "Replay" on entry → Verify request loads into editor

**Verified By:** ⏳ PENDING (⛵Captain Qollective 💎)
**Verified Date:** ___________
**Verification Result:**
- [ ] ✅ VERIFIED - All functionality working
- [ ] ❌ REGRESSION - Issue persists or new issues found
- [ ] ⚠️ PARTIAL - Some functionality working, some issues remain

**Notes:**
_[Add verification notes here]_

---

### P0-002: Settings Type Mismatch

**Status:** ✅ FIXED → ⏳ PENDING VERIFICATION
**Priority:** P0 (Blocker)
**Component:** Settings Page
**Functional Area:** Settings

**Description:**
Complete failure of Settings save/load due to type mismatch between frontend TypeScript and backend Rust structures.

**Symptoms:**
- Settings page fails to display current values
- Saving settings fails silently or with errors
- Settings do not persist across app restarts
- Console shows type deserialization errors

**Root Cause:**
Frontend `UserPreferences` interface includes fields not present in backend Rust struct:
- Frontend: includes `root_directory` → Backend: missing this field
- Frontend: includes `default_view_mode` and `theme` → Backend: tries to deserialize but these are frontend-only state

**Impact:**
Settings completely unusable. Users cannot:
- View current settings
- Change NATS connection settings
- Save configuration changes
- Persist settings across sessions

**Fix Applied:**
1. Added `root_directory` field to backend `UserPreferences` struct:
   - File: `src-tauri/src/models/preferences.rs`
   - Added: `pub root_directory: Option<String>`

2. Removed `default_view_mode` and `theme` from backend struct:
   - These are frontend-only state, not persisted
   - Kept in frontend TypeScript interface only

3. Full type alignment achieved between frontend and backend

**Files Modified:**
- `src-tauri/src/models/preferences.rs` (added root_directory field)

**Testing:**
- ✅ Automated: 6 tests passing in `settings.spec.ts`
- ⏳ Manual: 4 test cases pending (see manual-testing-checklist.md, SET-CRUD-001 through SET-CRUD-004)

**Verification Steps:**
1. Navigate to Settings page
2. Verify all settings display current values (NATS URL, port, timeout, etc.)
3. Change a setting (e.g., NATS server URL)
4. Click "Save Settings"
5. Verify success message appears
6. Close and reopen application
7. Navigate to Settings page
8. Verify changed setting persisted

**Verified By:** ⏳ PENDING (⛵Captain Qollective 💎)
**Verified Date:** ___________
**Verification Result:**
- [ ] ✅ VERIFIED - All functionality working
- [ ] ❌ REGRESSION - Issue persists or new issues found
- [ ] ⚠️ PARTIAL - Some functionality working, some issues remain

**Notes:**
_[Add verification notes here]_

---

## P1 (Critical) Issues

**No P1 issues found.**

All critical functionality is either working correctly or has been fixed (P0 fixes above).

---

## P2 (High) Issues

### P2-001: State Management - Error Clearing Bug

**Status:** ✅ FIXED → ⏳ PENDING VERIFICATION
**Priority:** P2 (High)
**Component:** MCP Tester Store
**Functional Area:** Cross-Cutting Concerns

**Description:**
Error state is incorrectly cleared when switching tabs in Response Viewer, causing users to lose error messages.

**Symptoms:**
- User sends request that fails with error
- Error displays in Response Viewer
- User switches to different tab (e.g., Request Editor)
- User switches back to Response tab
- Error message is gone, user doesn't know request failed

**Root Cause:**
`clearResponse()` action in `app/stores/mcpTester.ts` was clearing both:
- `response` state (correct - should clear)
- `error` state (incorrect - should preserve for display)

The method was being called on tab switches, unnecessarily clearing error state.

**Impact:**
Medium-high. Users lose error information and may not realize requests failed. Impacts debugging and troubleshooting.

**Fix Applied:**
Modified `clearResponse()` method in `app/stores/mcpTester.ts` (lines 125-128):

**Before:**
```typescript
clearResponse() {
    this.response = null;
    this.error = null;  // ❌ WRONG - clears error
    this.loading = false;
}
```

**After:**
```typescript
clearResponse() {
    this.response = null;
    // error is intentionally NOT cleared here - preserve it for display
    this.loading = false;
}
```

**Files Modified:**
- `app/stores/mcpTester.ts` (lines 125-128)

**Testing:**
- ✅ Automated: 17 tests passing in `mcpTester.spec.ts` (includes error persistence tests)
- ⏳ Manual: 5 test cases pending (see manual-testing-checklist.md, XC-ERR-001 through XC-ERR-005)

**Verification Steps:**
1. Navigate to MCP Tester
2. Send a request that will fail (e.g., invalid JSON or non-existent tool)
3. Verify error displays in Response Viewer
4. Click on Request Editor tab
5. Click back to Response tab
6. **Verify:** Error message still displays (not cleared)
7. Send a new successful request
8. **Verify:** Error is cleared and replaced with new response

**Verified By:** ⏳ PENDING (⛵Captain Qollective 💎)
**Verified Date:** ___________
**Verification Result:**
- [ ] ✅ VERIFIED - Error persists correctly
- [ ] ❌ REGRESSION - Error still being cleared
- [ ] ⚠️ PARTIAL - Error persists but other issues found

**Notes:**
_[Add verification notes here]_

---

## P3 (Medium) Issues

**No P3 issues found.**

All medium-priority functionality is working correctly.

---

## P4 (Low) Issues - Deferred to Future Iterations

### P4-001: Trail Viewer - "Load More" Pagination Not Implemented

**Status:** ⏭️ DEFERRED (Enhancement Opportunity)
**Priority:** P4 (Low)
**Component:** Trail Viewer
**Functional Area:** Trail Viewer

**Description:**
Pagination with "Load More" button is not implemented. All trails load at once, which may impact performance with 100+ trail files.

**Current Behavior:**
- All trails in directory load immediately
- No pagination controls visible
- Performance acceptable for typical use cases (< 50 trails)

**Enhancement:**
- Implement "Load More" button to load trails in batches
- Load 50 trails initially, then 50 more on each click
- Show loading indicator during batch loading

**Impact:**
Low. Most users have < 50 trails, performance is acceptable. Only impacts users with 100+ trails.

**Workaround:**
Performance is acceptable for typical use cases. Users with many trails can organize into subdirectories.

**Deferred Reason:**
Enhancement opportunity, not a bug. Existing functionality works correctly.

**Estimated Effort:** Small (1-2 days)

**Recommended Priority for Future:** P3 (if user feedback indicates need)

---

### P4-002: Trail Viewer - Tenant Filter Not Included in "Clear Filters"

**Status:** ⏭️ DEFERRED (Minor Enhancement)
**Priority:** P4 (Low)
**Component:** Trail Viewer
**Functional Area:** Trail Viewer

**Description:**
"Clear Filters" button clears search, age group, language, and status filters but does not reset tenant selector.

**Current Behavior:**
- Clicking "Clear Filters" resets:
  - Search text
  - Age group dropdown
  - Language dropdown
  - Status dropdown
- Does NOT reset:
  - Tenant selector

**Enhancement:**
- Include tenant selector in "Clear All Filters" action
- OR add separate "Reset Tenant" button

**Impact:**
Low. Tenant is typically intentionally selected and users may not want it reset when clearing other filters.

**Workaround:**
Manually change tenant selector if needed.

**Deferred Reason:**
Current behavior may be intentional design. Users typically select tenant deliberately and don't want it changed.

**Estimated Effort:** Small (1 day)

**Recommended Priority for Future:** P4 (consider UX research first)

---

### P4-003: Trail Viewer - Bulk Operations Not Implemented

**Status:** ⏭️ DEFERRED (Enhancement Opportunity)
**Priority:** P4 (Low)
**Component:** Trail Viewer
**Functional Area:** Trail Viewer

**Description:**
Bulk operations (bulk delete, bulk export) are not implemented. Users must operate on trails one at a time.

**Current Behavior:**
- Delete one trail at a time
- No select-all checkbox
- No bulk actions menu

**Enhancement:**
- Add checkbox to each trail card
- Add "Select All" / "Select None" controls
- Add bulk actions menu:
  - Bulk Delete (with confirmation)
  - Bulk Export (download multiple trails)
  - Bulk Status Change (draft → published, etc.)

**Impact:**
Low. Most users work with individual trails. Power users with many trails would benefit.

**Workaround:**
Delete or manage trails individually.

**Deferred Reason:**
Enhancement opportunity for power users, not blocking normal workflows.

**Estimated Effort:** Medium (3-5 days)

**Recommended Priority for Future:** P3 (if user feedback indicates need)

---

### P4-004: Monitoring Page - "Clear Filters" Button Not Present

**Status:** ⏭️ DEFERRED (Minor Enhancement)
**Priority:** P4 (Low)
**Component:** Monitoring Page
**Functional Area:** Monitoring

**Description:**
No dedicated "Clear All Filters" button on Monitoring Page. Users must manually clear endpoint filter and text search.

**Current Behavior:**
- Endpoint filter: Manually select "All Endpoints"
- Text search: Manually clear search box
- No single "Clear All Filters" button

**Enhancement:**
- Add "Clear All Filters" button
- Clicking button resets both endpoint and text filters
- Similar to Trail Viewer's "Clear Filters" button

**Impact:**
Low. Filters are simple to clear manually (2 clicks/actions).

**Workaround:**
- Select "All Endpoints" from dropdown
- Clear text in search box

**Deferred Reason:**
Minor UX enhancement, current approach works fine.

**Estimated Effort:** Small (1 day)

**Recommended Priority for Future:** P4 (low priority)

---

## Issues by Functional Area

### MCP Testing UI

**Total Issues:** 0
**Status:** ⚠️ MANUAL TESTING REQUIRED

**Note:** MCP Testing UI requires full manual verification (22 test cases). No automated testing completed yet due to GUI interaction requirements. Potential issues may be identified during manual testing.

**Pending Manual Tests:**
- MCP-TB-001 through MCP-TB-008: Template Browser (8 tests)
- MCP-RE-001 through MCP-RE-008: Request Editor (8 tests)
- MCP-RV-001 through MCP-RV-006: Response Viewer (6 tests)

**If issues found during manual testing:**
- Document in manual-testing-checklist.md
- Add to this issue tracker with appropriate priority
- Determine if fixes needed or can be deferred

---

### Trail Viewer

**Total Issues:** 3 (all P4 - deferred)
**Status:** ✅ PRODUCTION READY

**Issues:**
- P4-001: "Load More" pagination not implemented (DEFERRED)
- P4-002: Tenant filter not in "Clear Filters" (DEFERRED)
- P4-003: Bulk operations not implemented (DEFERRED)

**Summary:**
All core functionality working correctly. P4 issues are enhancement opportunities, not bugs.

**Audit Result:** 100% pass rate for all implemented features
**Test Coverage:** 13 automated tests + 18 manual test cases

---

### Monitoring Page

**Total Issues:** 1 (P4 - deferred)
**Status:** ✅ PRODUCTION READY

**Issues:**
- P4-004: "Clear Filters" button not present (DEFERRED)

**Summary:**
All core functionality working correctly. P4 issue is minor UX enhancement.

**Audit Result:** 100% pass rate for all implemented features
**Test Coverage:** 11 automated tests + 14 manual test cases

---

### Request History

**Total Issues:** 1 (P0 - fixed, pending verification)
**Status:** ✅ FIXED → ⏳ PENDING VERIFICATION

**Issues:**
- P0-001: Type mismatch causing complete failure (FIXED)

**Summary:**
Critical type mismatch resolved. Automated tests passing. Manual verification required.

**Audit Result:** Broken before fix, expected to be working after fix
**Test Coverage:** 8 automated tests + 11 manual test cases

---

### Settings

**Total Issues:** 1 (P0 - fixed, pending verification)
**Status:** ✅ FIXED → ⏳ PENDING VERIFICATION

**Issues:**
- P0-002: Type mismatch causing complete failure (FIXED)

**Summary:**
Critical type mismatch resolved. Automated tests passing. Manual verification required.

**Audit Result:** Broken before fix, expected to be working after fix
**Test Coverage:** 6 automated tests + 4 manual test cases

---

### Cross-Cutting Concerns

**Total Issues:** 1 (P2 - fixed, pending verification)
**Status:** ✅ FIXED → ⏳ PENDING VERIFICATION

**Issues:**
- P2-001: State management error clearing bug (FIXED)

**Summary:**
Error persistence bug resolved. Automated tests passing. Manual verification required.

**Audit Result:** Bug fixed, improved error UX
**Test Coverage:** 32 automated tests + 20 manual test cases

---

## Manual Verification Checklist

### P0-P2 Issues to Verify

⛵Captain Qollective 💎, please verify the following fixed issues:

**P0-001: Request History Type Mismatch**
- [ ] Navigate to MCP Tester → Request History
- [ ] Verify history entries load without errors
- [ ] Test pagination ("Load More" button)
- [ ] Test search by tool name
- [ ] Test status filter (success/error)
- [ ] Test replay workflow
- [ ] **Result:** [ ] ✅ VERIFIED [ ] ❌ REGRESSION [ ] ⚠️ PARTIAL

**P0-002: Settings Type Mismatch**
- [ ] Navigate to Settings page
- [ ] Verify all settings display current values
- [ ] Change a setting and save
- [ ] Close and reopen application
- [ ] Verify setting persisted
- [ ] **Result:** [ ] ✅ VERIFIED [ ] ❌ REGRESSION [ ] ⚠️ PARTIAL

**P2-001: State Management Error Clearing**
- [ ] Send request that fails (invalid JSON or tool)
- [ ] Verify error displays in Response Viewer
- [ ] Switch to Request Editor tab
- [ ] Switch back to Response tab
- [ ] Verify error still displays
- [ ] **Result:** [ ] ✅ VERIFIED [ ] ❌ REGRESSION [ ] ⚠️ PARTIAL

---

## New Issues Found During Verification

**To be completed by ⛵Captain Qollective 💎 during manual testing**

### Template for New Issues

```markdown
### [NEW-001]: [Issue Title]

**Status:** Not Started / In Progress / Fixed / Verified / Deferred
**Priority:** P0 / P1 / P2 / P3 / P4
**Component:** [Component Name]
**Functional Area:** [Area]
**Found During:** Manual Verification - [Test Case ID]

**Description:**
[Clear description of the issue]

**Symptoms:**
- [Observable symptom 1]
- [Observable symptom 2]

**Reproduction Steps:**
1. [Step 1]
2. [Step 2]
3. [Expected vs Actual]

**Impact:**
[Description of user impact]

**Root Cause:**
[If known]

**Recommended Fix:**
[If known]

**Workaround:**
[If exists]
```

---

## Verification Sign-Off

**To be completed by ⛵Captain Qollective 💎**

**I have verified the following:**
- [ ] All P0 issues tested and verified as fixed
- [ ] All P1 issues tested (if any exist)
- [ ] All P2 issues tested and verified as fixed
- [ ] New issues documented in this tracker
- [ ] Manual testing checklist updated with results

**Verification Summary:**
- P0 Issues Verified: __ / 2
- P2 Issues Verified: __ / 1
- New Issues Found: __
- New P0-P1 Issues: __
- New P2-P3 Issues: __
- New P4 Issues: __

**Overall Verification Result:**
- [ ] ✅ ALL FIXED - No regressions, all issues resolved
- [ ] ⚠️ MOSTLY FIXED - Minor issues found (P3-P4 only)
- [ ] ❌ ISSUES REMAIN - P0-P2 regressions or new critical issues found

**Signature:** ___________________________
**Date:** ___________________________

---

**END OF ISSUE TRACKER**
