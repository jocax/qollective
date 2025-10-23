<template>
  <div class="dag-visualization">
    <!-- Controls Bar -->
    <div class="controls-bar flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-900 rounded-lg mb-6">
      <div class="flex items-center gap-3">
        <UButtonGroup>
          <UButton
            icon="i-heroicons-plus"
            size="sm"
            @click="zoomIn"
            title="Zoom In"
          >
            Zoom In
          </UButton>
          <UButton
            icon="i-heroicons-minus"
            size="sm"
            @click="zoomOut"
            title="Zoom Out"
          >
            Zoom Out
          </UButton>
          <UButton
            icon="i-heroicons-arrow-path"
            size="sm"
            @click="resetView"
            title="Reset View"
          >
            Reset
          </UButton>
        </UButtonGroup>

        <USelect
          v-model="layoutMode"
          :options="layoutOptions"
          size="sm"
          class="w-40"
        />
      </div>

      <div class="text-sm text-gray-600 dark:text-gray-400">
        {{ nodePositions.length }} nodes, {{ trail.dag.edges.length }} edges
      </div>
    </div>

    <!-- Performance Warning -->
    <UAlert
      v-if="renderTime && renderTime > 3000"
      color="yellow"
      variant="subtle"
      icon="i-heroicons-exclamation-triangle"
      title="Performance Warning"
      :description="`Graph rendering took ${(renderTime / 1000).toFixed(2)}s. Consider using a simpler layout for large graphs.`"
      class="mb-4"
    />

    <!-- SVG Canvas -->
    <div class="canvas-container bg-white dark:bg-gray-950 rounded-lg border border-gray-200 dark:border-gray-800 overflow-hidden">
      <svg
        ref="svgRef"
        :width="width"
        :height="height"
        class="cursor-grab active:cursor-grabbing"
        @mousedown="startPan"
        @mousemove="doPan"
        @mouseup="endPan"
        @mouseleave="endPan"
        @wheel.prevent="handleWheel"
      >
        <defs>
          <!-- Arrow marker for edges -->
          <marker
            id="arrowhead"
            markerWidth="10"
            markerHeight="10"
            refX="9"
            refY="3"
            orient="auto"
            markerUnits="strokeWidth"
          >
            <polygon points="0 0, 10 3, 0 6" fill="#999" />
          </marker>

          <!-- Arrow marker for selected edges -->
          <marker
            id="arrowhead-selected"
            markerWidth="10"
            markerHeight="10"
            refX="9"
            refY="3"
            orient="auto"
            markerUnits="strokeWidth"
          >
            <polygon points="0 0, 10 3, 0 6" fill="#3b82f6" />
          </marker>
        </defs>

        <g :transform="`translate(${panX}, ${panY}) scale(${zoom})`">
          <!-- Edges -->
          <g class="edges">
            <line
              v-for="edge in trail.dag.edges"
              :key="`${edge.from_node_id}-${edge.to_node_id}`"
              :x1="getNodePosition(edge.from_node_id).x"
              :y1="getNodePosition(edge.from_node_id).y"
              :x2="getNodePosition(edge.to_node_id).x"
              :y2="getNodePosition(edge.to_node_id).y"
              :stroke="isEdgeSelected(edge) ? '#3b82f6' : '#999'"
              :stroke-width="isEdgeSelected(edge) ? '3' : '2'"
              :marker-end="isEdgeSelected(edge) ? 'url(#arrowhead-selected)' : 'url(#arrowhead)'"
              class="transition-all"
            />
          </g>

          <!-- Nodes -->
          <g class="nodes">
            <g
              v-for="nodePos in nodePositions"
              :key="nodePos.id"
              :transform="`translate(${nodePos.x}, ${nodePos.y})`"
              class="cursor-pointer"
              @click="selectNode(nodePos.id)"
            >
              <!-- Node circle -->
              <circle
                r="25"
                :fill="getNodeColor(nodePos.id)"
                :stroke="selectedNodeId === nodePos.id ? '#000' : 'none'"
                :stroke-width="selectedNodeId === nodePos.id ? '3' : '0'"
                class="transition-all hover:r-28"
              />

              <!-- Node label -->
              <text
                text-anchor="middle"
                dy="0.35em"
                fill="white"
                font-size="13"
                font-weight="600"
                class="pointer-events-none select-none"
              >
                {{ nodePos.id }}
              </text>
            </g>
          </g>
        </g>
      </svg>
    </div>

    <!-- Node Details Panel -->
    <div v-if="selectedNode" class="node-details mt-6">
      <UCard>
        <template #header>
          <div class="flex items-center justify-between">
            <h3 class="text-lg font-semibold">
              Node {{ selectedNode.id }}
              <UBadge v-if="isConvergencePoint(selectedNode.id)" color="purple" class="ml-2">
                Convergence Point
              </UBadge>
            </h3>
            <UButton
              icon="i-heroicons-x-mark"
              variant="ghost"
              size="sm"
              @click="selectedNodeId = null"
            />
          </div>
        </template>

        <StoryNode :node="selectedNode" :show-metadata="true" />
      </UCard>
    </div>

    <!-- Legend -->
    <div class="legend mt-6 p-4 bg-gray-50 dark:bg-gray-900 rounded-lg">
      <h4 class="text-sm font-semibold mb-3 text-gray-700 dark:text-gray-300">Legend</h4>
      <div class="flex flex-wrap gap-4">
        <div class="flex items-center gap-2">
          <div class="w-6 h-6 rounded-full bg-blue-500"></div>
          <span class="text-sm text-gray-600 dark:text-gray-400">Standard Node</span>
        </div>
        <div class="flex items-center gap-2">
          <div class="w-6 h-6 rounded-full bg-purple-500"></div>
          <span class="text-sm text-gray-600 dark:text-gray-400">Convergence Point</span>
        </div>
        <div class="flex items-center gap-2">
          <div class="w-6 h-6 rounded-full bg-green-500"></div>
          <span class="text-sm text-gray-600 dark:text-gray-400">Start Node</span>
        </div>
        <div class="flex items-center gap-2">
          <div class="w-6 h-6 rounded-full border-4 border-black"></div>
          <span class="text-sm text-gray-600 dark:text-gray-400">Selected Node</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import type { Trail, ContentNode, Edge } from '~/types/trails'

interface Props {
  trail: Trail
}

const props = defineProps<Props>()

interface NodePosition {
  id: string
  x: number
  y: number
  level: number
}

// State
const svgRef = ref<SVGSVGElement | null>(null)
const width = ref(1200)
const height = ref(800)
const zoom = ref(1)
const panX = ref(50)
const panY = ref(50)
const isPanning = ref(false)
const panStartX = ref(0)
const panStartY = ref(0)
const selectedNodeId = ref<string | null>(null)
const layoutMode = ref<'tree' | 'force'>('tree')
const renderTime = ref<number | null>(null)

const layoutOptions = [
  { label: 'Tree Layout', value: 'tree' },
  { label: 'Force Layout', value: 'force' }
]

// Computed properties
const convergencePoints = computed(() => {
  return props.trail.dag.convergence_points || []
})

const selectedNode = computed<ContentNode | null>(() => {
  if (!selectedNodeId.value) return null
  return props.trail.dag.nodes[selectedNodeId.value] || null
})

/**
 * Calculate tree layout positions for nodes
 */
function calculateTreeLayout(): NodePosition[] {
  const startTime = performance.now()
  const positions: NodePosition[] = []
  const levelMap = new Map<string, number>()
  const startNodeId = props.trail.metadata.start_node_id

  // BFS to assign levels
  const queue: Array<{ id: string; level: number }> = [{ id: startNodeId, level: 0 }]
  const visited = new Set<string>()

  while (queue.length > 0) {
    const { id, level } = queue.shift()!
    if (visited.has(id)) continue
    visited.add(id)
    levelMap.set(id, level)

    const children = props.trail.dag.edges.filter(e => e.from_node_id === id)
    children.forEach(edge => {
      queue.push({ id: edge.to_node_id, level: level + 1 })
    })
  }

  // Group nodes by level
  const levelGroups = new Map<number, string[]>()
  levelMap.forEach((level, id) => {
    if (!levelGroups.has(level)) levelGroups.set(level, [])
    levelGroups.get(level)!.push(id)
  })

  // Calculate positions
  const horizontalSpacing = 150
  const verticalSpacing = 120

  levelGroups.forEach((ids, level) => {
    const levelWidth = ids.length * horizontalSpacing
    const startX = (width.value - levelWidth) / 2

    ids.forEach((id, index) => {
      positions.push({
        id,
        x: startX + index * horizontalSpacing + horizontalSpacing / 2,
        y: level * verticalSpacing + 80,
        level
      })
    })
  })

  const endTime = performance.now()
  renderTime.value = endTime - startTime
  console.log(`Tree layout calculated in ${renderTime.value.toFixed(2)}ms`)

  return positions
}

/**
 * Calculate force-directed layout positions
 * Simplified force-directed algorithm
 */
function calculateForceLayout(): NodePosition[] {
  const startTime = performance.now()
  const positions: NodePosition[] = []
  const nodes = Object.keys(props.trail.dag.nodes)

  // Initialize positions randomly
  const posMap = new Map<string, { x: number; y: number }>()
  nodes.forEach(id => {
    posMap.set(id, {
      x: Math.random() * (width.value - 100) + 50,
      y: Math.random() * (height.value - 100) + 50
    })
  })

  // Simple force simulation (limited iterations for performance)
  const iterations = Math.min(50, nodes.length * 2)
  const repulsion = 2000
  const attraction = 0.01

  for (let iter = 0; iter < iterations; iter++) {
    const forces = new Map<string, { x: number; y: number }>()
    nodes.forEach(id => forces.set(id, { x: 0, y: 0 }))

    // Repulsion between all nodes
    for (let i = 0; i < nodes.length; i++) {
      for (let j = i + 1; j < nodes.length; j++) {
        const node1 = nodes[i]
        const node2 = nodes[j]
        const pos1 = posMap.get(node1)!
        const pos2 = posMap.get(node2)!

        const dx = pos2.x - pos1.x
        const dy = pos2.y - pos1.y
        const dist = Math.sqrt(dx * dx + dy * dy) || 1

        const force = repulsion / (dist * dist)
        const fx = (dx / dist) * force
        const fy = (dy / dist) * force

        forces.get(node1)!.x -= fx
        forces.get(node1)!.y -= fy
        forces.get(node2)!.x += fx
        forces.get(node2)!.y += fy
      }
    }

    // Attraction along edges
    props.trail.dag.edges.forEach(edge => {
      const pos1 = posMap.get(edge.from_node_id)!
      const pos2 = posMap.get(edge.to_node_id)!

      const dx = pos2.x - pos1.x
      const dy = pos2.y - pos1.y

      const fx = dx * attraction
      const fy = dy * attraction

      forces.get(edge.from_node_id)!.x += fx
      forces.get(edge.from_node_id)!.y += fy
      forces.get(edge.to_node_id)!.x -= fx
      forces.get(edge.to_node_id)!.y -= fy
    })

    // Apply forces
    nodes.forEach(id => {
      const pos = posMap.get(id)!
      const force = forces.get(id)!

      pos.x += force.x * 0.1
      pos.y += force.y * 0.1

      // Keep within bounds
      pos.x = Math.max(50, Math.min(width.value - 50, pos.x))
      pos.y = Math.max(50, Math.min(height.value - 50, pos.y))
    })
  }

  // Convert to positions array
  posMap.forEach((pos, id) => {
    positions.push({ id, x: pos.x, y: pos.y, level: 0 })
  })

  const endTime = performance.now()
  renderTime.value = endTime - startTime
  console.log(`Force layout calculated in ${renderTime.value.toFixed(2)}ms`)

  return positions
}

const nodePositions = computed<NodePosition[]>(() => {
  if (layoutMode.value === 'force') {
    return calculateForceLayout()
  }
  return calculateTreeLayout()
})

// Helper functions
function getNodePosition(nodeId: string): { x: number; y: number } {
  const pos = nodePositions.value.find(p => p.id === nodeId)
  return pos ? { x: pos.x, y: pos.y } : { x: 0, y: 0 }
}

function getNodeColor(nodeId: string): string {
  if (nodeId === props.trail.metadata.start_node_id) return '#22c55e' // green for start
  if (isConvergencePoint(nodeId)) return '#a855f7' // purple for convergence
  return '#3b82f6' // blue for regular nodes
}

function isConvergencePoint(nodeId: string): boolean {
  return convergencePoints.value.includes(nodeId)
}

function isEdgeSelected(edge: Edge): boolean {
  if (!selectedNodeId.value) return false
  return edge.from_node_id === selectedNodeId.value || edge.to_node_id === selectedNodeId.value
}

// Interaction methods
function selectNode(nodeId: string) {
  selectedNodeId.value = nodeId
}

function zoomIn() {
  zoom.value = Math.min(zoom.value * 1.2, 3)
}

function zoomOut() {
  zoom.value = Math.max(zoom.value / 1.2, 0.3)
}

function resetView() {
  zoom.value = 1
  panX.value = 50
  panY.value = 50
  selectedNodeId.value = null
}

function handleWheel(event: WheelEvent) {
  const delta = event.deltaY
  if (delta > 0) {
    zoomOut()
  } else {
    zoomIn()
  }
}

function startPan(event: MouseEvent) {
  isPanning.value = true
  panStartX.value = event.clientX - panX.value
  panStartY.value = event.clientY - panY.value
}

function doPan(event: MouseEvent) {
  if (!isPanning.value) return
  panX.value = event.clientX - panStartX.value
  panY.value = event.clientY - panStartY.value
}

function endPan() {
  isPanning.value = false
}

// Keyboard shortcuts
function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    selectedNodeId.value = null
  } else if (event.key === 'r' || event.key === 'R') {
    resetView()
  } else if (event.key === '+' || event.key === '=') {
    zoomIn()
  } else if (event.key === '-' || event.key === '_') {
    zoomOut()
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})

// Watch for layout mode changes
watch(layoutMode, () => {
  selectedNodeId.value = null
})
</script>

<style scoped>
.dag-visualization {
  max-width: 100%;
  margin: 0 auto;
}

.canvas-container {
  position: relative;
  width: 100%;
  height: 800px;
  overflow: hidden;
}

svg {
  display: block;
  width: 100%;
  height: 100%;
}

.nodes circle {
  transition: stroke 0.2s, stroke-width 0.2s;
}

.nodes circle:hover {
  filter: brightness(1.1);
}

.edges line {
  transition: stroke 0.2s, stroke-width 0.2s;
}
</style>
