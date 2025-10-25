/**
 * Integration tests for TrailCard using real test data
 *
 * Tests the trail card component functionality with actual generated story data
 * from response_test_epic_2.json:
 * - Title: "Heimische Flora und Fauna im mediteranen Raum"
 * - Nodes: 24
 * - Language: DE
 * - Age Group: 15-17
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { mountSuspended } from '@nuxt/test-utils/runtime'
import type { VueWrapper } from '@vue/test-utils'
import TrailCard from '../TrailCard.vue'
import { loadTestTrailListItem } from '~/utils/__tests__/fixtures/testDataLoader'
import type { TrailListItem } from '~/types/trails'

// Mock dependencies
const mockRouterPush = vi.fn()

vi.mock('#app', () => ({
  useRouter: () => ({
    push: mockRouterPush
  })
}))

vi.mock('~/utils/trailStorage', () => ({
  saveRecentTrail: vi.fn()
}))

vi.mock('~/utils/tenantColors', () => ({
  getTenantColor: (tenantId?: string) => 'blue',
  getTenantDisplayName: (tenantId?: string) => `Tenant ${tenantId || '?'}`
}))

describe('TrailCard - Integration Tests with Real Data', () => {
  let wrapper: VueWrapper<any>
  let testTrail: TrailListItem

  beforeEach(() => {
    // Load real test trail data
    testTrail = loadTestTrailListItem()
    vi.clearAllMocks()
  })

  afterEach(() => {
    // mountSuspended handles cleanup automatically
  })

  describe('Real Data Rendering', () => {
    it('renders actual trail title from test data', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      expect(wrapper.text()).toContain('Heimische Flora und Fauna im mediteranen Raum')
    })

    it('renders correct node count from test data (24 nodes)', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      expect(wrapper.text()).toContain('24 nodes')
    })

    it('displays correct language badge (DE)', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      expect(wrapper.text()).toContain('DE')
    })

    it('displays correct age group (15-17)', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      expect(wrapper.text()).toContain('15-17')
    })

    it('displays correct theme from test data', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      expect(wrapper.text()).toContain('Heimische Flora und Fauna im mediteranen Raum')
    })

    it('renders actual generated_at timestamp', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      // Should format the real timestamp from test data
      const text = wrapper.text()
      expect(text).toMatch(/Oct|2025/)
    })

    it('displays completed status badge', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      expect(wrapper.text()).toContain('completed')
    })

    it('renders real description text', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      expect(wrapper.text()).toContain('Interactive story for 15-17 age group')
    })

    it('displays tenant badge with tenant-1', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      expect(wrapper.text()).toContain('Tenant tenant-1')
    })

    it('renders all test tags (first 3)', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      const text = wrapper.text()
      expect(text).toContain('nature')
      expect(text).toContain('mediterranean')
      expect(text).toContain('flora')
    })

    it('shows remaining tags count (+2 more)', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      // Test trail has 5 tags, should show 3 + "+2 more"
      expect(wrapper.text()).toContain('+2 more')
    })
  })

  describe('Description Truncation', () => {
    it('displays description without truncation when short', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      const description = testTrail.description
      expect(wrapper.text()).toContain(description)
      // Short description should not have ellipsis
      expect(wrapper.vm.truncatedDescription).toBe(description)
    })

    it('truncates description at 150 characters when long', async () => {
      const longDescription = 'A'.repeat(200)
      const modifiedTrail = { ...testTrail, description: longDescription }

      wrapper = await mountSuspended(TrailCard, {
        props: { trail: modifiedTrail }
      })

      expect(wrapper.vm.truncatedDescription).toBe('A'.repeat(150) + '...')
      expect(wrapper.vm.truncatedDescription.length).toBe(153)
    })
  })

  describe('Computed Properties', () => {
    it('computes statusColor correctly for completed', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      expect(wrapper.vm.statusColor).toBe('green')
    })

    it('computes tenantColor for tenant-1', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      expect(wrapper.vm.tenantColor).toBe('blue')
    })

    it('computes tenantDisplay for tenant-1', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      expect(wrapper.vm.tenantDisplay).toBe('Tenant tenant-1')
    })

    it('formats date correctly from real timestamp', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      // Real timestamp: 2025-10-23T10:01:40.151329+00:00
      const formatted = wrapper.vm.formattedDate
      expect(formatted).toContain('Oct')
      expect(formatted).toContain('2025')
    })

    it('computes displayTags correctly (first 3 tags)', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      expect(wrapper.vm.displayTags).toEqual(['nature', 'mediterranean', 'flora'])
      expect(wrapper.vm.displayTags.length).toBe(3)
    })

    it('computes remainingTagsCount correctly (2 remaining)', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      // Test trail has 5 tags, so 5 - 3 = 2 remaining
      expect(wrapper.vm.remainingTagsCount).toBe(2)
    })
  })

  describe('Delete Functionality', () => {
    it('initializes with showDeleteConfirm as false', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      expect(wrapper.vm.showDeleteConfirm).toBe(false)
    })

    it('initializes with deleting as false', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      expect(wrapper.vm.deleting).toBe(false)
    })

    it('opens delete confirmation modal', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      wrapper.vm.openDeleteConfirm()
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.showDeleteConfirm).toBe(true)
    })

    it('closes modal on cancel', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      wrapper.vm.openDeleteConfirm()
      expect(wrapper.vm.showDeleteConfirm).toBe(true)

      wrapper.vm.cancelDelete()
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.showDeleteConfirm).toBe(false)
    })

    it('emits delete event with trail ID on confirmation', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      wrapper.vm.confirmDelete()
      await wrapper.vm.$nextTick()

      expect(wrapper.emitted('delete')).toBeTruthy()
      expect(wrapper.emitted('delete')?.[0]).toEqual(['test-trail-epic-2'])
    })

    it('sets deleting state and closes modal on confirmation', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      wrapper.vm.openDeleteConfirm()
      expect(wrapper.vm.showDeleteConfirm).toBe(true)

      wrapper.vm.confirmDelete()
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.deleting).toBe(true)
      expect(wrapper.vm.showDeleteConfirm).toBe(false)
    })

    it('displays delete confirmation with trail title', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      wrapper.vm.openDeleteConfirm()
      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('Delete Trail')
      expect(wrapper.text()).toContain('Heimische Flora und Fauna im mediteranen Raum')
      expect(wrapper.text()).toContain('This action cannot be undone')
    })
  })

  describe('Click Navigation', () => {
    it('calls handleClick method', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      expect(() => wrapper.vm.handleClick()).not.toThrow()
    })

    it('navigates to viewer with correct trail ID', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      wrapper.vm.handleClick()

      expect(mockRouterPush).toHaveBeenCalledWith('/viewer/test-trail-epic-2')
    })
  })

  describe('Visual State Changes', () => {
    it('applies disabled state when deleting', async () => {
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: testTrail }
      })

      wrapper.vm.confirmDelete()
      await wrapper.vm.$nextTick()

      // Check that deleting state affects component
      expect(wrapper.vm.deleting).toBe(true)
    })
  })

  describe('Edge Cases with Real Data', () => {
    it('handles empty tags array', async () => {
      const trailNoTags = { ...testTrail, tags: [] }
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: trailNoTags }
      })

      expect(wrapper.vm.displayTags).toEqual([])
      expect(wrapper.vm.remainingTagsCount).toBe(0)
    })

    it('handles failed status', async () => {
      const failedTrail = { ...testTrail, status: 'failed' }
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: failedTrail }
      })

      expect(wrapper.vm.statusColor).toBe('red')
      expect(wrapper.text()).toContain('failed')
    })

    it('handles partial status', async () => {
      const partialTrail = { ...testTrail, status: 'partial' }
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: partialTrail }
      })

      expect(wrapper.vm.statusColor).toBe('yellow')
      expect(wrapper.text()).toContain('partial')
    })

    it('handles missing tenantId', async () => {
      const noTenantTrail = { ...testTrail, tenantId: undefined }
      wrapper = await mountSuspended(TrailCard, {
        props: { trail: noTenantTrail }
      })

      // Should not crash and tenant badge should not render
      expect(wrapper.exists()).toBe(true)
    })
  })
})
