<template>
  <div class="execution-trace">
    <!-- Summary Card -->
    <div class="summary-card mb-6">
      <UCard>
        <div class="stats-grid grid grid-cols-3 gap-6">
          <div class="stat text-center">
            <p class="label text-sm text-gray-600 dark:text-gray-400 mb-1">
              Total Invocations
            </p>
            <p class="value text-3xl font-bold text-primary-600 dark:text-primary-400">
              {{ totalInvocations }}
            </p>
          </div>
          <div class="stat text-center">
            <p class="label text-sm text-gray-600 dark:text-gray-400 mb-1">
              Total Duration
            </p>
            <p class="value text-3xl font-bold text-blue-600 dark:text-blue-400">
              {{ formatDuration(totalDuration) }}
            </p>
          </div>
          <div class="stat text-center">
            <p class="label text-sm text-gray-600 dark:text-gray-400 mb-1">
              Success Rate
            </p>
            <p class="value text-3xl font-bold" :class="successRate >= 90 ? 'text-green-600 dark:text-green-400' : 'text-yellow-600 dark:text-yellow-400'">
              {{ successRate }}%
            </p>
          </div>
        </div>
      </UCard>
    </div>

    <!-- Filters Bar -->
    <div class="filters-bar flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-900 rounded-lg mb-6">
      <div class="flex items-center gap-4">
        <div class="flex items-center gap-2">
          <label class="text-sm font-medium text-gray-700 dark:text-gray-300">
            Phase:
          </label>
          <USelectMenu
            v-model="selectedPhase"
            :options="phaseOptions"
            placeholder="All Phases"
            size="sm"
            class="w-48"
          />
        </div>

        <div class="flex items-center gap-2">
          <label class="text-sm font-medium text-gray-700 dark:text-gray-300">
            Sort by:
          </label>
          <USelect
            v-model="sortBy"
            :options="sortOptions"
            size="sm"
            class="w-32"
          />
        </div>

        <div class="flex items-center gap-2">
          <label class="text-sm font-medium text-gray-700 dark:text-gray-300">
            Status:
          </label>
          <USelectMenu
            v-model="statusFilter"
            :options="statusOptions"
            placeholder="All"
            size="sm"
            class="w-32"
          />
        </div>
      </div>

      <div class="text-sm text-gray-600 dark:text-gray-400">
        Showing {{ filteredInvocations.length }} of {{ totalInvocations }} invocations
      </div>
    </div>

    <!-- Empty State -->
    <div v-if="filteredInvocations.length === 0" class="empty-state p-12 text-center">
      <UCard>
        <div class="p-8">
          <svg class="w-16 h-16 mx-auto mb-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
          </svg>
          <h3 class="text-xl font-semibold mb-2">No Service Invocations</h3>
          <p class="text-gray-600 dark:text-gray-400">
            No service invocations match the current filters.
          </p>
        </div>
      </UCard>
    </div>

    <!-- Timeline / Invocations List -->
    <div v-else class="invocations-list space-y-4">
      <UCard
        v-for="(invocation, index) in filteredInvocations"
        :key="index"
        class="invocation-card"
      >
        <div class="invocation-header flex items-center justify-between mb-3">
          <div class="flex items-center gap-3">
            <!-- Success/failure badge -->
            <UBadge :color="invocation.success ? 'green' : 'red'" size="lg">
              <template v-if="invocation.success">
                <svg class="w-4 h-4 inline mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
                Success
              </template>
              <template v-else>
                <svg class="w-4 h-4 inline mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
                Failed
              </template>
            </UBadge>

            <!-- Service name -->
            <h3 class="service-name text-lg font-semibold text-gray-900 dark:text-gray-100">
              {{ invocation.service_name }}
            </h3>
          </div>

          <!-- Phase badge -->
          <UBadge :color="getPhaseColor(invocation.phase)" variant="subtle" size="lg">
            {{ invocation.phase }}
          </UBadge>
        </div>

        <!-- Timing bar -->
        <div class="timing-bar mb-2">
          <div class="flex items-center justify-between mb-1">
            <span class="text-sm text-gray-600 dark:text-gray-400">Duration</span>
            <span class="text-sm font-semibold text-gray-900 dark:text-gray-100">
              {{ formatDuration(invocation.duration_ms) }}
            </span>
          </div>
          <div class="bar-container bg-gray-200 dark:bg-gray-700 rounded-full h-2 overflow-hidden">
            <div
              class="bar h-full rounded-full transition-all duration-300"
              :class="invocation.success ? 'bg-green-500' : 'bg-red-500'"
              :style="{ width: getBarWidth(invocation) }"
            ></div>
          </div>
        </div>

        <!-- Error message (if failed) -->
        <div v-if="!invocation.success && invocation.error_message" class="error-message mt-3">
          <UAlert color="red" title="Error Details" variant="subtle">
            <template #description>
              <p class="text-sm">{{ invocation.error_message }}</p>
            </template>
          </UAlert>
        </div>
      </UCard>
    </div>
  </div>
</template>

<script lang="ts" setup>
import type { GenerationResponse, ServiceInvocation } from '~/types/trails'

interface Props {
  generationResponse: GenerationResponse
}

const props = defineProps<Props>()

// State
const selectedPhase = ref<string | null>(null)
const sortBy = ref<'timestamp' | 'duration'>('timestamp')
const statusFilter = ref<'all' | 'success' | 'failed'>('all')

// Computed properties
const serviceInvocations = computed(() => {
  return props.generationResponse.service_invocations || []
})

const totalInvocations = computed(() => {
  return serviceInvocations.value.length
})

const totalDuration = computed(() => {
  return serviceInvocations.value.reduce((sum, inv) => sum + (inv.duration_ms || 0), 0)
})

const successRate = computed(() => {
  if (totalInvocations.value === 0) return 100
  const successCount = serviceInvocations.value.filter(inv => inv.success).length
  return Math.round((successCount / totalInvocations.value) * 100)
})

const phases = computed(() => {
  const phaseSet = new Set(serviceInvocations.value.map(i => i.phase))
  return Array.from(phaseSet).sort()
})

const phaseOptions = computed(() => {
  return [
    { label: 'All Phases', value: null },
    ...phases.value.map(phase => ({ label: phase, value: phase }))
  ]
})

const sortOptions = [
  { label: 'Timestamp', value: 'timestamp' },
  { label: 'Duration', value: 'duration' }
]

const statusOptions = [
  { label: 'All', value: 'all' },
  { label: 'Success', value: 'success' },
  { label: 'Failed', value: 'failed' }
]

/**
 * Filter and sort invocations based on current filters
 */
const filteredInvocations = computed(() => {
  let invocations = [...serviceInvocations.value]

  // Filter by phase
  if (selectedPhase.value) {
    invocations = invocations.filter(inv => inv.phase === selectedPhase.value)
  }

  // Filter by status
  if (statusFilter.value === 'success') {
    invocations = invocations.filter(inv => inv.success)
  } else if (statusFilter.value === 'failed') {
    invocations = invocations.filter(inv => !inv.success)
  }

  // Sort
  if (sortBy.value === 'duration') {
    invocations.sort((a, b) => (b.duration_ms || 0) - (a.duration_ms || 0))
  }
  // timestamp sort is already in chronological order

  return invocations
})

// Helper functions
function formatDuration(ms: number): string {
  if (ms < 1000) {
    return `${ms}ms`
  }
  const seconds = (ms / 1000).toFixed(2)
  return `${seconds}s`
}

function getBarWidth(invocation: ServiceInvocation): string {
  const maxDuration = Math.max(...filteredInvocations.value.map(i => i.duration_ms || 0))
  if (maxDuration === 0) return '0%'
  const percent = ((invocation.duration_ms || 0) / maxDuration) * 100
  return `${Math.max(5, percent)}%` // Minimum 5% for visibility
}

function getPhaseColor(phase: string): string {
  const colors: Record<string, string> = {
    'planning': 'blue',
    'generation': 'purple',
    'validation': 'yellow',
    'quality-control': 'green',
    'default': 'gray'
  }
  return colors[phase.toLowerCase()] || colors.default
}
</script>

<style scoped>
.execution-trace {
  max-width: 1200px;
  margin: 0 auto;
}

.invocation-card {
  transition: transform 0.2s, box-shadow 0.2s;
}

.invocation-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.bar-container {
  position: relative;
  overflow: hidden;
}
</style>
