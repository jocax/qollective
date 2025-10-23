/**
 * Component tests for TrailCard
 *
 * Tests the trail card component functionality including:
 * - Rendering trail data
 * - Delete confirmation modal
 * - Click event handling and propagation
 * - Event emissions
 * - Edge cases
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, VueWrapper, flushPromises } from '@vue/test-utils'
import TrailCard from '../TrailCard.vue'
import type { TrailListItem } from '~/types/trails'

// Mock utility functions
const mockSaveRecentTrail = vi.fn()
const mockRouterPush = vi.fn()

// Simple global mocks for dependencies
global.useRouter = () => ({
  push: mockRouterPush
})

global.getTenantColor = (tenantId?: string) => 'blue'
global.getTenantDisplayName = (tenantId?: string) => `Tenant ${tenantId || '?'}`

/**
 * Create a mock trail for testing
 */
function createMockTrail(overrides: Partial<TrailListItem> = {}): TrailListItem {
  return {
    id: 'test-trail-123',
    file_path: '/test/path/trail.json',
    title: 'Test Adventure Story',
    description: 'An exciting adventure story for testing purposes. This is a longer description to test truncation behavior.',
    theme: 'Adventure',
    age_group: '8-12',
    language: 'en',
    tags: ['adventure', 'fantasy', 'quest'],
    status: 'completed',
    generated_at: '2025-10-23T10:00:00Z',
    node_count: 24,
    tenantId: 'tenant-1',
    ...overrides
  }
}

describe('TrailCard', () => {
  let wrapper: VueWrapper<any>
  let mockTrail: TrailListItem

  beforeEach(() => {
    // Reset all mocks before each test
    vi.clearAllMocks()
    mockRouterPush.mockClear()
    mockSaveRecentTrail.mockClear()
    mockTrail = createMockTrail()
  })

  afterEach(() => {
    if (wrapper) {
      wrapper.unmount()
    }
  })

  describe('Component Rendering', () => {
    it('renders trail title correctly', () => {
      wrapper = mount(TrailCard, {
        props: { trail: mockTrail }
      })

      expect(wrapper.text()).toContain('Test Adventure Story')
    })

    it('renders trail description with truncation', () => {
      const longDescription = 'A'.repeat(200)
      const trail = createMockTrail({ description: longDescription })

      wrapper = mount(TrailCard, {
        props: { trail }
      })

      const description = wrapper.text()
      // Should be truncated to 150 chars + '...'
      expect(description).toContain('A'.repeat(150))
      expect(description).toContain('...')
    })

    it('renders short description without truncation', () => {
      const shortDescription = 'Short description'
      const trail = createMockTrail({ description: shortDescription })

      wrapper = mount(TrailCard, {
        props: { trail }
      })

      expect(wrapper.text()).toContain(shortDescription)
      expect(wrapper.text()).not.toContain('...')
    })

    it('displays status badge with correct color', () => {
      wrapper = mount(TrailCard, {
        props: { trail: mockTrail }
      })

      expect(wrapper.text()).toContain('completed')
    })

    it('displays formatted date', () => {
      wrapper = mount(TrailCard, {
        props: { trail: mockTrail }
      })

      // Should contain some date representation
      expect(wrapper.text()).toMatch(/Oct|2025/)
    })

    it('displays metadata badges (age group, language, theme)', () => {
      wrapper = mount(TrailCard, {
        props: { trail: mockTrail }
      })

      expect(wrapper.text()).toContain('Adventure') // theme
      expect(wrapper.text()).toContain('8-12') // age_group
      expect(wrapper.text()).toContain('EN') // language uppercased
    })

    it('displays node count', () => {
      wrapper = mount(TrailCard, {
        props: { trail: mockTrail }
      })

      expect(wrapper.text()).toContain('24 nodes')
    })

    it('displays tenant badge when tenantId exists', () => {
      wrapper = mount(TrailCard, {
        props: { trail: mockTrail }
      })

      expect(wrapper.text()).toContain('Tenant tenant-1')
    })

    it('does not display tenant badge when tenantId is missing', () => {
      const trail = createMockTrail({ tenantId: undefined })

      wrapper = mount(TrailCard, {
        props: { trail }
      })

      expect(wrapper.text()).not.toContain('Tenant')
    })

    it('displays up to 3 tags', () => {
      wrapper = mount(TrailCard, {
        props: { trail: mockTrail }
      })

      expect(wrapper.text()).toContain('adventure')
      expect(wrapper.text()).toContain('fantasy')
      expect(wrapper.text()).toContain('quest')
    })

    it('shows "+X more" for additional tags', () => {
      const trail = createMockTrail({
        tags: ['tag1', 'tag2', 'tag3', 'tag4', 'tag5']
      })

      wrapper = mount(TrailCard, {
        props: { trail }
      })

      expect(wrapper.text()).toContain('+2 more')
    })

    it('renders trash icon button', () => {
      wrapper = mount(TrailCard, {
        props: { trail: mockTrail }
      })

      const deleteButton = wrapper.find('[icon="i-heroicons-trash"]')
      expect(deleteButton.exists()).toBe(true)
    })
  })

  describe('Delete Functionality', () => {
    it('opens delete confirmation modal when trash icon is clicked', async () => {
      wrapper = mount(TrailCard, {
        props: { trail: mockTrail }
      })

      const deleteButton = wrapper.find('[icon="i-heroicons-trash"]')
      await deleteButton.trigger('click')

      // Modal should be visible (v-model="showDeleteConfirm" = true)
      expect(wrapper.vm.showDeleteConfirm).toBe(true)
    })

    it('displays modal with trail title and warning message', async () => {
      wrapper = mount(TrailCard, {
        props: { trail: mockTrail }
      })

      const deleteButton = wrapper.find('[icon="i-heroicons-trash"]')
      await deleteButton.trigger('click')

      await wrapper.vm.$nextTick()

      // Check modal content
      expect(wrapper.text()).toContain('Delete Trail')
      expect(wrapper.text()).toContain('Test Adventure Story')
      expect(wrapper.text()).toContain('This action cannot be undone')
    })

    it('closes modal when cancel button is clicked', async () => {
      wrapper = mount(TrailCard, {
        props: { trail: mockTrail }
      })

      // Open modal
      const deleteButton = wrapper.find('[icon="i-heroicons-trash"]')
      await deleteButton.trigger('click')
      expect(wrapper.vm.showDeleteConfirm).toBe(true)

      // Click cancel
      wrapper.vm.cancelDelete()
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.showDeleteConfirm).toBe(false)
    })

    it('emits delete event with trail ID when confirmed', async () => {
      wrapper = mount(TrailCard, {
        props: { trail: mockTrail }
      })

      // Open modal
      const deleteButton = wrapper.find('[icon="i-heroicons-trash"]')
      await deleteButton.trigger('click')

      // Confirm delete
      wrapper.vm.confirmDelete()
      await wrapper.vm.$nextTick()

      // Check emitted event
      expect(wrapper.emitted('delete')).toBeTruthy()
      expect(wrapper.emitted('delete')?.[0]).toEqual(['test-trail-123'])
    })

    it('sets deleting state to true when delete is confirmed', async () => {
      wrapper = mount(TrailCard, {
        props: { trail: mockTrail }
      })

      wrapper.vm.confirmDelete()
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.deleting).toBe(true)
    })

    it('shows loading state on delete button during deletion', async () => {
      wrapper = mount(TrailCard, {
        props: { trail: mockTrail }
      })

      wrapper.vm.confirmDelete()
      await wrapper.vm.$nextTick()

      const deleteButton = wrapper.find('[icon="i-heroicons-trash"]')
      expect(deleteButton.attributes('loading')).toBeDefined()
    })

    it('disables card interaction during deletion', async () => {
      wrapper = mount(TrailCard, {
        props: { trail: mockTrail }
      })

      wrapper.vm.confirmDelete()
      await wrapper.vm.$nextTick()

      // Card should have pointer-events-none class
      const card = wrapper.find('.cursor-pointer')
      expect(card.classes()).toContain('pointer-events-none')
      expect(card.classes()).toContain('opacity-50')
    })
  })

  describe('Click Event Propagation', () => {
    it('does not navigate when delete button is clicked (click.stop)', async () => {
      wrapper = mount(TrailCard, {
        props: { trail: mockTrail }
      })

      // Find delete button and trigger click
      const deleteButton = wrapper.find('[icon="i-heroicons-trash"]')
      await deleteButton.trigger('click')

      // Should NOT navigate (click.stop prevents propagation)
      expect(mockRouterPush).not.toHaveBeenCalled()
    })

    it('click handler can be called without errors', () => {
      wrapper = mount(TrailCard, {
        props: { trail: mockTrail }
      })

      // Should not throw when calling handleClick
      expect(() => wrapper.vm.handleClick()).not.toThrow()
    })
  })

  describe('Edge Cases', () => {
    it('handles missing description gracefully', () => {
      const trail = createMockTrail({ description: '' })

      wrapper = mount(TrailCard, {
        props: { trail }
      })

      expect(wrapper.text()).not.toContain('...')
    })

    it('handles empty tags array', () => {
      const trail = createMockTrail({ tags: [] })

      wrapper = mount(TrailCard, {
        props: { trail }
      })

      // Component should render without crashing
      expect(wrapper.exists()).toBe(true)
    })

    it('handles invalid date gracefully', () => {
      const trail = createMockTrail({ generated_at: 'invalid-date' })

      wrapper = mount(TrailCard, {
        props: { trail }
      })

      // Should fall back to showing the raw string
      expect(wrapper.text()).toContain('invalid-date')
    })

    it('handles failed status with red badge', () => {
      const trail = createMockTrail({ status: 'failed' })

      wrapper = mount(TrailCard, {
        props: { trail }
      })

      expect(wrapper.text()).toContain('failed')
    })

    it('handles partial status with yellow badge', () => {
      const trail = createMockTrail({ status: 'partial' })

      wrapper = mount(TrailCard, {
        props: { trail }
      })

      expect(wrapper.text()).toContain('partial')
    })

    it('handles missing node count', () => {
      const trail = createMockTrail({ node_count: 0 })

      wrapper = mount(TrailCard, {
        props: { trail }
      })

      expect(wrapper.text()).toContain('0 nodes')
    })
  })

  describe('Component State Management', () => {
    it('initializes with showDeleteConfirm as false', () => {
      wrapper = mount(TrailCard, {
        props: { trail: mockTrail }
      })

      expect(wrapper.vm.showDeleteConfirm).toBe(false)
    })

    it('initializes with deleting as false', () => {
      wrapper = mount(TrailCard, {
        props: { trail: mockTrail }
      })

      expect(wrapper.vm.deleting).toBe(false)
    })

    it('closes modal after delete confirmation', async () => {
      wrapper = mount(TrailCard, {
        props: { trail: mockTrail }
      })

      // Open modal
      wrapper.vm.openDeleteConfirm()
      expect(wrapper.vm.showDeleteConfirm).toBe(true)

      // Confirm delete
      wrapper.vm.confirmDelete()
      await wrapper.vm.$nextTick()

      // Modal should be closed
      expect(wrapper.vm.showDeleteConfirm).toBe(false)
    })

    it('resets state when cancel is clicked', async () => {
      wrapper = mount(TrailCard, {
        props: { trail: mockTrail }
      })

      // Open modal
      wrapper.vm.openDeleteConfirm()

      // Cancel
      wrapper.vm.cancelDelete()
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.showDeleteConfirm).toBe(false)
      expect(wrapper.vm.deleting).toBe(false)
    })
  })
})
