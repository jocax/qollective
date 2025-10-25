/**
 * Integration tests for InteractiveReader using real test data
 *
 * Tests interactive story navigation with actual 24-node DAG from
 * response_test_epic_2.json. Verifies:
 * - Node navigation and history tracking
 * - Choice selection and validation
 * - Progress calculation
 * - Convergence point detection
 * - Back button and restart functionality
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { mountSuspended } from '@nuxt/test-utils/runtime'
import type { VueWrapper } from '@vue/test-utils'
import { flushPromises } from '@vue/test-utils'
import InteractiveReader from '../InteractiveReader.vue'
import { loadTestTrailWithDAG, getTestTrailStats } from '~/utils/__tests__/fixtures/testDataLoader'
import type { Trail, Choice } from '~/types/trails'

describe('InteractiveReader - Integration Tests with Real Data', () => {
  let wrapper: VueWrapper<any>
  let testTrail: Trail
  let stats: ReturnType<typeof getTestTrailStats>

  beforeEach(() => {
    // Load real test trail with DAG
    const data = loadTestTrailWithDAG()
    testTrail = data.trail
    stats = getTestTrailStats()
    vi.clearAllMocks()
  })

  afterEach(() => {
    // mountSuspended handles cleanup automatically
  })

  describe('Initial State with Real Data', () => {
    it('initializes with start node ID "0"', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      expect(wrapper.vm.currentNodeId).toBe('0')
      expect(stats.startNodeId).toBe('0')
    })

    it('loads actual 24-node DAG structure', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      expect(wrapper.vm.totalNodes).toBe(24)
      expect(stats.nodeCount).toBe(24)
    })

    it('initializes with empty navigation history', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      expect(wrapper.vm.navigationHistory).toEqual([])
    })

    it('initializes with no explored nodes', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      expect(wrapper.vm.exploredNodes.size).toBe(0)
    })

    it('disables back button at start', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      expect(wrapper.vm.canGoBack).toBe(false)
    })

    it('initializes with showInsights true', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      expect(wrapper.vm.showInsights).toBe(true)
    })

    it('initializes with showMetadata false', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      expect(wrapper.vm.showMetadata).toBe(false)
    })

    it('has no errors on initialization', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      expect(wrapper.vm.error).toBeNull()
    })
  })

  describe('Current Node Resolution', () => {
    it('resolves current node from start node ID', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      const currentNode = wrapper.vm.currentNode
      expect(currentNode).not.toBeNull()
      expect(currentNode.id).toBe('0')
    })

    it('current node has content with text', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      const currentNode = wrapper.vm.currentNode
      expect(currentNode.content).toBeDefined()
      expect(currentNode.content.text).toBeTruthy()
      expect(typeof currentNode.content.text).toBe('string')
    })

    it('current node has choices array', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      const currentNode = wrapper.vm.currentNode
      expect(currentNode.content.choices).toBeDefined()
      expect(Array.isArray(currentNode.content.choices)).toBe(true)
      expect(currentNode.content.choices.length).toBeGreaterThan(0)
    })
  })

  describe('Choice Selection and Navigation', () => {
    it('navigates to next node when valid choice selected', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      const initialNodeId = wrapper.vm.currentNodeId
      const firstChoice = wrapper.vm.currentNode.content.choices[0]

      wrapper.vm.selectChoice(firstChoice)
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.currentNodeId).toBe(firstChoice.next_node_id)
      expect(wrapper.vm.currentNodeId).not.toBe(initialNodeId)
    })

    it('adds current node to navigation history on choice', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      const initialNodeId = wrapper.vm.currentNodeId
      const firstChoice = wrapper.vm.currentNode.content.choices[0]

      wrapper.vm.selectChoice(firstChoice)
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.navigationHistory).toContain(initialNodeId)
      expect(wrapper.vm.navigationHistory.length).toBe(1)
    })

    it('marks node as explored after navigating from it', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      const initialNodeId = wrapper.vm.currentNodeId
      const firstChoice = wrapper.vm.currentNode.content.choices[0]

      wrapper.vm.selectChoice(firstChoice)
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.exploredNodes.has(initialNodeId)).toBe(true)
    })

    it('tracks unique choices made', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      const firstChoice = wrapper.vm.currentNode.content.choices[0]

      wrapper.vm.selectChoice(firstChoice)
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.uniqueChoicesMade.has(firstChoice.id)).toBe(true)
      expect(wrapper.vm.uniqueChoicesMade.size).toBe(1)
    })

    it('clears errors on valid choice selection', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      // Set an error manually
      wrapper.vm.error = 'Test error'

      const firstChoice = wrapper.vm.currentNode.content.choices[0]
      wrapper.vm.selectChoice(firstChoice)
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.error).toBeNull()
    })

    it('handles multiple sequential choices', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      // Navigate through 3 nodes
      for (let i = 0; i < 3; i++) {
        const choice = wrapper.vm.currentNode.content.choices[0]
        wrapper.vm.selectChoice(choice)
        await wrapper.vm.$nextTick()
      }

      expect(wrapper.vm.navigationHistory.length).toBe(3)
      expect(wrapper.vm.exploredNodes.size).toBe(3)
      expect(wrapper.vm.uniqueChoicesMade.size).toBe(3)
    })
  })

  describe('Error Handling', () => {
    it('sets error when choice has no next_node_id', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      const invalidChoice: Choice = {
        id: 'invalid',
        text: 'Invalid choice',
        next_node_id: ''
      }

      wrapper.vm.selectChoice(invalidChoice)
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.error).toBe('Invalid choice: no next node specified')
    })

    it('sets error when next node does not exist in DAG', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      const invalidChoice: Choice = {
        id: 'invalid',
        text: 'Invalid choice',
        next_node_id: 'nonexistent-node-999'
      }

      wrapper.vm.selectChoice(invalidChoice)
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.error).toContain('not found in trail')
    })

    it('does not navigate when choice is invalid', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      const initialNodeId = wrapper.vm.currentNodeId

      const invalidChoice: Choice = {
        id: 'invalid',
        text: 'Invalid choice',
        next_node_id: ''
      }

      wrapper.vm.selectChoice(invalidChoice)
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.currentNodeId).toBe(initialNodeId)
    })
  })

  describe('Back Button Functionality', () => {
    it('enables back button after first navigation', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      expect(wrapper.vm.canGoBack).toBe(false)

      const firstChoice = wrapper.vm.currentNode.content.choices[0]
      wrapper.vm.selectChoice(firstChoice)
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.canGoBack).toBe(true)
    })

    it('navigates back to previous node', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      const startNodeId = wrapper.vm.currentNodeId
      const firstChoice = wrapper.vm.currentNode.content.choices[0]

      // Navigate forward
      wrapper.vm.selectChoice(firstChoice)
      await wrapper.vm.$nextTick()

      const secondNodeId = wrapper.vm.currentNodeId
      expect(secondNodeId).not.toBe(startNodeId)

      // Navigate back
      wrapper.vm.goBack()
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.currentNodeId).toBe(startNodeId)
    })

    it('removes last entry from navigation history on goBack', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      const firstChoice = wrapper.vm.currentNode.content.choices[0]
      wrapper.vm.selectChoice(firstChoice)
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.navigationHistory.length).toBe(1)

      wrapper.vm.goBack()
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.navigationHistory.length).toBe(0)
    })

    it('clears error on goBack', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      const firstChoice = wrapper.vm.currentNode.content.choices[0]
      wrapper.vm.selectChoice(firstChoice)
      await wrapper.vm.$nextTick()

      // Set error
      wrapper.vm.error = 'Test error'
      expect(wrapper.vm.error).not.toBeNull()

      wrapper.vm.goBack()
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.error).toBeNull()
    })

    it('does nothing when goBack called at start', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      const startNodeId = wrapper.vm.currentNodeId

      wrapper.vm.goBack()
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.currentNodeId).toBe(startNodeId)
      expect(wrapper.vm.navigationHistory.length).toBe(0)
    })
  })

  describe('Restart Functionality', () => {
    it('resets to start node on restart', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      // Navigate away
      const firstChoice = wrapper.vm.currentNode.content.choices[0]
      wrapper.vm.selectChoice(firstChoice)
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.currentNodeId).not.toBe('0')

      // Restart
      wrapper.vm.restart()
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.currentNodeId).toBe('0')
    })

    it('clears navigation history on restart', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      // Navigate multiple times
      for (let i = 0; i < 3; i++) {
        const choice = wrapper.vm.currentNode.content.choices[0]
        wrapper.vm.selectChoice(choice)
        await wrapper.vm.$nextTick()
      }

      expect(wrapper.vm.navigationHistory.length).toBeGreaterThan(0)

      wrapper.vm.restart()
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.navigationHistory).toEqual([])
    })

    it('clears explored nodes on restart', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      // Navigate to explore nodes
      const firstChoice = wrapper.vm.currentNode.content.choices[0]
      wrapper.vm.selectChoice(firstChoice)
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.exploredNodes.size).toBeGreaterThan(0)

      wrapper.vm.restart()
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.exploredNodes.size).toBe(0)
    })

    it('clears convergence points tracking on restart', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      // Manually add convergence point
      wrapper.vm.exploredConvergencePoints.add('test-node')

      wrapper.vm.restart()
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.exploredConvergencePoints.size).toBe(0)
    })

    it('clears unique choices on restart', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      const firstChoice = wrapper.vm.currentNode.content.choices[0]
      wrapper.vm.selectChoice(firstChoice)
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.uniqueChoicesMade.size).toBeGreaterThan(0)

      wrapper.vm.restart()
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.uniqueChoicesMade.size).toBe(0)
    })

    it('clears errors on restart', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      wrapper.vm.error = 'Test error'

      wrapper.vm.restart()
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.error).toBeNull()
    })
  })

  describe('Progress Tracking', () => {
    it('calculates progressPercent correctly with 0 explored', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      expect(wrapper.vm.progressPercent).toBe(0)
    })

    it('calculates progressPercent after exploring nodes', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      // Explore 1 node (total 24)
      const firstChoice = wrapper.vm.currentNode.content.choices[0]
      wrapper.vm.selectChoice(firstChoice)
      await wrapper.vm.$nextTick()

      const expectedPercent = Math.round((1 / 24) * 100) // ~4%
      expect(wrapper.vm.progressPercent).toBe(expectedPercent)
    })

    it('updates totalNodes display', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      expect(wrapper.vm.totalNodes).toBe(24)
    })
  })

  describe('Convergence Points', () => {
    it('loads convergence points from trail DAG', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      const convergencePoints = wrapper.vm.convergencePoints
      expect(Array.isArray(convergencePoints)).toBe(true)
    })

    it('detects when current node is convergence point', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      // Test depends on actual data structure
      const isConvergence = wrapper.vm.isCurrentNodeConvergence
      expect(typeof isConvergence).toBe('boolean')
    })

    it('tracks explored convergence points', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      // Navigate and check if convergence points are tracked
      const initialSize = wrapper.vm.exploredConvergencePoints.size

      const firstChoice = wrapper.vm.currentNode.content.choices[0]
      wrapper.vm.selectChoice(firstChoice)
      await wrapper.vm.$nextTick()

      // Size may or may not increase depending on whether node is convergence point
      expect(wrapper.vm.exploredConvergencePoints.size).toBeGreaterThanOrEqual(initialSize)
    })
  })

  describe('Toggle Controls', () => {
    it('toggles showMetadata', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      expect(wrapper.vm.showMetadata).toBe(false)

      wrapper.vm.showMetadata = true
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.showMetadata).toBe(true)
    })

    it('toggles showInsights', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      expect(wrapper.vm.showInsights).toBe(true)

      wrapper.vm.showInsights = false
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.showInsights).toBe(false)
    })
  })

  describe('Explored Choices Tracking', () => {
    it('identifies explored choices from current node', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      // Initially no explored choices
      let exploredChoiceIds = wrapper.vm.getExploredChoiceIds()
      expect(exploredChoiceIds).toEqual([])

      // Navigate and come back
      const firstChoice = wrapper.vm.currentNode.content.choices[0]
      const targetNodeId = firstChoice.next_node_id

      wrapper.vm.selectChoice(firstChoice)
      await wrapper.vm.$nextTick()

      // Mark the target as explored
      wrapper.vm.exploredNodes.add(targetNodeId)

      // Go back
      wrapper.vm.goBack()
      await wrapper.vm.$nextTick()

      // Now should have explored choices
      exploredChoiceIds = wrapper.vm.getExploredChoiceIds()
      expect(exploredChoiceIds.length).toBeGreaterThan(0)
    })

    it('returns empty array when no choices', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      // Manually set current node to one with no choices (if any exist)
      wrapper.vm.currentNodeId = 'nonexistent'
      await wrapper.vm.$nextTick()

      const exploredChoiceIds = wrapper.vm.getExploredChoiceIds()
      expect(exploredChoiceIds).toEqual([])
    })
  })

  describe('Trail Change Watcher', () => {
    it('restarts when trail prop changes', async () => {
      wrapper = await mountSuspended(InteractiveReader, {
        props: { trail: testTrail }
      })

      await flushPromises()

      // Navigate away from start
      const firstChoice = wrapper.vm.currentNode.content.choices[0]
      wrapper.vm.selectChoice(firstChoice)
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.currentNodeId).not.toBe('0')

      // Change trail prop (update with same trail but trigger watcher)
      await wrapper.setProps({ trail: { ...testTrail } })
      await flushPromises()

      // Should reset to start
      expect(wrapper.vm.currentNodeId).toBe('0')
    })
  })
})
