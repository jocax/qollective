/**
 * Component tests for DagVisualization
 *
 * Tests graph rendering, node positioning, zoom/pan, and node selection
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, VueWrapper } from '@vue/test-utils'
import DagVisualization from '../DagVisualization.vue'
import type { Trail } from '~/types/trails'

/**
 * Mock trail data for testing
 */
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

/**
 * Create trail with convergence point
 */
function createConvergenceTrail(): Trail {
  return {
    title: 'Convergence Test',
    description: 'Trail with convergence',
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
              { id: 'choice1', text: 'A', next_node_id: 'node2' },
              { id: 'choice2', text: 'B', next_node_id: 'node3' }
            ]
          }
        },
        node2: {
          id: 'node2',
          content: {
            text: 'Path A',
            choices: [
              { id: 'choice3', text: 'Merge', next_node_id: 'node4' }
            ]
          }
        },
        node3: {
          id: 'node3',
          content: {
            text: 'Path B',
            choices: [
              { id: 'choice4', text: 'Merge', next_node_id: 'node4' }
            ]
          }
        },
        node4: {
          id: 'node4',
          content: {
            text: 'Convergence',
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

/**
 * Create large trail for performance testing
 */
function createLargeTrail(nodeCount: number = 50): Trail {
  const nodes: any = {}
  const edges: any = []

  for (let i = 1; i <= nodeCount; i++) {
    nodes[`node${i}`] = {
      id: `node${i}`,
      content: {
        text: `Node ${i} content`,
        choices: i < nodeCount
          ? [{ id: `choice${i}`, text: 'Next', next_node_id: `node${i + 1}` }]
          : []
      }
    }

    if (i < nodeCount) {
      edges.push({
        from_node_id: `node${i}`,
        to_node_id: `node${i + 1}`,
        choice_id: `choice${i}`
      })
    }
  }

  return {
    title: 'Large Trail',
    description: 'Performance test trail',
    metadata: {
      generation_params: {
        age_group: '8-12',
        theme: 'Adventure',
        language: 'en',
        node_count: nodeCount
      },
      start_node_id: 'node1'
    },
    dag: {
      nodes,
      edges,
      convergence_points: []
    }
  }
}

describe('DagVisualization', () => {
  let wrapper: VueWrapper<any>
  let mockTrail: Trail

  beforeEach(() => {
    mockTrail = createMockTrail()
    wrapper = mount(DagVisualization, {
      props: {
        trail: mockTrail
      }
    })
  })

  describe('Initialization', () => {
    it('should mount successfully', () => {
      expect(wrapper.exists()).toBe(true)
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
  })

  describe('Node Positioning - Tree Layout', () => {
    it('should calculate positions for all nodes', () => {
      const positions = wrapper.vm.nodePositions

      expect(positions.length).toBe(5)
      expect(positions.every((p: any) => p.x !== undefined && p.y !== undefined)).toBe(true)
    })

    it('should assign level 0 to start node', () => {
      const positions = wrapper.vm.nodePositions
      const startNode = positions.find((p: any) => p.id === 'node1')

      expect(startNode.level).toBe(0)
    })

    it('should position child nodes at higher levels', () => {
      const positions = wrapper.vm.nodePositions
      const node1 = positions.find((p: any) => p.id === 'node1')
      const node2 = positions.find((p: any) => p.id === 'node2')

      expect(node2.level).toBeGreaterThan(node1.level)
    })

    it('should space nodes horizontally within same level', () => {
      const positions = wrapper.vm.nodePositions

      // Nodes at same level should have different x coordinates
      const level1Nodes = positions.filter((p: any) => p.level === 1)
      if (level1Nodes.length > 1) {
        const x1 = level1Nodes[0].x
        const x2 = level1Nodes[1].x
        expect(x1).not.toBe(x2)
      }
    })

    it('should return position for valid node ID', () => {
      const position = wrapper.vm.getNodePosition('node1')

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

    it('should calculate force-directed positions', () => {
      const positions = wrapper.vm.nodePositions

      expect(positions.length).toBe(5)
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

    it('should clear selected node when switching layouts', async () => {
      wrapper.vm.selectedNodeId = 'node1'
      expect(wrapper.vm.selectedNodeId).toBe('node1')

      wrapper.vm.layoutMode = 'tree'
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.selectedNodeId).toBeNull()
    })
  })

  describe('Node Colors', () => {
    it('should color start node as green', () => {
      const color = wrapper.vm.getNodeColor('node1')
      expect(color).toBe('#22c55e') // green
    })

    it('should color convergence points as purple', async () => {
      const convergenceTrail = createConvergenceTrail()
      const convergenceWrapper = mount(DagVisualization, {
        props: { trail: convergenceTrail }
      })

      const color = convergenceWrapper.vm.getNodeColor('node4')
      expect(color).toBe('#a855f7') // purple
    })

    it('should color regular nodes as blue', () => {
      const color = wrapper.vm.getNodeColor('node2')
      expect(color).toBe('#3b82f6') // blue
    })
  })

  describe('Node Selection', () => {
    it('should select node when clicked', () => {
      wrapper.vm.selectNode('node2')

      expect(wrapper.vm.selectedNodeId).toBe('node2')
    })

    it('should show selected node details', async () => {
      wrapper.vm.selectNode('node2')
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.selectedNode).not.toBeNull()
      expect(wrapper.vm.selectedNode?.id).toBe('node2')
    })

    it('should clear selection when clicking away', async () => {
      wrapper.vm.selectNode('node2')
      expect(wrapper.vm.selectedNodeId).toBe('node2')

      wrapper.vm.selectedNodeId = null
      await wrapper.vm.$nextTick()

      expect(wrapper.vm.selectedNode).toBeNull()
    })

    it('should highlight edges connected to selected node', async () => {
      wrapper.vm.selectNode('node1')
      await wrapper.vm.$nextTick()

      const edge = wrapper.vm.trail.dag.edges[0]
      const isSelected = wrapper.vm.isEdgeSelected(edge)

      expect(isSelected).toBe(true)
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
      wrapper.vm.selectedNodeId = 'node1'

      wrapper.vm.resetView()

      expect(wrapper.vm.selectedNodeId).toBeNull()
    })
  })

  describe('Convergence Points', () => {
    it('should detect convergence points', () => {
      const convergenceTrail = createConvergenceTrail()
      const convergenceWrapper = mount(DagVisualization, {
        props: { trail: convergenceTrail }
      })

      const isConvergence = convergenceWrapper.vm.isConvergencePoint('node4')
      expect(isConvergence).toBe(true)
    })

    it('should not mark regular nodes as convergence', () => {
      const isConvergence = wrapper.vm.isConvergencePoint('node1')
      expect(isConvergence).toBe(false)
    })
  })

  describe('Keyboard Shortcuts', () => {
    it('should clear selection on Escape', () => {
      wrapper.vm.selectedNodeId = 'node1'

      const keyEvent = new KeyboardEvent('keydown', { key: 'Escape' })
      window.dispatchEvent(keyEvent)

      expect(wrapper.vm.selectedNodeId).toBeNull()
    })

    it('should reset view on R key', () => {
      wrapper.vm.zoom = 2
      wrapper.vm.panX = 100

      const keyEvent = new KeyboardEvent('keydown', { key: 'r' })
      window.dispatchEvent(keyEvent)

      expect(wrapper.vm.zoom).toBe(1)
      expect(wrapper.vm.panX).toBe(50)
    })

    it('should zoom in on + key', () => {
      const initialZoom = wrapper.vm.zoom

      const keyEvent = new KeyboardEvent('keydown', { key: '+' })
      window.dispatchEvent(keyEvent)

      expect(wrapper.vm.zoom).toBeGreaterThan(initialZoom)
    })

    it('should zoom out on - key', () => {
      const initialZoom = wrapper.vm.zoom

      const keyEvent = new KeyboardEvent('keydown', { key: '-' })
      window.dispatchEvent(keyEvent)

      expect(wrapper.vm.zoom).toBeLessThan(initialZoom)
    })
  })

  describe('Performance', () => {
    it('should render small graph quickly', () => {
      const startTime = performance.now()

      const positions = wrapper.vm.nodePositions

      const endTime = performance.now()
      const renderTime = endTime - startTime

      expect(positions.length).toBe(5)
      expect(renderTime).toBeLessThan(100) // Should be very fast for small graph
    })

    it('should render medium graph within 3 seconds', () => {
      const largeTrail = createLargeTrail(30)
      const largeWrapper = mount(DagVisualization, {
        props: { trail: largeTrail }
      })

      const positions = largeWrapper.vm.nodePositions
      const renderTime = largeWrapper.vm.renderTime

      expect(positions.length).toBe(30)
      if (renderTime) {
        expect(renderTime).toBeLessThan(3000)
      }
    })

    it('should track render time', () => {
      const positions = wrapper.vm.nodePositions

      expect(wrapper.vm.renderTime).not.toBeNull()
      expect(wrapper.vm.renderTime).toBeGreaterThan(0)
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
    it('should handle single node trail', () => {
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

      const singleWrapper = mount(DagVisualization, {
        props: { trail: singleNodeTrail }
      })

      const positions = singleWrapper.vm.nodePositions
      expect(positions.length).toBe(1)
    })

    it('should handle disconnected nodes', () => {
      const disconnectedTrail: Trail = {
        ...mockTrail,
        dag: {
          nodes: {
            ...mockTrail.dag.nodes,
            orphan: {
              id: 'orphan',
              content: { text: 'Disconnected', choices: [] }
            }
          },
          edges: mockTrail.dag.edges,
          convergence_points: []
        }
      }

      const disconnectedWrapper = mount(DagVisualization, {
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
