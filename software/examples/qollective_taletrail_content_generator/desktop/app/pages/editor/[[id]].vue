<template>
  <UContainer class="relative overflow-hidden min-h-screen">
    <div class="flex flex-col p-6">
      <!-- Loading State -->
      <div v-if="loading" class="flex items-center justify-center p-12">
        <div class="text-center">
          <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-primary-500 mb-4" />
          <p class="text-gray-600 dark:text-gray-400">
            {{ isReplay ? 'Loading original trail...' : 'Preparing editor...' }}
          </p>
        </div>
      </div>

      <!-- Error State -->
      <UAlert
        v-else-if="error"
        color="red"
        variant="subtle"
        icon="i-heroicons-exclamation-triangle"
        title="Error"
        :description="error"
        class="mb-4"
      >
        <template #actions>
          <UButton color="red" variant="ghost" to="/">
            Return to List
          </UButton>
        </template>
      </UAlert>

      <!-- Editor -->
      <div v-else>
        <!-- Original Trail Info (for replays) -->
        <UAlert
          v-if="isReplay && originalTrail"
          color="blue"
          variant="subtle"
          icon="i-heroicons-information-circle"
          title="Replaying Request"
          class="mb-6"
        >
          <p class="text-sm">
            You are creating a replay of "<strong>{{ originalTrail.trail.title }}</strong>".
            Modify the parameters below and submit to generate a new trail for comparison.
          </p>
        </UAlert>

        <!-- Request Editor Component -->
        <RequestEditor
          :initial-request="initialRequest"
          :is-replay="isReplay"
          :original-request-id="originalTrailId"
          @submit="handleSubmit"
          @cancel="handleCancel"
        />

        <!-- Submission Status -->
        <UAlert
          v-if="submitting"
          color="blue"
          variant="subtle"
          icon="i-heroicons-arrow-path"
          title="Submitting Request"
          description="Your generation request is being submitted to the pipeline..."
          class="mt-6"
        />

        <!-- Generation Progress Monitor -->
        <GenerationProgressMonitor
          v-if="showProgressMonitor && generationRequestId"
          :request-id="generationRequestId"
          :show="showProgressMonitor"
          @complete="handleGenerationComplete"
          @cancel="showProgressMonitor = false"
        />

        <UAlert
          v-if="submitError"
          color="red"
          variant="subtle"
          icon="i-heroicons-exclamation-triangle"
          title="Submission Failed"
          :description="submitError"
          class="mt-6"
        >
          <template #actions>
            <UButton color="red" variant="ghost" @click="clearSubmitError">
              Dismiss
            </UButton>
          </template>
        </UAlert>
      </div>
    </div>
  </UContainer>
</template>

<script lang="ts" setup>
import { invoke } from '@tauri-apps/api/core'
import type { GenerationResponse, SubmitGenerationRequest } from '~/types/trails'
import { getTrailFilePath } from '~/utils/trailStorage'
import GenerationProgressMonitor from '~/components/GenerationProgressMonitor.vue'

definePageMeta({
  layout: 'default',
  name: 'Create New Story',
  description: 'Submit new generation request',
  icon: 'i-heroicons-plus-circle',
  category: 'other',
  showInNav: true
})

const route = useRoute()
const router = useRouter()
const { submitRequest, replayRequest } = useRequests()

// Route params
const originalTrailId = computed(() => route.params.id as string | undefined)
const isReplay = computed(() => !!originalTrailId.value)

// State
const loading = ref(true)
const error = ref<string | null>(null)
const originalTrail = ref<GenerationResponse | null>(null)
const initialRequest = ref<Partial<SubmitGenerationRequest> | undefined>(undefined)

// Submission state
const submitting = ref(false)
const submitSuccess = ref(false)
const submitError = ref<string | null>(null)
const lastRequestId = ref<string | null>(null)
const countdown = ref(3)
const showProgressMonitor = ref(false)
const generationRequestId = ref<string | null>(null)

/**
 * Load original trail for replay
 */
async function loadOriginalTrail() {
  if (!originalTrailId.value) {
    loading.value = false
    return
  }

  try {
    loading.value = true
    error.value = null

    console.log('[Editor] Loading original trail for replay:', originalTrailId.value)

    // Get file path from recent trails
    const filePath = getTrailFilePath(originalTrailId.value)

    if (!filePath) {
      throw new Error(`Trail ${originalTrailId.value} not found. Please select it from the trail list first.`)
    }

    // Load full trail data
    originalTrail.value = await invoke<GenerationResponse>('load_trail_full', {
      filePath
    })

    // Extract generation parameters to pre-fill form
    const params = originalTrail.value.trail.metadata.generation_params
    initialRequest.value = {
      theme: params.theme,
      age_group: params.age_group as any,
      language: params.language,
      node_count: params.node_count,
      vocabulary_level: 'moderate', // Default since not in original params
      tenant_id: 'tenant-default'
    }

    console.log('[Editor] Original trail loaded successfully')
  } catch (e) {
    console.error('[Editor] Failed to load original trail:', e)
    error.value = (e as Error).message
  } finally {
    loading.value = false
  }
}

/**
 * Handle request submission
 */
async function handleSubmit(request: SubmitGenerationRequest) {
  submitting.value = true
  submitError.value = null
  submitSuccess.value = false

  try {
    console.log('[Editor] Request to submit:', JSON.stringify(request, null, 2))
    let requestId: string | null

    if (isReplay.value && originalTrailId.value) {
      console.log('[Editor] Submitting replay request')
      requestId = await replayRequest(request, request.request_id)
    } else {
      console.log('[Editor] Submitting new request')
      requestId = await submitRequest(request)
    }

    if (requestId) {
      lastRequestId.value = requestId
      generationRequestId.value = requestId

      // Show progress monitor instead of success alert
      showProgressMonitor.value = true
      submitSuccess.value = false  // Hide the old success alert
    } else {
      const actualError = error.value || 'Unknown error occurred'
      console.error('[Editor] Submit returned null. Error:', actualError)
      submitError.value = `Failed to submit request: ${actualError}`
    }
  } catch (err) {
    console.error('[Editor] Submission error:', err)
    const errorMsg = err instanceof Error ? err.message : String(err)
    submitError.value = `Submission failed: ${errorMsg}`
  } finally {
    submitting.value = false
  }
}

/**
 * Handle generation completion
 */
function handleGenerationComplete(trailId: string) {
  console.log('[Editor] Generation complete, navigating to viewer:', trailId)
  showProgressMonitor.value = false

  // Navigate to the viewer page for the generated trail
  router.push(`/viewer/${trailId}`)
}

/**
 * Handle cancel
 */
function handleCancel() {
  if (isReplay.value && originalTrailId.value) {
    router.push(`/viewer/${originalTrailId.value}`)
  } else {
    router.push('/')
  }
}

/**
 * Clear submit error
 */
function clearSubmitError() {
  submitError.value = null
}

// Load data on mount
onMounted(() => {
  loadOriginalTrail()
})

// Update document title
watchEffect(() => {
  useHead({
    title: isReplay.value
      ? 'Replay Request - TaleTrail Viewer'
      : 'New Request - TaleTrail Viewer'
  })
})
</script>
