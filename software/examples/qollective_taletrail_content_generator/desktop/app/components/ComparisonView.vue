<script setup lang="ts">
import type { GenerationResponse } from '~/types/trails'

interface Props {
  originalTrail: GenerationResponse
  newTrail: GenerationResponse
  syncScrolling?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  syncScrolling: false
})

const emit = defineEmits<{
  'update:syncScrolling': [value: boolean]
}>()

// Comparison mode
const comparisonMode = ref<'metadata' | 'content' | 'execution'>('metadata')

/**
 * Metadata comparison
 */
const metadataComparison = computed(() => {
  const orig = props.originalTrail.trail.metadata.generation_params
  const newParams = props.newTrail.trail.metadata.generation_params

  return {
    theme: {
      original: orig.theme,
      new: newParams.theme,
      changed: orig.theme !== newParams.theme
    },
    ageGroup: {
      original: orig.age_group,
      new: newParams.age_group,
      changed: orig.age_group !== newParams.age_group
    },
    language: {
      original: orig.language,
      new: newParams.language,
      changed: orig.language !== newParams.language
    },
    nodeCount: {
      original: orig.node_count,
      new: newParams.node_count,
      changed: orig.node_count !== newParams.node_count,
      diff: newParams.node_count - orig.node_count
    }
  }
})

/**
 * Structure comparison
 */
const structureComparison = computed(() => {
  const origNodes = Object.keys(props.originalTrail.trail.dag.nodes).length
  const newNodes = Object.keys(props.newTrail.trail.dag.nodes).length
  const origEdges = props.originalTrail.trail.dag.edges.length
  const newEdges = props.newTrail.trail.dag.edges.length

  return {
    nodes: {
      original: origNodes,
      new: newNodes,
      diff: newNodes - origNodes
    },
    edges: {
      original: origEdges,
      new: newEdges,
      diff: newEdges - origEdges
    }
  }
})

/**
 * Scroll sync handlers
 */
const originalScrollRef = ref<HTMLElement | null>(null)
const newScrollRef = ref<HTMLElement | null>(null)
let isSyncing = false

function handleScroll(source: 'original' | 'new') {
  if (!props.syncScrolling || isSyncing) return

  isSyncing = true

  if (source === 'original' && originalScrollRef.value && newScrollRef.value) {
    newScrollRef.value.scrollTop = originalScrollRef.value.scrollTop
  } else if (source === 'new' && originalScrollRef.value && newScrollRef.value) {
    originalScrollRef.value.scrollTop = newScrollRef.value.scrollTop
  }

  setTimeout(() => {
    isSyncing = false
  }, 50)
}

function toggleSyncScrolling() {
  emit('update:syncScrolling', !props.syncScrolling)
}
</script>

<template>
  <div class="space-y-6">
    <!-- Header Controls -->
    <div class="flex items-center justify-between">
      <h2 class="text-2xl font-bold">Trail Comparison</h2>

      <div class="flex items-center gap-3">
        <!-- Sync Scrolling Toggle -->
        <UButton
          :variant="syncScrolling ? 'solid' : 'ghost'"
          icon="i-heroicons-arrows-up-down"
          size="sm"
          @click="toggleSyncScrolling"
        >
          {{ syncScrolling ? 'Synced' : 'Sync Scroll' }}
        </UButton>
      </div>
    </div>

    <!-- Comparison Mode Tabs -->
    <UTabs
      v-model="comparisonMode"
      :items="[
        { key: 'metadata', label: 'Metadata', icon: 'i-heroicons-information-circle' },
        { key: 'content', label: 'Content Diff', icon: 'i-heroicons-document-text' },
        { key: 'execution', label: 'Execution Trace', icon: 'i-heroicons-chart-bar' }
      ]"
      class="w-full"
    />

    <!-- Metadata Comparison -->
    <div v-if="comparisonMode === 'metadata'" class="space-y-4">
      <!-- Trail Titles -->
      <UCard>
        <template #header>
          <h3 class="text-lg font-semibold">Trail Information</h3>
        </template>

        <div class="grid grid-cols-2 gap-4">
          <div>
            <p class="text-xs text-gray-500 dark:text-gray-400 mb-2">Original</p>
            <h4 class="text-xl font-bold mb-1">{{ originalTrail.trail.title }}</h4>
            <p class="text-sm text-gray-600 dark:text-gray-400">
              {{ originalTrail.trail.description }}
            </p>
          </div>

          <div>
            <p class="text-xs text-gray-500 dark:text-gray-400 mb-2">New</p>
            <h4 class="text-xl font-bold mb-1">{{ newTrail.trail.title }}</h4>
            <p class="text-sm text-gray-600 dark:text-gray-400">
              {{ newTrail.trail.description }}
            </p>
          </div>
        </div>
      </UCard>

      <!-- Generation Parameters -->
      <UCard>
        <template #header>
          <h3 class="text-lg font-semibold">Generation Parameters</h3>
        </template>

        <div class="space-y-3">
          <!-- Theme -->
          <div class="flex items-center justify-between p-3 rounded-lg" :class="{ 'bg-yellow-50 dark:bg-yellow-950': metadataComparison.theme.changed }">
            <span class="font-medium">Theme</span>
            <div class="flex items-center gap-4">
              <UBadge color="blue" variant="subtle">{{ metadataComparison.theme.original }}</UBadge>
              <UIcon name="i-heroicons-arrow-right" />
              <UBadge color="purple" variant="subtle">{{ metadataComparison.theme.new }}</UBadge>
            </div>
          </div>

          <!-- Age Group -->
          <div class="flex items-center justify-between p-3 rounded-lg" :class="{ 'bg-yellow-50 dark:bg-yellow-950': metadataComparison.ageGroup.changed }">
            <span class="font-medium">Age Group</span>
            <div class="flex items-center gap-4">
              <UBadge color="blue" variant="subtle">{{ metadataComparison.ageGroup.original }}</UBadge>
              <UIcon name="i-heroicons-arrow-right" />
              <UBadge color="purple" variant="subtle">{{ metadataComparison.ageGroup.new }}</UBadge>
            </div>
          </div>

          <!-- Language -->
          <div class="flex items-center justify-between p-3 rounded-lg" :class="{ 'bg-yellow-50 dark:bg-yellow-950': metadataComparison.language.changed }">
            <span class="font-medium">Language</span>
            <div class="flex items-center gap-4">
              <UBadge color="blue" variant="subtle">{{ metadataComparison.language.original }}</UBadge>
              <UIcon name="i-heroicons-arrow-right" />
              <UBadge color="purple" variant="subtle">{{ metadataComparison.language.new }}</UBadge>
            </div>
          </div>

          <!-- Node Count -->
          <div class="flex items-center justify-between p-3 rounded-lg" :class="{ 'bg-yellow-50 dark:bg-yellow-950': metadataComparison.nodeCount.changed }">
            <span class="font-medium">Requested Node Count</span>
            <div class="flex items-center gap-4">
              <UBadge color="blue" variant="subtle">{{ metadataComparison.nodeCount.original }}</UBadge>
              <UIcon name="i-heroicons-arrow-right" />
              <UBadge color="purple" variant="subtle">{{ metadataComparison.nodeCount.new }}</UBadge>
              <UBadge
                v-if="metadataComparison.nodeCount.diff !== 0"
                :color="metadataComparison.nodeCount.diff > 0 ? 'green' : 'red'"
                variant="subtle"
              >
                {{ metadataComparison.nodeCount.diff > 0 ? '+' : '' }}{{ metadataComparison.nodeCount.diff }}
              </UBadge>
            </div>
          </div>
        </div>
      </UCard>

      <!-- Structure Comparison -->
      <UCard>
        <template #header>
          <h3 class="text-lg font-semibold">Trail Structure</h3>
        </template>

        <div class="grid grid-cols-2 gap-4">
          <!-- Nodes -->
          <div class="p-4 bg-gray-50 dark:bg-gray-900 rounded-lg">
            <p class="text-sm text-gray-600 dark:text-gray-400 mb-2">Total Nodes</p>
            <div class="flex items-center gap-3">
              <div>
                <p class="text-2xl font-bold">{{ structureComparison.nodes.original }}</p>
                <p class="text-xs text-gray-500">Original</p>
              </div>
              <UIcon name="i-heroicons-arrow-right" class="text-gray-400" />
              <div>
                <p class="text-2xl font-bold">{{ structureComparison.nodes.new }}</p>
                <p class="text-xs text-gray-500">New</p>
              </div>
            </div>
            <UBadge
              v-if="structureComparison.nodes.diff !== 0"
              :color="structureComparison.nodes.diff > 0 ? 'green' : 'red'"
              variant="subtle"
              class="mt-2"
            >
              {{ structureComparison.nodes.diff > 0 ? '+' : '' }}{{ structureComparison.nodes.diff }} nodes
            </UBadge>
          </div>

          <!-- Edges -->
          <div class="p-4 bg-gray-50 dark:bg-gray-900 rounded-lg">
            <p class="text-sm text-gray-600 dark:text-gray-400 mb-2">Total Edges</p>
            <div class="flex items-center gap-3">
              <div>
                <p class="text-2xl font-bold">{{ structureComparison.edges.original }}</p>
                <p class="text-xs text-gray-500">Original</p>
              </div>
              <UIcon name="i-heroicons-arrow-right" class="text-gray-400" />
              <div>
                <p class="text-2xl font-bold">{{ structureComparison.edges.new }}</p>
                <p class="text-xs text-gray-500">New</p>
              </div>
            </div>
            <UBadge
              v-if="structureComparison.edges.diff !== 0"
              :color="structureComparison.edges.diff > 0 ? 'green' : 'red'"
              variant="subtle"
              class="mt-2"
            >
              {{ structureComparison.edges.diff > 0 ? '+' : '' }}{{ structureComparison.edges.diff }} edges
            </UBadge>
          </div>
        </div>
      </UCard>
    </div>

    <!-- Content Diff -->
    <div v-else-if="comparisonMode === 'content'">
      <ContentDiff
        :original-nodes="originalTrail.trail.dag.nodes"
        :new-nodes="newTrail.trail.dag.nodes"
      />
    </div>

    <!-- Execution Trace Comparison -->
    <div v-else-if="comparisonMode === 'execution'">
      <ExecutionTraceComparison
        :original-trace="originalTrail.service_invocations"
        :new-trace="newTrail.service_invocations"
      />
    </div>
  </div>
</template>
