<template>
  <UContainer class="relative overflow-hidden h-screen">
    <div class="flex flex-col h-full p-6">
      <!-- Header Section -->
      <div class="mb-6">
        <div class="flex items-center justify-between mb-4">
          <UButton variant="ghost" icon="i-heroicons-arrow-left" to="/">
            Back to List
          </UButton>
          <div class="flex items-center gap-2">
            <UButton
              v-if="trail"
              color="blue"
              variant="soft"
              icon="i-heroicons-arrow-path"
              :to="`/editor/${trailId}`"
            >
              Replay Request
            </UButton>
          </div>
        </div>

        <!-- Trail Info -->
        <div v-if="trail && !loading" class="space-y-2">
          <h1 class="text-3xl font-bold font-heading">
            {{ trail.trail.title }}
          </h1>
          <p class="text-gray-600 dark:text-gray-400 text-lg">
            {{ trail.trail.description }}
          </p>
          <div class="flex flex-wrap gap-2">
            <UBadge color="blue" variant="subtle" size="lg">
              {{ trail.trail.metadata.generation_params.age_group }}
            </UBadge>
            <UBadge color="green" variant="subtle" size="lg">
              {{ trail.trail.metadata.generation_params.theme }}
            </UBadge>
            <UBadge color="purple" variant="subtle" size="lg">
              {{ trail.trail.metadata.generation_params.language }}
            </UBadge>
            <UBadge color="gray" variant="subtle" size="lg">
              {{ trail.trail.metadata.generation_params.node_count }} nodes
            </UBadge>
          </div>
        </div>
      </div>

      <!-- Mode Switcher Tabs -->
      <div v-if="trail && !loading" class="mb-6">
        <UTabs v-model="currentMode" :items="modeItems" class="w-full" />
      </div>

      <!-- Content Section -->
      <div class="flex-1 overflow-auto">
        <!-- Loading State -->
        <div v-if="loading" class="flex items-center justify-center p-12">
          <div class="text-center">
            <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-primary-500 mb-4"></div>
            <p class="text-gray-600 dark:text-gray-400">Loading trail...</p>
          </div>
        </div>

        <!-- Error State -->
        <UAlert
          v-else-if="error"
          color="red"
          variant="subtle"
          icon="i-heroicons-exclamation-triangle"
          title="Error loading trail"
          :description="error"
          class="mb-4"
        >
          <template #actions>
            <UButton color="red" variant="ghost" to="/">
              Return to List
            </UButton>
          </template>
        </UAlert>

        <!-- Trail Content -->
        <div v-else-if="trail">
          <!-- Interactive Mode -->
          <InteractiveReader
            v-if="currentMode === 'interactive'"
            :trail="trail.trail"
          />

          <!-- Linear Mode -->
          <LinearReader
            v-else-if="currentMode === 'linear'"
            :trail="trail.trail"
          />

          <!-- DAG Mode -->
          <DagVisualization
            v-else-if="currentMode === 'dag'"
            :trail="trail.trail"
          />

          <!-- Execution Trace Mode -->
          <ExecutionTrace
            v-else-if="currentMode === 'trace'"
            :generation-response="trail"
          />
        </div>
      </div>
    </div>
  </UContainer>
</template>

<script lang="ts" setup>
import { invoke } from '@tauri-apps/api/core'
import type { GenerationResponse, TrailListItem } from '~/types/trails'
import { getTrailFilePath, saveRecentTrail } from '~/utils/trailStorage'
import { reconstructDAG } from '~/utils/dagReconstruction'

definePageMeta({
  layout: 'default',
  showInNav: false
})

const route = useRoute()
const trailId = route.params.id as string

// State
const trail = ref<GenerationResponse | null>(null)
const loading = ref(true)
const error = ref<string | null>(null)
const currentMode = ref<'interactive' | 'linear' | 'dag' | 'trace'>('interactive')

// Mode tabs configuration
const modeItems = [
  {
    key: 'interactive',
    label: 'Interactive',
    value: 'interactive',
    icon: 'i-heroicons-cursor-arrow-rays',
    description: 'Choice-based navigation'
  },
  {
    key: 'linear',
    label: 'Linear',
    value: 'linear',
    icon: 'i-heroicons-document-text',
    description: 'Read all paths sequentially'
  },
  {
    key: 'dag',
    label: 'DAG View',
    value: 'dag',
    icon: 'i-heroicons-chart-bar',
    description: 'Story graph visualization'
  },
  {
    key: 'trace',
    label: 'Execution Trace',
    value: 'trace',
    icon: 'i-heroicons-clipboard-document-list',
    description: 'Generation pipeline details'
  }
]

/**
 * Load trail data from file with fallback to directory search
 */
async function loadTrail() {
  try {
    loading.value = true
    error.value = null

    console.log('[Viewer] Loading trail with ID:', trailId)

    // Try to get file path from recent trails first
    let filePath = getTrailFilePath(trailId)

    console.log('[Viewer] Retrieved file path from recent trails:', filePath)

    // Fallback: If not in recent trails, search the directory
    if (!filePath) {
      console.log('[Viewer] Trail not in recent cache, searching directory...')

      try {
        // Load all trails from directory to find the one we need
        const allTrails = await invoke<TrailListItem[]>('load_trails_from_directory', {
          directory: '/Users/ms/development/qollective/software/examples/qollective_taletrail_content_generator/desktop/test-trails'
        })

        console.log('[Viewer] Loaded trails from directory:', allTrails.length)

        // Find the trail by ID
        const trailItem = allTrails.find(t => t.id === trailId)

        if (trailItem) {
          filePath = trailItem.file_path
          console.log('[Viewer] Found trail in directory:', filePath)

          // Save to recent trails for future use
          saveRecentTrail(trailItem)
        } else {
          throw new Error(`Trail with ID "${trailId}" not found in the trails directory. Available trails: ${allTrails.length}`)
        }
      } catch (dirErr) {
        console.error('[Viewer] Failed to search directory:', dirErr)
        throw new Error(`Trail "${trailId}" not found. Unable to search directory: ${(dirErr as Error).message}`)
      }
    }

    if (!filePath) {
      throw new Error(`Unable to locate trail file for ID: ${trailId}`)
    }

    console.log('[Viewer] Loading trail data from file:', filePath)

    // Load full trail data using Tauri command
    trail.value = await invoke<GenerationResponse>('load_trail_full', {
      filePath
    })

    // Validate trail data
    if (!trail.value?.trail) {
      throw new Error('Invalid trail data: missing trail object')
    }

    // Check for either DAG or trail_steps
    const hasDAG = trail.value.trail.dag !== undefined && trail.value.trail.dag !== null
    const hasTrailSteps = trail.value.trail_steps && trail.value.trail_steps.length > 0

    console.log('[Viewer] Trail data format check:', {
      hasDAG,
      hasTrailSteps,
      stepCount: trail.value.trail_steps?.length || 0
    })

    if (!hasDAG && !hasTrailSteps) {
      throw new Error(
        'Invalid trail data: missing both DAG structure and trail_steps. ' +
        'This trail was generated with an older version of the orchestrator. ' +
        'Please regenerate the trail with the updated system.'
      )
    }

    // Reconstruct DAG from trail_steps if needed
    if (!hasDAG && hasTrailSteps) {
      const start_node_id = trail.value.trail.metadata.start_node_id ||
                           trail.value.trail.metadata.generation_params?.start_node_id

      if (!start_node_id) {
        throw new Error('Invalid trail data: missing start node ID for reconstruction')
      }

      console.log('[Viewer] Reconstructing DAG from trail_steps...', {
        stepCount: trail.value.trail_steps!.length,
        startNodeId: start_node_id
      })

      try {
        trail.value.trail.dag = reconstructDAG(
          trail.value.trail_steps!,
          start_node_id
        )
        console.log('[Viewer] Successfully reconstructed DAG from trail_steps:', {
          nodeCount: Object.keys(trail.value.trail.dag.nodes).length,
          edgeCount: trail.value.trail.dag.edges.length,
          convergencePoints: trail.value.trail.dag.convergence_points?.length || 0
        })
      } catch (reconstructErr) {
        console.error('[Viewer] DAG reconstruction failed:', reconstructErr)
        throw new Error(
          `Failed to reconstruct trail structure from trail_steps. ` +
          `This may indicate missing or invalid choice data (next_node_id fields). ` +
          `Error: ${(reconstructErr as Error).message}. ` +
          `Please check the console for detailed validation warnings.`
        )
      }
    }

    // Now validate DAG structure exists
    if (!trail.value.trail.dag) {
      throw new Error('Invalid trail data: DAG reconstruction failed')
    }

    if (!trail.value.trail.metadata?.start_node_id) {
      throw new Error('Invalid trail data: missing start node ID')
    }

    console.log('[Viewer] Trail loaded successfully:', {
      title: trail.value.trail.title,
      nodeCount: Object.keys(trail.value.trail.dag.nodes).length,
      edgeCount: trail.value.trail.dag.edges.length,
      startNode: trail.value.trail.metadata.start_node_id,
      filePath
    })

  } catch (e) {
    console.error('[Viewer] Failed to load trail:', e)
    error.value = (e as Error).message
  } finally {
    loading.value = false
  }
}

// Load trail on mount
onMounted(() => {
  loadTrail()
})

// Update document title
watchEffect(() => {
  if (trail.value?.trail.title) {
    useHead({
      title: `${trail.value.trail.title} - TaleTrail Viewer`
    })
  }
})
</script>
