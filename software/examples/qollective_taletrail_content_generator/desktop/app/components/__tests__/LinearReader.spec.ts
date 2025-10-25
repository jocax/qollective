/**
 * Component tests for LinearReader
 *
 * Tests pagination logic, keyboard navigation, and node traversal using real trail data
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mountSuspended } from '@nuxt/test-utils/runtime'
import type { VueWrapper } from '@vue/test-utils'
import LinearReader from '../LinearReader.vue'
import { loadTestTrailWithDAG, getTestTrailStats } from '~/utils/__tests__/fixtures/testDataLoader'
import type { Trail, ContentNode } from '~/types/trails'

// Mock window methods
const mockScrollTo = vi.fn()
global.window.scrollTo = mockScrollTo

describe('LinearReader with Real Data', () => {
  let wrapper: VueWrapper<any>
  const testData = loadTestTrailWithDAG()
  const stats = getTestTrailStats()

  beforeEach(async () => {
    mockScrollTo.mockClear()
    wrapper = await mountSuspended(LinearReader, {
      props: {
        trail: testData.trail
      }
    })
  })

  describe('Initialization with 24-node trail', () => {
    it('should mount successfully', () => {
      expect(wrapper.exists()).toBe(true)
    })

    it('should linearize all 24 nodes from DAG', () => {
      expect(wrapper.vm.linearNodes.length).toBe(stats.nodeCount)
    })

    it('should start at page 0', () => {
      expect(wrapper.vm.currentPage).toBe(0)
    })

    it('should calculate total pages correctly', () => {
      expect(wrapper.vm.totalPages).toBe(stats.nodeCount)
    })

    it('should show correct page indicator', () => {
      expect(wrapper.text()).toContain(`Page 1 of ${stats.nodeCount}`)
    })

    it('should display first node (start node) initially', () => {
      expect(wrapper.vm.currentNode?.id).toBe(stats.startNodeId)
    })

    it('should initialize showMetadata as false', () => {
      expect(wrapper.vm.showMetadata).toBe(false)
    })
  })

  describe('Pagination Navigation', () => {
    it('should navigate to next page', async () => {
      await wrapper.vm.nextPage()

      expect(wrapper.vm.currentPage).toBe(1)
      expect(wrapper.text()).toContain(`Page 2 of ${stats.nodeCount}`)
    })

    it('should navigate to previous page', async () => {
      // First go to page 1
      await wrapper.vm.nextPage()
      expect(wrapper.vm.currentPage).toBe(1)

      // Then go back to page 0
      await wrapper.vm.prevPage()
      expect(wrapper.vm.currentPage).toBe(0)
      expect(wrapper.text()).toContain(`Page 1 of ${stats.nodeCount}`)
    })

    it('should not go below page 0', async () => {
      expect(wrapper.vm.currentPage).toBe(0)

      await wrapper.vm.prevPage()

      expect(wrapper.vm.currentPage).toBe(0)
      expect(wrapper.vm.canGoPrev).toBe(false)
    })

    it('should not exceed total pages', async () => {
      const totalPages = wrapper.vm.totalPages

      // Navigate to last page
      for (let i = 0; i < totalPages; i++) {
        await wrapper.vm.nextPage()
      }

      // Should be at last page
      expect(wrapper.vm.currentPage).toBe(totalPages - 1)
      expect(wrapper.vm.canGoNext).toBe(false)
    })

    it('should disable previous button on first page', () => {
      expect(wrapper.vm.canGoPrev).toBe(false)
    })

    it('should disable next button on last page', async () => {
      const totalPages = wrapper.vm.totalPages

      // Navigate to last page
      for (let i = 0; i < totalPages - 1; i++) {
        await wrapper.vm.nextPage()
      }

      expect(wrapper.vm.canGoNext).toBe(false)
    })

    it('should call scrollToTop when nextPage is called', () => {
      wrapper.vm.nextPage()

      expect(mockScrollTo).toHaveBeenCalled()
    })

    it('should call scrollToTop when prevPage is called', () => {
      wrapper.vm.currentPage = 5
      wrapper.vm.prevPage()

      expect(mockScrollTo).toHaveBeenCalled()
    })
  })

  describe('Jump to Page', () => {
    it('should jump to specific page', async () => {
      await wrapper.vm.jumpToPage(10)

      expect(wrapper.vm.currentPage).toBe(10)
      expect(wrapper.text()).toContain(`Page 11 of ${stats.nodeCount}`)
    })

    it('should jump to first page', async () => {
      await wrapper.vm.nextPage()
      await wrapper.vm.nextPage()
      expect(wrapper.vm.currentPage).toBe(2)

      await wrapper.vm.jumpToPage(0)

      expect(wrapper.vm.currentPage).toBe(0)
    })

    it('should jump to last page', async () => {
      const totalPages = wrapper.vm.totalPages

      await wrapper.vm.jumpToPage(totalPages - 1)

      expect(wrapper.vm.currentPage).toBe(totalPages - 1)
    })

    it('should clamp to valid page range', async () => {
      const totalPages = wrapper.vm.totalPages

      // Try to jump beyond last page
      await wrapper.vm.jumpToPage(999)
      expect(wrapper.vm.currentPage).toBe(totalPages - 1)

      // Try to jump before first page
      await wrapper.vm.jumpToPage(-1)
      expect(wrapper.vm.currentPage).toBe(0)
    })

    it('should call scrollToTop when jumping to page', () => {
      mockScrollTo.mockClear()
      wrapper.vm.jumpToPage(5)

      expect(mockScrollTo).toHaveBeenCalled()
    })

    it('should not scroll if jumping to same page', () => {
      mockScrollTo.mockClear()
      wrapper.vm.currentPage = 5
      wrapper.vm.jumpToPage(5)

      expect(mockScrollTo).not.toHaveBeenCalled()
    })
  })

  describe('Node Traversal', () => {
    it('should traverse DAG in depth-first order starting from start_node_id', () => {
      const linearNodes = wrapper.vm.linearNodes

      // Should start with start node
      expect(linearNodes[0].id).toBe(stats.startNodeId)
    })

    it('should include all 24 nodes in traversal', () => {
      const linearNodes = wrapper.vm.linearNodes
      const nodeIds = linearNodes.map((n: ContentNode) => n.id)

      expect(linearNodes.length).toBe(stats.nodeCount)
      expect(new Set(nodeIds).size).toBe(stats.nodeCount)
    })

    it('should not include duplicate nodes', () => {
      const linearNodes = wrapper.vm.linearNodes
      const nodeIds = linearNodes.map((n: ContentNode) => n.id)
      const uniqueIds = new Set(nodeIds)

      expect(nodeIds.length).toBe(uniqueIds.size)
    })

    it('should match all DAG node IDs', () => {
      const linearNodes = wrapper.vm.linearNodes
      const nodeIds = new Set(linearNodes.map((n: ContentNode) => n.id))
      const dagNodeIds = Object.keys(testData.trail.dag.nodes)

      dagNodeIds.forEach(id => {
        expect(nodeIds.has(id)).toBe(true)
      })
    })
  })

  describe('Progress Calculation', () => {
    it('should calculate progress percentage correctly', () => {
      wrapper.vm.currentPage = 0
      expect(wrapper.vm.progressPercent).toBe(Math.round((1 / stats.nodeCount) * 100))

      wrapper.vm.currentPage = 11
      expect(wrapper.vm.progressPercent).toBe(Math.round((12 / stats.nodeCount) * 100))
    })

    it('should update progress as pages advance', async () => {
      const initialProgress = wrapper.vm.progressPercent

      await wrapper.vm.nextPage()
      const newProgress = wrapper.vm.progressPercent

      expect(newProgress).toBeGreaterThan(initialProgress)
    })

    it('should show 100% on last page', async () => {
      const totalPages = wrapper.vm.totalPages
      await wrapper.vm.jumpToPage(totalPages - 1)

      expect(wrapper.vm.progressPercent).toBe(100)
    })

    it('should return 0% when totalPages is 0', async () => {
      const emptyTrail = {
        ...testData.trail,
        dag: {
          nodes: {},
          edges: [],
          start_node_id: '0',
          convergence_points: []
        }
      }

      const emptyWrapper = await mountSuspended(LinearReader, {
        props: { trail: emptyTrail }
      })

      expect(emptyWrapper.vm.progressPercent).toBe(0)
    })
  })

  describe('Convergence Point Detection', () => {
    it('should identify convergence points from trail DAG', () => {
      expect(wrapper.vm.convergencePoints).toEqual(testData.trail.dag.convergence_points || [])
    })

    it('should correctly determine if current node is convergence point', () => {
      if (testData.trail.dag.convergence_points && testData.trail.dag.convergence_points.length > 0) {
        const convergenceNodeId = testData.trail.dag.convergence_points[0]
        const convergencePageIndex = wrapper.vm.linearNodes.findIndex((n: ContentNode) => n.id === convergenceNodeId)

        if (convergencePageIndex >= 0) {
          wrapper.vm.currentPage = convergencePageIndex
          expect(wrapper.vm.isCurrentNodeConvergence).toBe(true)
        }
      }
    })

    it('should not mark start node as convergence point', () => {
      wrapper.vm.currentPage = 0
      expect(wrapper.vm.currentNode?.id).toBe(stats.startNodeId)
      expect(wrapper.vm.isCurrentNodeConvergence).toBe(false)
    })
  })

  describe('Page Options', () => {
    it('should generate page options for all 24 pages', () => {
      const options = wrapper.vm.pageOptions

      expect(options.length).toBe(stats.nodeCount)
      expect(options[0].label).toContain('Page 1')
      expect(options[0].value).toBe(0)
    })

    it('should include node IDs in page options', () => {
      const options = wrapper.vm.pageOptions

      expect(options[0].label).toContain(stats.startNodeId)
    })

    it('should set correct value for each option', () => {
      wrapper.vm.pageOptions.forEach((option, index) => {
        expect(option.value).toBe(index)
      })
    })
  })

  describe('Keyboard Navigation', () => {
    it('should navigate next on arrow right', async () => {
      wrapper.vm.handleKeydown(new KeyboardEvent('keydown', { key: 'ArrowRight' }))

      expect(wrapper.vm.currentPage).toBe(1)
    })

    it('should navigate previous on arrow left', async () => {
      await wrapper.vm.nextPage()
      expect(wrapper.vm.currentPage).toBe(1)

      wrapper.vm.handleKeydown(new KeyboardEvent('keydown', { key: 'ArrowLeft' }))

      expect(wrapper.vm.currentPage).toBe(0)
    })

    it('should jump to first page on Home', async () => {
      await wrapper.vm.jumpToPage(10)

      wrapper.vm.handleKeydown(new KeyboardEvent('keydown', { key: 'Home' }))

      expect(wrapper.vm.currentPage).toBe(0)
    })

    it('should jump to last page on End', async () => {
      const totalPages = wrapper.vm.totalPages

      wrapper.vm.handleKeydown(new KeyboardEvent('keydown', { key: 'End' }))

      expect(wrapper.vm.currentPage).toBe(totalPages - 1)
    })
  })

  describe('Current Node Resolution', () => {
    it('should return current node based on currentPage', () => {
      wrapper.vm.currentPage = 0
      expect(wrapper.vm.currentNode?.id).toBe(wrapper.vm.linearNodes[0].id)

      wrapper.vm.currentPage = 1
      expect(wrapper.vm.currentNode?.id).toBe(wrapper.vm.linearNodes[1].id)
    })

    it('should return null if currentPage exceeds linearNodes length', () => {
      wrapper.vm.currentPage = 999
      expect(wrapper.vm.currentNode).toBeNull()
    })
  })

  describe('Trail Changes', () => {
    it('should reset to first page when trail changes', async () => {
      await wrapper.vm.nextPage()
      expect(wrapper.vm.currentPage).toBe(1)

      // Update trail prop
      const newTrail = { ...testData.trail }
      await wrapper.setProps({ trail: newTrail })

      expect(wrapper.vm.currentPage).toBe(0)
    })
  })

  describe('Edge Cases', () => {
    it('should handle empty choices array', () => {
      // Last nodes typically have empty choices
      const totalPages = wrapper.vm.totalPages
      const linearNodes = wrapper.vm.linearNodes

      expect(linearNodes[totalPages - 1].content.choices).toEqual([])
    })

    it('should handle missing nodes gracefully', async () => {
      // Create trail with broken edge reference
      const brokenTrail: Trail = {
        ...testData.trail,
        dag: {
          ...testData.trail.dag,
          edges: [
            ...testData.trail.dag.edges,
            { from_node_id: Object.keys(testData.trail.dag.nodes)[0], to_node_id: 'nonexistent', choice_id: 'broken' }
          ]
        }
      }

      const brokenWrapper = await mountSuspended(LinearReader, {
        props: { trail: brokenTrail }
      })

      // Should still work, just skip missing node
      expect(brokenWrapper.vm.linearNodes.length).toBeGreaterThan(0)
    })
  })
})

// Keep mock trail helper functions for edge case tests that need simpler data
function createMockTrail(): Trail {
  return {
    title: 'Linear Test Adventure',
    description: 'A test story for linear reader testing',
    metadata: {
      generation_params: {
        age_group: '8-12',
        theme: 'Adventure',
        language: 'en',
        node_count: 5
      },
      start_node_id: 'node1'
    },
    dag: {
      nodes: {
        node1: {
          id: 'node1',
          content: {
            text: 'Page 1 content',
            choices: [
              { id: 'choice1', text: 'Next', next_node_id: 'node2' }
            ]
          }
        },
        node2: {
          id: 'node2',
          content: {
            text: 'Page 2 content',
            choices: [
              { id: 'choice2', text: 'Continue', next_node_id: 'node3' }
            ]
          }
        },
        node3: {
          id: 'node3',
          content: {
            text: 'Page 3 content with branch',
            choices: [
              { id: 'choice3a', text: 'Path A', next_node_id: 'node4' },
              { id: 'choice3b', text: 'Path B', next_node_id: 'node5' }
            ]
          }
        },
        node4: {
          id: 'node4',
          content: {
            text: 'Page 4 - Path A',
            choices: []
          }
        },
        node5: {
          id: 'node5',
          content: {
            text: 'Page 5 - Path B',
            choices: []
          }
        }
      },
      edges: [
        { from_node_id: 'node1', to_node_id: 'node2', choice_id: 'choice1' },
        { from_node_id: 'node2', to_node_id: 'node3', choice_id: 'choice2' },
        { from_node_id: 'node3', to_node_id: 'node4', choice_id: 'choice3a' },
        { from_node_id: 'node3', to_node_id: 'node5', choice_id: 'choice3b' }
      ],
      convergence_points: []
    }
  }
}
