import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { SubmitGenerationRequest } from '~/types/trails'

export function useRequests() {
  const loading = ref(false)
  const error = ref<string | null>(null)
  const lastRequestId = ref<string | null>(null)

  /**
   * Submit a new generation request
   */
  async function submitRequest(request: SubmitGenerationRequest): Promise<string | null> {
    loading.value = true
    error.value = null

    try {
      console.log('[useRequests] Submitting generation request:', JSON.stringify(request, null, 2))

      const requestId = await invoke<string>('submit_generation_request', {
        request
      })

      console.log('[useRequests] Request submitted successfully:', requestId)
      lastRequestId.value = requestId

      return requestId
    } catch (err) {
      console.error('[useRequests] Failed to submit request:', err)
      console.error('[useRequests] Error type:', typeof err)
      console.error('[useRequests] Error details:', JSON.stringify(err, null, 2))

      // Extract detailed error message
      if (err instanceof Error) {
        error.value = `${err.name}: ${err.message}`
      } else if (typeof err === 'string') {
        error.value = err
      } else {
        error.value = JSON.stringify(err)
      }

      console.error('[useRequests] Error value set to:', error.value)
      return null
    } finally {
      loading.value = false
    }
  }

  /**
   * Replay a generation request with modifications
   */
  async function replayRequest(
    originalRequest: SubmitGenerationRequest,
    newRequestId: string
  ): Promise<string | null> {
    loading.value = true
    error.value = null

    try {
      console.log('[useRequests] Replaying generation request:', {
        originalId: originalRequest.requestId,
        newId: newRequestId
      })

      const resultId = await invoke<string>('replay_generation_request', {
        originalRequest,
        newRequestId
      })

      console.log('[useRequests] Replay request submitted successfully:', resultId)
      lastRequestId.value = resultId

      return resultId
    } catch (err) {
      console.error('[useRequests] Failed to replay request:', err)
      error.value = err instanceof Error ? err.message : String(err)
      return null
    } finally {
      loading.value = false
    }
  }

  /**
   * Generate a unique request ID
   */
  function generateRequestId(): string {
    return `req-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`
  }

  /**
   * Clear error state
   */
  function clearError() {
    error.value = null
  }

  return {
    loading,
    error,
    lastRequestId,
    submitRequest,
    replayRequest,
    generateRequestId,
    clearError
  }
}
