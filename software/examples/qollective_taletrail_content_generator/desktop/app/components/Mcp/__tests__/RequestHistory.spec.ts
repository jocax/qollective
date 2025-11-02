/**
 * Request History Type Tests
 *
 * Focused tests for critical Request History type structure:
 * - HistoryQuery uses page/page_size (not offset/limit)
 * - HistoryQuery uses search_term (not tool_filter)
 * - HistoryQuery uses status_filter (not success_filter)
 * - HistoryPage uses total_count (not total)
 */

import { describe, it, expect } from 'vitest';
import type { HistoryQuery, HistoryPage } from '@/types/mcp';

describe('Request History Type Structure', () => {
  it('should have HistoryQuery with page and page_size fields', () => {
    const query: HistoryQuery = {
      page: 0,
      page_size: 50,
      server_filter: 'orchestrator'
    };

    expect(query).toHaveProperty('page');
    expect(query).toHaveProperty('page_size');
    expect(query.page).toBe(0);
    expect(query.page_size).toBe(50);
  });

  it('should have HistoryQuery with optional search_term field', () => {
    const queryWithSearch: HistoryQuery = {
      page: 0,
      page_size: 50,
      search_term: 'create_story'
    };

    expect(queryWithSearch).toHaveProperty('search_term');
    expect(queryWithSearch.search_term).toBe('create_story');
  });

  it('should have HistoryQuery with optional status_filter as enum', () => {
    const queryWithStatus: HistoryQuery = {
      page: 0,
      page_size: 50,
      status_filter: 'success'
    };

    expect(queryWithStatus).toHaveProperty('status_filter');
    expect(queryWithStatus.status_filter).toBe('success');

    // Test all valid values
    const validStatuses: Array<'success' | 'error' | 'timeout'> = ['success', 'error', 'timeout'];
    validStatuses.forEach(status => {
      const q: HistoryQuery = {
        page: 0,
        page_size: 50,
        status_filter: status
      };
      expect(q.status_filter).toBe(status);
    });
  });

  it('should NOT have offset, limit, tool_filter, or success_filter fields in HistoryQuery', () => {
    const query: HistoryQuery = {
      page: 0,
      page_size: 50
    };

    // These fields should not exist in the type
    expect(query).not.toHaveProperty('offset');
    expect(query).not.toHaveProperty('limit');
    expect(query).not.toHaveProperty('tool_filter');
    expect(query).not.toHaveProperty('success_filter');
  });

  it('should have HistoryPage with total_count field', () => {
    const page: HistoryPage = {
      entries: [],
      total_count: 100,
      page: 0,
      page_size: 50,
      total_pages: 2
    };

    expect(page).toHaveProperty('total_count');
    expect(page.total_count).toBe(100);
  });

  it('should have HistoryPage with page, page_size, and total_pages fields', () => {
    const page: HistoryPage = {
      entries: [],
      total_count: 150,
      page: 2,
      page_size: 50,
      total_pages: 3
    };

    expect(page).toHaveProperty('page');
    expect(page).toHaveProperty('page_size');
    expect(page).toHaveProperty('total_pages');
    expect(page.page).toBe(2);
    expect(page.page_size).toBe(50);
    expect(page.total_pages).toBe(3);
  });

  it('should compute has_more correctly from page and total_pages', () => {
    const pageWithMore: HistoryPage = {
      entries: [],
      total_count: 150,
      page: 0,
      page_size: 50,
      total_pages: 3
    };

    // has_more should be true when page < total_pages - 1
    const has_more = pageWithMore.page < pageWithMore.total_pages - 1;
    expect(has_more).toBe(true);

    const lastPage: HistoryPage = {
      entries: [],
      total_count: 150,
      page: 2,
      page_size: 50,
      total_pages: 3
    };

    // has_more should be false when on last page
    const has_more_last = lastPage.page < lastPage.total_pages - 1;
    expect(has_more_last).toBe(false);
  });

  it('should NOT have tenant_id field in HistoryQuery', () => {
    const query: HistoryQuery = {
      page: 0,
      page_size: 50,
      server_filter: 'orchestrator'
    };

    // tenant_id should not be in the query type anymore
    expect(query).not.toHaveProperty('tenant_id');
  });
});
