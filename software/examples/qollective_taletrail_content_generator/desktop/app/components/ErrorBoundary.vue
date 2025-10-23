<template>
  <div>
    <slot v-if="!error" />

    <div v-else class="error-state p-6">
      <UAlert
        color="red"
        title="Something went wrong"
        :description="error.message"
        class="mb-4"
      >
        <template #actions>
          <UButton @click="reset" size="sm">
            Try Again
          </UButton>
        </template>
      </UAlert>

      <!-- Optional: Show error details in development -->
      <details v-if="showDetails" class="mt-4">
        <summary class="cursor-pointer text-sm text-gray-600 dark:text-gray-400">
          Show error details
        </summary>
        <pre class="mt-2 p-4 bg-gray-100 dark:bg-gray-800 rounded text-xs overflow-auto">{{ error.stack }}</pre>
      </details>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const props = withDefaults(
  defineProps<{
    showDetails?: boolean
  }>(),
  {
    showDetails: false,
  }
)

const error = ref<Error | null>(null)

function reset() {
  error.value = null
}

function setError(e: Error) {
  error.value = e
}

// Expose to parent
defineExpose({ error, setError, reset })
</script>

<style scoped>
.error-state {
  min-height: 200px;
}
</style>
