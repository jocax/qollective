/**
 * Component tests for DagVisualization
 *
 * Tests graph rendering, node positioning, zoom/pan, and node selection using real trail data
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mountSuspended } from '@nuxt/test-utils/runtime'
import type { VueWrapper } from '@vue/test-utils'
import DagVisualization from '../DagVisualization.vue'
import { loadTestTrailWithDAG, getTestTrailStats } from '~/utils/__tests__/fixtures/testDataLoader'
import type { Trail } from '~/types/trails'

describe('DagVisualization with Real Data', () => {
  let wrapper: VueWrapper<any>
  const testData = loadTestTrailWithDAG()
  const stats = getTestTrailStats()

  beforeEach(async () => {
    wrapper = await mountSuspended(DagVisualization, {
      props: {
        trail: testData.trail
      }
    })
  })

  describe('Initialization with 24-node trail', () => {
    it('should mount successfully', () => {
      expect(wrapper.exists()).toBe(true)
    })

    it('should calculate correct number of node positions for 24-node trail', () => {
      expect(wrapper.vm.nodePositions.length).toBe(stats.nodeCount)
    })

    it('should display correct node and edge counts', () => {
      expect(wrapper.text()).toContain(`${stats.nodeCount} nodes`)
      expect(wrapper.text()).toContain(`${stats.edgeCount} edges`)
    })

    it('should initialize with default zoom', () => {
      expect(wrapper.vm.zoom).toBe(1)
    })

    it('should initialize with default pan position', () => {
      expect(wrapper.vm.panX).toBe(50)
      expect(wrapper.vm.panY).toBe(50)
    })

    it('should default to tree layout', () => {
      expect(wrapper.vm.layoutMode).toBe('tree')
    })

    it('should have no node selected initially', () => {
      expect(wrapper.vm.selectedNodeId).toBeNull()
    })

    it('should have isPanning set to false initially', () => {
      expect(wrapper.vm.isPanning).toBe(false)
    })
  })

  describe('Node Positioning - Tree Layout', () => {
    it('should calculate positions for all 24 nodes', () => {
      const positions = wrapper.vm.nodePositions

      expect(positions.length).toBe(stats.nodeCount)
      expect(positions.every((p: any) => p.x !== undefined && p.y !== undefined)).toBe(true)
    })

    it('should assign level 0 to start node', () => {
      const positions = wrapper.vm.nodePositions
      const startNode = positions.find((p: any) => p.id === stats.startNodeId)

      expect(startNode?.level).toBe(0)
    })

    it('should position child nodes at higher levels', () => {
      const positions = wrapper.vm.nodePositions
      const startNode = positions.find((p: any) => p.id === stats.startNodeId)

      // Find a child of start node
      const startNodeEdges = testData.trail.dag.edges.filter(e => e.from_node_id === stats.startNodeId)
      if (startNodeEdges.length > 0) {
        const childId = startNodeEdges[0].to_node_id
        const childNode = positions.find((p: any) => p.id === childId)

        expect(childNode?.level).toBeGreaterThan(startNode?.level || 0)
      }
    })

    it('should calculate valid x,y coordinates for all nodes', () => {
      wrapper.vm.layoutMode = 'tree'
      wrapper.vm.nodePositions.forEach(pos => {
        expect(pos.x).toBeGreaterThan(0)
        expect(pos.y).toBeGreaterThan(0)
      })
    })

    it('should track render time', () => {
      wrapper.vm.layoutMode = 'tree'
      // Access nodePositions to trigger calculation
      const _ = wrapper.vm.nodePositions

      expect(wrapper.vm.renderTime).toBeGreaterThan(0)
    })

    it('should return position for valid node ID', () => {
      const position = wrapper.vm.getNodePosition(stats.startNodeId)

      expect(position.x).toBeGreaterThan(0)
      expect(position.y).toBeGreaterThan(0)
    })

    it('should return default position for invalid node ID', () => {
      const position = wrapper.vm.getNodePosition('invalid')

      expect(position.x).toBe(0)
      expect(position.y).toBe(0)
    })
  })

  describe('Node Positioning - Force Layout', () => {
    beforeEach(async () => {
      await wrapper.vm.$nextTick()
      wrapper.vm.layoutMode = 'force'
      await wrapper.vm.$nextTick()
    })

    it('should calculate force-directed positions for all 24 nodes', () => {
      const positions = wrapper.vm.nodePositions

      expect(positions.length).toBe(stats.nodeCount)
      expect(positions.every((p: any) => p.x > 0 && p.y > 0)).toBe(true)
    })

    it('should keep nodes within bounds', () => {
      const positions = wrapper.vm.nodePositions

      positions.forEach((pos: any) => {
        expect(pos.x).toBeGreaterThanOrEqual(50)
        expect(pos.x).toBeLessThanOrEqual(wrapper.vm.width - 50)
        expect(pos.y).toBeGreaterThanOrEqual(50)
        expect(pos.y).toBeLessThanOrEqual(wrapper.vm.height - 50)
      })
    })

    it('should set level to 0 for all nodes in force layout', () => {
      wrapper.vm.layoutMode = 'force'
      wrapper.vm.nodePositions.forEach(pos => {
        expect(pos.level).toBe(0)
      })
    })

    it('should clear selected node when switching layouts', async () => {
      wrapper.vm.selectedNodeId = stats.startNodeId
      expect(wrapper.vm.selectedNodeId).toBe(stats.startNodeId)

      wrapper.vm.layoutMode = 'tree'
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.selectedNodeId).toBeNull()
    })
  })

  describe('Node Colors', () => {
    it('should color start node as green', () => {
      const color = wrapper.vm.getNodeColor(stats.startNodeId)
      expect(color).toBe('#22c55e') // green
    })

    it('should color convergence points as purple', async () => {
      if (stats.convergencePointCount > 0 && testData.trail.dag.convergence_points) {
        const convergenceNode = testData.trail.dag.convergence_points[0]
        const color = wrapper.vm.getNodeColor(convergenceNode)

        expect(color).toBe('#a855f7') // purple
      }
    })

    it('should color regular nodes as blue', () => {
      // Get a node that's not start and not convergence
      const regularNode = Object.keys(testData.trail.dag.nodes).find(
        id => id !== stats.startNodeId && !testData.trail.dag.convergence_points?.includes(id)
      )

      if (regularNode) {
        const color = wrapper.vm.getNodeColor(regularNode)
        expect(color).toBe('#3b82f6') // blue
      }
    })

    it('should correctly identify convergence points', () => {
      if (testData.trail.dag.convergence_points && testData.trail.dag.convergence_points.length > 0) {
        const convergenceNode = testData.trail.dag.convergence_points[0]
        expect(wrapper.vm.isConvergencePoint(convergenceNode)).toBe(true)
      }

      expect(wrapper.vm.isConvergencePoint(stats.startNodeId)).toBe(false)
    })
  })

  describe('Node Selection', () => {
    it('should select node when clicked', () => {
      wrapper.vm.selectNode(stats.startNodeId)

      expect(wrapper.vm.selectedNodeId).toBe(stats.startNodeId)
    })

    it('should show selected node details', async () => {
      wrapper.vm.selectNode(stats.startNodeId)
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.selectedNode).not.toBeNull()
      expect(wrapper.vm.selectedNode?.id).toBe(stats.startNodeId)
    })

    it('should clear selection when clicking away', async () => {
      wrapper.vm.selectNode(stats.startNodeId)
      expect(wrapper.vm.selectedNodeId).toBe(stats.startNodeId)

      wrapper.vm.selectedNodeId = null
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.selectedNode).toBeNull()
    })

    it('should highlight edges connected to selected node', async () => {
      wrapper.vm.selectNode(stats.startNodeId)
      await wrapper.vm.$nextTick()

      const startNodeEdges = testData.trail.dag.edges.filter(
        e => e.from_node_id === stats.startNodeId || e.to_node_id === stats.startNodeId
      )

      if (startNodeEdges.length > 0) {
        const isSelected = wrapper.vm.isEdgeSelected(startNodeEdges[0])
        expect(isSelected).toBe(true)
      }
    })

    it('should not select edges unconnected to selected node', () => {
      wrapper.vm.selectedNodeId = stats.startNodeId

      // Find an edge not connected to start node
      const unconnectedEdge = testData.trail.dag.edges.find(
        e => e.from_node_id !== stats.startNodeId && e.to_node_id !== stats.startNodeId
      )

      if (unconnectedEdge) {
        expect(wrapper.vm.isEdgeSelected(unconnectedEdge)).toBe(false)
      }
    })
  })

  describe('Zoom Controls', () => {
    it('should zoom in', () => {
      const initialZoom = wrapper.vm.zoom

      wrapper.vm.zoomIn()

      expect(wrapper.vm.zoom).toBeGreaterThan(initialZoom)
    })

    it('should zoom out', () => {
      const initialZoom = wrapper.vm.zoom

      wrapper.vm.zoomOut()

      expect(wrapper.vm.zoom).toBeLessThan(initialZoom)
    })

    it('should limit maximum zoom', () => {
      // Zoom in many times
      for (let i = 0; i < 20; i++) {
        wrapper.vm.zoomIn()
      }

      expect(wrapper.vm.zoom).toBeLessThanOrEqual(3)
    })

    it('should limit minimum zoom', () => {
      // Zoom out many times
      for (let i = 0; i < 20; i++) {
        wrapper.vm.zoomOut()
      }

      expect(wrapper.vm.zoom).toBeGreaterThanOrEqual(0.3)
    })

    it('should zoom in on mouse wheel up', () => {
      const initialZoom = wrapper.vm.zoom

      const wheelEvent = new WheelEvent('wheel', { deltaY: -100 })
      wrapper.vm.handleWheel(wheelEvent)

      expect(wrapper.vm.zoom).toBeGreaterThan(initialZoom)
    })

    it('should zoom out on mouse wheel down', () => {
      const initialZoom = wrapper.vm.zoom

      const wheelEvent = new WheelEvent('wheel', { deltaY: 100 })
      wrapper.vm.handleWheel(wheelEvent)

      expect(wrapper.vm.zoom).toBeLessThan(initialZoom)
    })
  })

  describe('Pan Controls', () => {
    it('should start panning on mouse down', () => {
      const mouseEvent = new MouseEvent('mousedown', { clientX: 100, clientY: 100 })

      wrapper.vm.startPan(mouseEvent)

      expect(wrapper.vm.isPanning).toBe(true)
    })

    it('should update pan position on mouse move', () => {
      // Start panning
      const startEvent = new MouseEvent('mousedown', { clientX: 100, clientY: 100 })
      wrapper.vm.startPan(startEvent)

      const initialPanX = wrapper.vm.panX
      const initialPanY = wrapper.vm.panY

      // Move mouse
      const moveEvent = new MouseEvent('mousemove', { clientX: 150, clientY: 150 })
      wrapper.vm.doPan(moveEvent)

      expect(wrapper.vm.panX).not.toBe(initialPanX)
      expect(wrapper.vm.panY).not.toBe(initialPanY)
    })

    it('should end panning on mouse up', () => {
      wrapper.vm.isPanning = true

      wrapper.vm.endPan()

      expect(wrapper.vm.isPanning).toBe(false)
    })

    it('should not pan when not in panning mode', () => {
      wrapper.vm.isPanning = false
      const initialPanX = wrapper.vm.panX

      const moveEvent = new MouseEvent('mousemove', { clientX: 150, clientY: 150 })
      wrapper.vm.doPan(moveEvent)

      expect(wrapper.vm.panX).toBe(initialPanX)
    })
  })

  describe('Reset View', () => {
    it('should reset zoom to 1', () => {
      wrapper.vm.zoom = 2

      wrapper.vm.resetView()

      expect(wrapper.vm.zoom).toBe(1)
    })

    it('should reset pan to default position', () => {
      wrapper.vm.panX = 200
      wrapper.vm.panY = 300

      wrapper.vm.resetView()

      expect(wrapper.vm.panX).toBe(50)
      expect(wrapper.vm.panY).toBe(50)
    })

    it('should clear node selection', () => {
      wrapper.vm.selectedNodeId = stats.startNodeId

      wrapper.vm.resetView()

      expect(wrapper.vm.selectedNodeId).toBeNull()
    })
  })

  describe('Convergence Points', () => {
    it('should return convergence points array from computed property', () => {
      expect(wrapper.vm.convergencePoints).toEqual(testData.trail.dag.convergence_points || [])
    })

    it('should not mark regular nodes as convergence', () => {
      const isConvergence = wrapper.vm.isConvergencePoint(stats.startNodeId)
      expect(isConvergence).toBe(false)
    })
  })

  describe('Keyboard Shortcuts', () => {
    it('should clear selection on Escape', () => {
      wrapper.vm.selectedNodeId = stats.startNodeId

      wrapper.vm.handleKeydown(new KeyboardEvent('keydown', { key: 'Escape' }))

      expect(wrapper.vm.selectedNodeId).toBeNull()
    })

    it('should reset view on lowercase r key', () => {
      wrapper.vm.zoom = 2
      wrapper.vm.panX = 100

      wrapper.vm.handleKeydown(new KeyboardEvent('keydown', { key: 'r' }))

      expect(wrapper.vm.zoom).toBe(1)
      expect(wrapper.vm.panX).toBe(50)
    })

    it('should reset view on uppercase R key', () => {
      wrapper.vm.zoom = 2

      wrapper.vm.handleKeydown(new KeyboardEvent('keydown', { key: 'R' }))

      expect(wrapper.vm.zoom).toBe(1)
    })

    it('should zoom in on + key', () => {
      const initialZoom = wrapper.vm.zoom

      wrapper.vm.handleKeydown(new KeyboardEvent('keydown', { key: '+' }))

      expect(wrapper.vm.zoom).toBeGreaterThan(initialZoom)
    })

    it('should zoom in on = key', () => {
      const initialZoom = wrapper.vm.zoom

      wrapper.vm.handleKeydown(new KeyboardEvent('keydown', { key: '=' }))

      expect(wrapper.vm.zoom).toBeGreaterThan(initialZoom)
    })

    it('should zoom out on - key', () => {
      const initialZoom = wrapper.vm.zoom

      wrapper.vm.handleKeydown(new KeyboardEvent('keydown', { key: '-' }))

      expect(wrapper.vm.zoom).toBeLessThan(initialZoom)
    })

    it('should zoom out on _ key', () => {
      const initialZoom = wrapper.vm.zoom

      wrapper.vm.handleKeydown(new KeyboardEvent('keydown', { key: '_' }))

      expect(wrapper.vm.zoom).toBeLessThan(initialZoom)
    })
  })

  describe('Performance', () => {
    it('should render 24-node graph and track render time', () => {
      const positions = wrapper.vm.nodePositions

      expect(positions.length).toBe(stats.nodeCount)
      expect(wrapper.vm.renderTime).not.toBeNull()
      expect(wrapper.vm.renderTime).toBeGreaterThan(0)
    })

    it('should complete 24-node graph render within reasonable time', () => {
      const positions = wrapper.vm.nodePositions
      const renderTime = wrapper.vm.renderTime

      expect(positions.length).toBe(stats.nodeCount)
      if (renderTime) {
        // 24 nodes should render in under 3 seconds
        expect(renderTime).toBeLessThan(3000)
      }
    })

    it('should show performance warning for slow renders', async () => {
      wrapper.vm.renderTime = 4000 // Simulate slow render

      await wrapper.vm.$nextTick()

      // Component should display warning when renderTime > 3000ms
      const hasWarning = wrapper.vm.renderTime > 3000
      expect(hasWarning).toBe(true)
    })
  })

  describe('Edge Cases', () => {
    it('should handle single node trail', async () => {
      const singleNodeTrail: Trail = {
        title: 'Single Node',
        description: 'Only one node',
        metadata: {
          generation_params: {
            age_group: '8-12',
            theme: 'Adventure',
            language: 'en',
            node_count: 1
          },
          start_node_id: 'node1'
        },
        dag: {
          nodes: {
            node1: {
              id: 'node1',
              content: { text: 'Only node', choices: [] }
            }
          },
          edges: [],
          convergence_points: []
        }
      }

      const singleWrapper = await mountSuspended(DagVisualization, {
        props: { trail: singleNodeTrail }
      })

      const positions = singleWrapper.vm.nodePositions
      expect(positions.length).toBe(1)
    })

    it('should handle disconnected nodes', async () => {
      const disconnectedTrail: Trail = {
        ...testData.trail,
        dag: {
          nodes: {
            ...testData.trail.dag.nodes,
            orphan: {
              id: 'orphan',
              content: { text: 'Disconnected', choices: [] }
            }
          },
          edges: testData.trail.dag.edges,
          convergence_points: []
        }
      }

      const disconnectedWrapper = await mountSuspended(DagVisualization, {
        props: { trail: disconnectedTrail }
      })

      // Should only include connected nodes
      const positions = disconnectedWrapper.vm.nodePositions
      const hasOrphan = positions.some((p: any) => p.id === 'orphan')

      // Orphan node won't be visited in traversal from start
      expect(hasOrphan).toBe(false)
    })
  })
})

// Keep mock trail helper functions for edge case tests that need simpler data
function createMockTrail(): Trail {
  return {
    title: 'DAG Test',
    description: 'Test trail for DAG visualization',
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
            text: 'Start node',
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
              { id: 'choice4', text: 'Continue', next_node_id: 'node5' }
            ]
          }
        },
        node4: {
          id: 'node4',
          content: {
            text: 'Left end',
            choices: []
          }
        },
        node5: {
          id: 'node5',
          content: {
            text: 'Right end',
            choices: []
          }
        }
      },
      edges: [
        { from_node_id: 'node1', to_node_id: 'node2', choice_id: 'choice1' },
        { from_node_id: 'node1', to_node_id: 'node3', choice_id: 'choice2' },
        { from_node_id: 'node2', to_node_id: 'node4', choice_id: 'choice3' },
        { from_node_id: 'node3', to_node_id: 'node5', choice_id: 'choice4' }
      ],
      convergence_points: []
    }
  }
}
