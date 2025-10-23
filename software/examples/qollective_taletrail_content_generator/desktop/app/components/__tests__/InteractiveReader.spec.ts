/**
 * Component tests for InteractiveReader
 *
 * NOTE: These tests require Vitest and @vue/test-utils to be installed.
 * To set up testing, run:
 *
 *   bun add -D vitest @vue/test-utils @vitejs/plugin-vue happy-dom
 *
 * Then add to nuxt.config.ts:
 *
 *   export default defineNuxtConfig({
 *     vite: {
 *       test: {
 *         environment: 'happy-dom',
 *         globals: true
 *       }
 *     }
 *   })
 *
 * Run tests with: bun test or npm test
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, VueWrapper } from '@vue/test-utils'
import InteractiveReader from '../InteractiveReader.vue'
import type { Trail, ContentNode, Choice } from '~/types/trails'

/**
 * Mock trail data for testing
 */
function createMockTrail(): Trail {
  return {
    title: 'Test Adventure',
    description: 'A test story for unit testing',
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
            text: 'You are at the starting point. What do you do?',
            choices: [
              { id: 'choice1', text: 'Go left', next_node_id: 'node2' },
              { id: 'choice2', text: 'Go right', next_node_id: 'node3' }
            ]
          },
          generation_metadata: {
            llm_model: 'gpt-4',
            timestamp: '2024-01-01T00:00:00Z'
          }
        },
        node2: {
          id: 'node2',
          content: {
            text: 'You went left and found a treasure!',
            choices: [
              { id: 'choice3', text: 'Take treasure', next_node_id: 'node4' }
            ]
          }
        },
        node3: {
          id: 'node3',
          content: {
            text: 'You went right and met a dragon!',
            choices: [
              { id: 'choice4', text: 'Fight dragon', next_node_id: 'node4' }
            ]
          }
        },
        node4: {
          id: 'node4',
          content: {
            text: 'The end of your adventure!',
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

describe('InteractiveReader', () => {
  let wrapper: VueWrapper<any>
  let mockTrail: Trail

  beforeEach(() => {
    mockTrail = createMockTrail()
    wrapper = mount(InteractiveReader, {
      props: {
        trail: mockTrail
      }
    })
  })

  describe('Initialization', () => {
    it('should initialize at start node', () => {
      expect(wrapper.vm.currentNodeId).toBe('node1')
    })

    it('should display start node content', () => {
      const text = wrapper.text()
      expect(text).toContain('You are at the starting point')
    })

    it('should have empty navigation history on start', () => {
      expect(wrapper.vm.navigationHistory).toHaveLength(0)
    })

    it('should have no explored nodes initially', () => {
      expect(wrapper.vm.exploredNodes.size).toBe(0)
    })

    it('should disable back button at start', () => {
      expect(wrapper.vm.canGoBack).toBe(false)
    })
  })

  describe('Node Navigation', () => {
    it('should navigate to next node when choice is selected', async () => {
      const choice: Choice = mockTrail.dag.nodes.node1.content.choices![0]

      await wrapper.vm.selectChoice(choice)

      expect(wrapper.vm.currentNodeId).toBe('node2')
      expect(wrapper.text()).toContain('You went left and found a treasure!')
    })

    it('should add current node to history when navigating forward', async () => {
      const choice: Choice = mockTrail.dag.nodes.node1.content.choices![0]

      await wrapper.vm.selectChoice(choice)

      expect(wrapper.vm.navigationHistory).toContain('node1')
      expect(wrapper.vm.navigationHistory).toHaveLength(1)
    })

    it('should track explored nodes', async () => {
      const choice: Choice = mockTrail.dag.nodes.node1.content.choices![0]

      await wrapper.vm.selectChoice(choice)

      expect(wrapper.vm.exploredNodes.has('node1')).toBe(true)
    })

    it('should enable back button after navigation', async () => {
      const choice: Choice = mockTrail.dag.nodes.node1.content.choices![0]

      await wrapper.vm.selectChoice(choice)

      expect(wrapper.vm.canGoBack).toBe(true)
    })

    it('should handle invalid choice gracefully', async () => {
      const invalidChoice: Choice = {
        id: 'invalid',
        text: 'Invalid',
        next_node_id: 'nonexistent'
      }

      await wrapper.vm.selectChoice(invalidChoice)

      expect(wrapper.vm.error).toBeTruthy()
      expect(wrapper.vm.currentNodeId).toBe('node1') // Should stay at current node
    })
  })

  describe('Back Navigation', () => {
    it('should return to previous node when going back', async () => {
      // Navigate forward
      const choice: Choice = mockTrail.dag.nodes.node1.content.choices![0]
      await wrapper.vm.selectChoice(choice)
      expect(wrapper.vm.currentNodeId).toBe('node2')

      // Navigate back
      await wrapper.vm.goBack()
      expect(wrapper.vm.currentNodeId).toBe('node1')
    })

    it('should remove node from history when going back', async () => {
      const choice: Choice = mockTrail.dag.nodes.node1.content.choices![0]
      await wrapper.vm.selectChoice(choice)
      expect(wrapper.vm.navigationHistory).toHaveLength(1)

      await wrapper.vm.goBack()
      expect(wrapper.vm.navigationHistory).toHaveLength(0)
    })

    it('should not go back when at start node', async () => {
      const initialNode = wrapper.vm.currentNodeId

      await wrapper.vm.goBack()

      expect(wrapper.vm.currentNodeId).toBe(initialNode)
    })
  })

  describe('Restart Functionality', () => {
    it('should return to start node when restarting', async () => {
      // Navigate through trail
      const choice1: Choice = mockTrail.dag.nodes.node1.content.choices![0]
      await wrapper.vm.selectChoice(choice1)
      const choice2: Choice = mockTrail.dag.nodes.node2.content.choices![0]
      await wrapper.vm.selectChoice(choice2)

      // Restart
      await wrapper.vm.restart()

      expect(wrapper.vm.currentNodeId).toBe('node1')
    })

    it('should clear navigation history when restarting', async () => {
      const choice: Choice = mockTrail.dag.nodes.node1.content.choices![0]
      await wrapper.vm.selectChoice(choice)

      await wrapper.vm.restart()

      expect(wrapper.vm.navigationHistory).toHaveLength(0)
    })

    it('should clear explored nodes when restarting', async () => {
      const choice: Choice = mockTrail.dag.nodes.node1.content.choices![0]
      await wrapper.vm.selectChoice(choice)

      await wrapper.vm.restart()

      expect(wrapper.vm.exploredNodes.size).toBe(0)
    })

    it('should clear error state when restarting', async () => {
      wrapper.vm.error = 'Test error'

      await wrapper.vm.restart()

      expect(wrapper.vm.error).toBeNull()
    })
  })

  describe('Progress Tracking', () => {
    it('should calculate progress percentage correctly', async () => {
      // Explore 2 out of 4 nodes
      const choice1: Choice = mockTrail.dag.nodes.node1.content.choices![0]
      await wrapper.vm.selectChoice(choice1)
      const choice2: Choice = mockTrail.dag.nodes.node2.content.choices![0]
      await wrapper.vm.selectChoice(choice2)

      expect(wrapper.vm.progressPercent).toBe(50) // 2/4 = 50%
    })

    it('should track unique choices made', async () => {
      const choice1: Choice = mockTrail.dag.nodes.node1.content.choices![0]
      await wrapper.vm.selectChoice(choice1)

      expect(wrapper.vm.uniqueChoicesMade.has('choice1')).toBe(true)
      expect(wrapper.vm.uniqueChoicesMade.size).toBe(1)
    })

    it('should track convergence points explored', async () => {
      // Navigate to convergence point (node4)
      const choice1: Choice = mockTrail.dag.nodes.node1.content.choices![0]
      await wrapper.vm.selectChoice(choice1)
      const choice2: Choice = mockTrail.dag.nodes.node2.content.choices![0]
      await wrapper.vm.selectChoice(choice2)

      // node4 is a convergence point
      expect(wrapper.vm.exploredConvergencePoints.size).toBeGreaterThan(0)
    })
  })

  describe('UI State Management', () => {
    it('should toggle educational insights', async () => {
      expect(wrapper.vm.showInsights).toBe(true)

      wrapper.vm.showInsights = false

      expect(wrapper.vm.showInsights).toBe(false)
    })

    it('should toggle metadata display', async () => {
      expect(wrapper.vm.showMetadata).toBe(false)

      wrapper.vm.showMetadata = true

      expect(wrapper.vm.showMetadata).toBe(true)
    })

    it('should identify convergence points correctly', async () => {
      // Navigate to convergence point
      const choice1: Choice = mockTrail.dag.nodes.node1.content.choices![0]
      await wrapper.vm.selectChoice(choice1)
      const choice2: Choice = mockTrail.dag.nodes.node2.content.choices![0]
      await wrapper.vm.selectChoice(choice2)

      expect(wrapper.vm.isCurrentNodeConvergence).toBe(true)
    })
  })

  describe('Explored Choices', () => {
    it('should mark choices as explored after visiting their target nodes', async () => {
      const choice1: Choice = mockTrail.dag.nodes.node1.content.choices![0]
      await wrapper.vm.selectChoice(choice1)

      // Go back to node1
      await wrapper.vm.goBack()

      const exploredChoices = wrapper.vm.getExploredChoiceIds()
      expect(exploredChoices).toContain('choice1')
    })

    it('should not mark unvisited choices as explored', async () => {
      const exploredChoices = wrapper.vm.getExploredChoiceIds()
      expect(exploredChoices).toHaveLength(0)
    })
  })

  describe('End Node Handling', () => {
    it('should handle end nodes with no choices', async () => {
      // Navigate to end node
      const choice1: Choice = mockTrail.dag.nodes.node1.content.choices![0]
      await wrapper.vm.selectChoice(choice1)
      const choice2: Choice = mockTrail.dag.nodes.node2.content.choices![0]
      await wrapper.vm.selectChoice(choice2)

      const currentNode = wrapper.vm.currentNode
      expect(currentNode.content.choices).toHaveLength(0)
    })
  })

  describe('Error Handling', () => {
    it('should set error when selecting choice with no next_node_id', async () => {
      const invalidChoice: Choice = {
        id: 'bad',
        text: 'Bad choice',
        next_node_id: ''
      }

      await wrapper.vm.selectChoice(invalidChoice)

      expect(wrapper.vm.error).toBeTruthy()
    })

    it('should clear error on successful navigation', async () => {
      wrapper.vm.error = 'Previous error'

      const choice: Choice = mockTrail.dag.nodes.node1.content.choices![0]
      await wrapper.vm.selectChoice(choice)

      expect(wrapper.vm.error).toBeNull()
    })
  })
})

/**
 * Integration test scenarios to manually verify:
 *
 * 1. Load a trail from the list and verify it opens in Interactive mode
 * 2. Make a choice and verify the story progresses correctly
 * 3. Click Back and verify it returns to the previous node with the same content
 * 4. Make a different choice and verify branching works
 * 5. Navigate to a convergence point and verify the badge appears
 * 6. Click Restart and verify it returns to the start with cleared progress
 * 7. Toggle Educational Insights and verify it shows/hides
 * 8. Toggle Show Metadata and verify node metadata appears
 * 9. Navigate to an end node and verify "The End" message appears
 * 10. Verify progress indicator updates as you explore nodes
 * 11. Verify explored choices are marked with "Explored" badge
 * 12. Test keyboard navigation (arrow keys + Enter) on choices
 */
