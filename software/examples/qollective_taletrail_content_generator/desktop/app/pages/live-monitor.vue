<template>
  <UContainer class="relative overflow-hidden h-screen">
    <div class="flex flex-col h-full p-6">
      <!-- Header Section -->
      <div class="mb-6">
        <h1 class="text-3xl font-bold font-heading mb-2">
          Live Generation Monitor
        </h1>
        <p class="text-gray-600 dark:text-gray-400">
          Real-time monitoring of TaleTrail generation pipeline via NATS
        </p>
      </div>

      <!-- Toolbar Section -->
      <div class="mb-6 space-y-4">
        <!-- Connection Controls Row -->
        <div class="flex gap-3 flex-wrap">
          <!-- Subscribe Button -->
          <UButton
            v-if="!connectionStatus.subscribed"
            variant="solid"
            color="green"
            size="lg"
            icon="i-heroicons-play"
            :loading="loading"
            @click="handleSubscribe"
          >
            Subscribe
          </UButton>

          <!-- Unsubscribe Button -->
          <UButton
            v-if="connectionStatus.subscribed"
            variant="solid"
            color="red"
            size="lg"
            icon="i-heroicons-stop"
            :loading="loading"
            @click="handleUnsubscribe"
          >
            Unsubscribe
          </UButton>

          <!-- Pause/Resume Button -->
          <UButton
            v-if="connectionStatus.subscribed"
            :variant="isPaused ? 'solid' : 'outline'"
            :color="isPaused ? 'orange' : 'blue'"
            size="lg"
            :icon="isPaused ? 'i-heroicons-play' : 'i-heroicons-pause'"
            @click="handlePauseResume"
          >
            {{ isPaused ? 'Resume' : 'Pause' }}
          </UButton>

          <!-- Cancel Button -->
          <UButton
            v-if="connectionStatus.subscribed"
            variant="outline"
            color="red"
            size="lg"
            icon="i-heroicons-x-mark"
            @click="handleCancel"
          >
            Cancel & Clear
          </UButton>

          <!-- Clear Events Button -->
          <UButton
            v-if="events.length > 0 && !connectionStatus.subscribed"
            variant="ghost"
            size="lg"
            icon="i-heroicons-trash"
            @click="clearAll"
          >
            Clear Events
          </UButton>

          <!-- Back to Home -->
          <UButton
            variant="ghost"
            size="lg"
            icon="i-heroicons-arrow-left"
            to="/"
          >
            Back to Home
          </UButton>
        </div>

        <!-- Filters Row -->
        <div v-if="connectionStatus.subscribed || events.length > 0" class="flex flex-wrap gap-3">
          <!-- Tenant Filter -->
          <USelectMenu
            v-model="selectedTenant"
            :options="tenantOptions"
            placeholder="All Tenants"
            class="w-48"
          >
            <template #leading>
              <UIcon name="i-heroicons-building-office" />
            </template>
          </USelectMenu>

          <!-- Status Filter -->
          <USelectMenu
            v-model="selectedStatus"
            :options="statusOptions"
            placeholder="All Statuses"
            class="w-40"
          >
            <template #leading>
              <UIcon name="i-heroicons-funnel" />
            </template>
          </USelectMenu>

          <!-- Clear Filters -->
          <UButton
            variant="ghost"
            icon="i-heroicons-x-mark"
            @click="handleClearFilters"
          >
            Clear Filters
          </UButton>
        </div>

        <!-- Connection Status Banner -->
        <div class="space-y-2">
          <div class="flex items-center gap-2 text-sm">
            <UBadge
              :color="connectionStatus.connected ? 'green' : 'gray'"
              variant="subtle"
            >
              <template #leading>
                <UIcon
                  :name="connectionStatus.connected ? 'i-heroicons-check-circle' : 'i-heroicons-x-circle'"
                />
              </template>
              {{ connectionStatus.connected ? 'Connected' : 'Disconnected' }}
            </UBadge>

            <UBadge
              v-if="connectionStatus.subscribed"
              color="blue"
              variant="subtle"
            >
              <template #leading>
                <UIcon name="i-heroicons-radio" class="animate-pulse" />
              </template>
              Subscribed
            </UBadge>

            <UBadge
              v-if="isPaused"
              color="orange"
              variant="subtle"
            >
              <template #leading>
                <UIcon name="i-heroicons-pause" />
              </template>
              Paused
            </UBadge>

            <span v-if="connectionStatus.tenantId" class="text-gray-600 dark:text-gray-400 text-xs">
              Tenant: {{ connectionStatus.tenantId }}
            </span>
          </div>

          <!-- Statistics -->
          <div class="grid grid-cols-2 md:grid-cols-4 gap-2 text-xs">
            <div class="bg-gray-100 dark:bg-gray-800 p-2 rounded">
              <div class="text-gray-500 dark:text-gray-400">Total Events</div>
              <div class="text-lg font-semibold text-gray-900 dark:text-gray-100">
                {{ statistics.total }}
              </div>
            </div>
            <div class="bg-blue-50 dark:bg-blue-900/20 p-2 rounded">
              <div class="text-blue-600 dark:text-blue-400">In Progress</div>
              <div class="text-lg font-semibold text-blue-700 dark:text-blue-300">
                {{ statistics.activeRequests }}
              </div>
            </div>
            <div class="bg-green-50 dark:bg-green-900/20 p-2 rounded">
              <div class="text-green-600 dark:text-green-400">Completed</div>
              <div class="text-lg font-semibold text-green-700 dark:text-green-300">
                {{ statistics.completedRequests }}
              </div>
            </div>
            <div class="bg-red-50 dark:bg-red-900/20 p-2 rounded">
              <div class="text-red-600 dark:text-red-400">Failed</div>
              <div class="text-lg font-semibold text-red-700 dark:text-red-300">
                {{ statistics.failedRequests }}
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Content Section with Two Columns -->
      <div class="flex-1 overflow-hidden">
        <!-- Error State -->
        <UAlert
          v-if="error"
          color="red"
          variant="subtle"
          title="Connection Error"
          :description="error"
          class="mb-4"
        />

        <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 h-full">
          <!-- Left Column: Active Requests -->
          <div class="flex flex-col overflow-hidden">
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-3">
              Active Requests
              <UBadge color="blue" variant="subtle" class="ml-2">
                {{ activeRequests.length }}
              </UBadge>
            </h3>
            <div class="flex-1 overflow-y-auto space-y-3 pr-2">
              <!-- Empty State -->
              <div v-if="activeRequests.length === 0" class="flex items-center justify-center h-full">
                <div class="text-center p-8">
                  <UIcon name="i-heroicons-clock" class="w-12 h-12 text-gray-400 mx-auto mb-3" />
                  <h4 class="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-1">
                    No Active Requests
                  </h4>
                  <p class="text-xs text-gray-600 dark:text-gray-400">
                    Waiting for generation requests...
                  </p>
                </div>
              </div>

              <!-- Active Request Cards -->
              <GenerationProgressCard
                v-for="request in activeRequests"
                :key="request.requestId"
                :request="request"
              />
            </div>
          </div>

          <!-- Right Column: Event Stream -->
          <div class="flex flex-col overflow-hidden">
            <LiveMonitor
              :events="filteredEvents"
              :auto-scroll="autoScroll"
            />
          </div>
        </div>

        <!-- Completed/Failed Requests Section (Collapsible) -->
        <div v-if="completedRequests.length > 0 || failedRequests.length > 0" class="mt-4">
          <UAccordion :items="historyAccordionItems">
            <template #completed>
              <div class="space-y-2 p-4">
                <GenerationProgressCard
                  v-for="request in completedRequests"
                  :key="request.requestId"
                  :request="request"
                />
              </div>
            </template>
            <template #failed>
              <div class="space-y-2 p-4">
                <GenerationProgressCard
                  v-for="request in failedRequests"
                  :key="request.requestId"
                  :request="request"
                />
              </div>
            </template>
          </UAccordion>
        </div>
      </div>
    </div>
  </UContainer>
</template>

<script lang="ts" setup>
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification'

definePageMeta({
  layout: 'default',
  name: 'Live Monitor',
  description: 'Real-time NATS generation monitoring',
  icon: 'i-heroicons-radio',
  category: 'monitoring',
  showInNav: true
})

// Use the NATS live monitor composable
const {
  events,
  requests,
  connectionStatus,
  error,
  loading,
  isPaused,
  autoScroll,
  statusFilter,
  tenantFilter,
  filteredEvents,
  activeRequests,
  completedRequests,
  failedRequests,
  statistics,
  uniqueTenantIds,
  subscribe,
  unsubscribe,
  clearAll,
  pause,
  resume,
  cancel,
  setStatusFilter,
  setTenantFilter,
  clearFilters
} = useNatsLiveMonitor()

// Notification permission state
const notificationPermissionGranted = ref(false)

// Selected filters
const selectedTenant = ref<string | null>(null)
const selectedStatus = ref<string>('all')

// Tenant filter options
const tenantOptions = computed(() => {
  return [
    { label: 'All Tenants', value: null },
    ...uniqueTenantIds.value.map(id => ({ label: id, value: id }))
  ]
})

// Status filter options
const statusOptions = [
  { label: 'All Statuses', value: 'all' },
  { label: 'In Progress', value: 'in_progress' },
  { label: 'Completed', value: 'completed' },
  { label: 'Failed', value: 'failed' }
]

// History accordion items
const historyAccordionItems = computed(() => {
  const items = []

  if (completedRequests.value.length > 0) {
    items.push({
      label: `Completed Requests (${completedRequests.value.length})`,
      icon: 'i-heroicons-check-circle',
      slot: 'completed',
      defaultOpen: false
    })
  }

  if (failedRequests.value.length > 0) {
    items.push({
      label: `Failed Requests (${failedRequests.value.length})`,
      icon: 'i-heroicons-x-circle',
      slot: 'failed',
      defaultOpen: false
    })
  }

  return items
})

// Watch filter changes
watch(selectedTenant, (newValue) => {
  setTenantFilter(newValue)
})

watch(selectedStatus, (newValue) => {
  setStatusFilter(newValue)
})

// Watch for completed/failed requests to send notifications
watch(requests, (newRequests) => {
  for (const [requestId, request] of newRequests.entries()) {
    if (request.status === 'completed' || request.status === 'failed') {
      sendGenerationNotification(request.requestId, request.status, request.tenantId)
    }
  }
}, { deep: true })

// Handle subscribe
async function handleSubscribe() {
  const tenantId = selectedTenant.value || undefined
  await subscribe(tenantId)
}

// Handle unsubscribe
async function handleUnsubscribe() {
  await unsubscribe()
}

// Handle pause/resume
function handlePauseResume() {
  if (isPaused.value) {
    resume()
  } else {
    pause()
  }
}

// Handle cancel
async function handleCancel() {
  const confirmed = confirm('Are you sure you want to cancel the subscription and clear all data?')
  if (confirmed) {
    await cancel()
  }
}

// Handle clear filters
function handleClearFilters() {
  selectedTenant.value = null
  selectedStatus.value = 'all'
  clearFilters()
}

// Send desktop notification for generation completion/failure
const notifiedRequests = new Set<string>()

async function sendGenerationNotification(requestId: string, status: string, tenantId: string) {
  // Skip if already notified
  if (notifiedRequests.has(requestId)) return
  notifiedRequests.add(requestId)

  // Check notification permission
  if (!notificationPermissionGranted.value) return

  try {
    const title = status === 'completed' ? 'Generation Complete' : 'Generation Failed'
    const body = `Request ${requestId.substring(0, 8)}... (Tenant: ${tenantId})`

    await sendNotification({
      title,
      body
    })
  } catch (err) {
    console.error('[LiveMonitor] Failed to send notification:', err)
  }
}

// Request notification permission on mount
onMounted(async () => {
  try {
    let granted = await isPermissionGranted()
    if (!granted) {
      const permission = await requestPermission()
      granted = permission === 'granted'
    }
    notificationPermissionGranted.value = granted
    console.log('[LiveMonitor] Notification permission:', granted)
  } catch (err) {
    console.error('[LiveMonitor] Failed to request notification permission:', err)
  }
})
</script>

<style scoped>
/* Custom scrollbar styling for the main container */
.overflow-y-auto {
  scrollbar-width: thin;
  scrollbar-color: rgba(156, 163, 175, 0.5) transparent;
}

.overflow-y-auto::-webkit-scrollbar {
  width: 8px;
}

.overflow-y-auto::-webkit-scrollbar-track {
  background: transparent;
}

.overflow-y-auto::-webkit-scrollbar-thumb {
  background-color: rgba(156, 163, 175, 0.5);
  border-radius: 4px;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover {
  background-color: rgba(156, 163, 175, 0.7);
}
</style>
