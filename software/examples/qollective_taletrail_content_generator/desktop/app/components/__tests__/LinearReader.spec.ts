/**
 * Component tests for LinearReader
 *
 * Tests pagination logic, keyboard navigation, and node traversal
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, VueWrapper } from '@vue/test-utils'
import LinearReader from '../LinearReader.vue'
import type { Trail, ContentNode } from '~/types/trails'

/**
 * Mock trail data for testing
 */
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

/**
 * Create trail with convergence point
 */
function createMockTrailWithConvergence(): Trail {
  return {
    title: 'Convergence Test',
    description: 'Test trail with convergence points',
    metadata: {
      generation_params: {
        age_group: '8-12',
        theme: 'Adventure',
        language: 'en',
        node_count: 4
      },
      start_node_id: 'node1'
    },
    dag: {
      nodes: {
        node1: {
          id: 'node1',
          content: {
            text: 'Start',
            choices: [
              { id: 'choice1', text: 'Left', next_node_id: 'node2' },
              { id: 'choice2', text: 'Right', next_node_id: 'node3' }
            ]
          }
        },
        node2: {
          id: 'node2',
          content: {
            text: 'Left path',
            choices: [
              { id: 'choice3', text: 'Continue', next_node_id: 'node4' }
            ]
          }
        },
        node3: {
          id: 'node3',
          content: {
            text: 'Right path',
            choices: [
              { id: 'choice4', text: 'Continue', next_node_id: 'node4' }
            ]
          }
        },
        node4: {
          id: 'node4',
          content: {
            text: 'Convergence point',
            choices: []
          }
        }
      },
      edges: [
        { from_node_id: 'node1', to_node_id: 'node2', choice_id: 'choice1' },
        { from_node_id: 'node1', to_node_id: 'node3', choice_id: 'choice2' },
        { from_node_id: 'node2', to_node_id: 'node4', choice_id: 'choice3' },
        { from_node_id: 'node3', to_node_id: 'node4', choice_id: 'choice4' }
      ],
      convergence_points: ['node4']
    }
  }
}

describe('LinearReader', () => {
  let wrapper: VueWrapper<any>
  let mockTrail: Trail

  beforeEach(() => {
    mockTrail = createMockTrail()
    wrapper = mount(LinearReader, {
      props: {
        trail: mockTrail
      }
    })
  })

  describe('Initialization', () => {
    it('should mount successfully', () => {
      expect(wrapper.exists()).toBe(true)
    })

    it('should start at page 0', () => {
      expect(wrapper.vm.currentPage).toBe(0)
    })

    it('should display first node content', () => {
      expect(wrapper.text()).toContain('Page 1 content')
    })

    it('should calculate total pages correctly', () => {
      // Should traverse all nodes in DAG (5 nodes total)
      expect(wrapper.vm.totalPages).toBe(5)
    })

    it('should show correct page indicator', () => {
      expect(wrapper.text()).toContain('Page 1 of 5')
    })
  })

  describe('Pagination Navigation', () => {
    it('should navigate to next page', async () => {
      const nextButton = wrapper.find('button:has(.i-heroicons-arrow-right)')

      await wrapper.vm.nextPage()

      expect(wrapper.vm.currentPage).toBe(1)
      expect(wrapper.text()).toContain('Page 2 of 5')
    })

    it('should navigate to previous page', async () => {
      // First go to page 1
      await wrapper.vm.nextPage()
      expect(wrapper.vm.currentPage).toBe(1)

      // Then go back to page 0
      await wrapper.vm.prevPage()
      expect(wrapper.vm.currentPage).toBe(0)
      expect(wrapper.text()).toContain('Page 1 of 5')
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
  })

  describe('Jump to Page', () => {
    it('should jump to specific page', async () => {
      await wrapper.vm.jumpToPage(2)

      expect(wrapper.vm.currentPage).toBe(2)
      expect(wrapper.text()).toContain('Page 3 of 5')
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
  })

  describe('Node Traversal', () => {
    it('should traverse DAG in depth-first order', () => {
      const linearNodes = wrapper.vm.linearNodes

      // Should start with node1
      expect(linearNodes[0].id).toBe('node1')

      // Should follow first edge to node2
      expect(linearNodes[1].id).toBe('node2')

      // Should continue to node3
      expect(linearNodes[2].id).toBe('node3')
    })

    it('should include all nodes in traversal', () => {
      const linearNodes = wrapper.vm.linearNodes
      const nodeIds = linearNodes.map((n: ContentNode) => n.id)

      // All 5 nodes should be included
      expect(nodeIds).toContain('node1')
      expect(nodeIds).toContain('node2')
      expect(nodeIds).toContain('node3')
      expect(nodeIds).toContain('node4')
      expect(nodeIds).toContain('node5')
      expect(linearNodes.length).toBe(5)
    })

    it('should handle convergence points correctly', async () => {
      const convergenceTrail = createMockTrailWithConvergence()
      const convergenceWrapper = mount(LinearReader, {
        props: {
          trail: convergenceTrail
        }
      })

      const linearNodes = convergenceWrapper.vm.linearNodes
      const nodeIds = linearNodes.map((n: ContentNode) => n.id)

      // Should include convergence node only once
      const node4Count = nodeIds.filter((id: string) => id === 'node4').length
      expect(node4Count).toBe(1)

      // Should include all unique nodes
      expect(new Set(nodeIds).size).toBe(4)
    })
  })

  describe('Progress Calculation', () => {
    it('should calculate progress percentage', () => {
      expect(wrapper.vm.progressPercent).toBe(20) // 1/5 = 20%
    })

    it('should update progress as pages advance', async () => {
      await wrapper.vm.nextPage()
      expect(wrapper.vm.progressPercent).toBe(40) // 2/5 = 40%

      await wrapper.vm.nextPage()
      expect(wrapper.vm.progressPercent).toBe(60) // 3/5 = 60%
    })

    it('should show 100% on last page', async () => {
      const totalPages = wrapper.vm.totalPages
      await wrapper.vm.jumpToPage(totalPages - 1)

      expect(wrapper.vm.progressPercent).toBe(100)
    })
  })

  describe('Convergence Point Detection', () => {
    it('should detect convergence points', async () => {
      const convergenceTrail = createMockTrailWithConvergence()
      const convergenceWrapper = mount(LinearReader, {
        props: {
          trail: convergenceTrail
        }
      })

      // Navigate to convergence point (node4)
      const linearNodes = convergenceWrapper.vm.linearNodes
      const node4Index = linearNodes.findIndex((n: ContentNode) => n.id === 'node4')

      await convergenceWrapper.vm.jumpToPage(node4Index)

      expect(convergenceWrapper.vm.isCurrentNodeConvergence).toBe(true)
    })

    it('should not mark regular nodes as convergence points', () => {
      expect(wrapper.vm.isCurrentNodeConvergence).toBe(false)
    })
  })

  describe('Page Options', () => {
    it('should generate page options for selector', () => {
      const options = wrapper.vm.pageOptions

      expect(options.length).toBe(5)
      expect(options[0].label).toContain('Page 1')
      expect(options[0].value).toBe(0)
    })

    it('should include node IDs in page options', () => {
      const options = wrapper.vm.pageOptions

      expect(options[0].label).toContain('node1')
    })
  })

  describe('Keyboard Navigation', () => {
    it('should navigate next on arrow right', async () => {
      const keyEvent = new KeyboardEvent('keydown', { key: 'ArrowRight' })
      window.dispatchEvent(keyEvent)

      await wrapper.vm.$nextTick()

      expect(wrapper.vm.currentPage).toBe(1)
    })

    it('should navigate previous on arrow left', async () => {
      await wrapper.vm.nextPage()
      expect(wrapper.vm.currentPage).toBe(1)

      const keyEvent = new KeyboardEvent('keydown', { key: 'ArrowLeft' })
      window.dispatchEvent(keyEvent)

      await wrapper.vm.$nextTick()

      expect(wrapper.vm.currentPage).toBe(0)
    })

    it('should jump to first page on Home', async () => {
      await wrapper.vm.jumpToPage(2)

      const keyEvent = new KeyboardEvent('keydown', { key: 'Home' })
      window.dispatchEvent(keyEvent)

      await wrapper.vm.$nextTick()

      expect(wrapper.vm.currentPage).toBe(0)
    })

    it('should jump to last page on End', async () => {
      const totalPages = wrapper.vm.totalPages

      const keyEvent = new KeyboardEvent('keydown', { key: 'End' })
      window.dispatchEvent(keyEvent)

      await wrapper.vm.$nextTick()

      expect(wrapper.vm.currentPage).toBe(totalPages - 1)
    })
  })

  describe('Trail Changes', () => {
    it('should reset to first page when trail changes', async () => {
      await wrapper.vm.nextPage()
      expect(wrapper.vm.currentPage).toBe(1)

      // Update trail prop
      const newTrail = createMockTrailWithConvergence()
      await wrapper.setProps({ trail: newTrail })

      expect(wrapper.vm.currentPage).toBe(0)
    })
  })

  describe('Edge Cases', () => {
    it('should handle empty choices array', () => {
      // Node4 and Node5 have empty choices
      const totalPages = wrapper.vm.totalPages
      const linearNodes = wrapper.vm.linearNodes

      expect(linearNodes[totalPages - 1].content.choices).toEqual([])
    })

    it('should handle missing nodes gracefully', () => {
      // Create trail with broken edge reference
      const brokenTrail: Trail = {
        ...mockTrail,
        dag: {
          ...mockTrail.dag,
          edges: [
            ...mockTrail.dag.edges,
            { from_node_id: 'node5', to_node_id: 'nonexistent', choice_id: 'broken' }
          ]
        }
      }

      const brokenWrapper = mount(LinearReader, {
        props: { trail: brokenTrail }
      })

      // Should still work, just skip missing node
      expect(brokenWrapper.vm.linearNodes.length).toBeGreaterThan(0)
    })
  })
})
