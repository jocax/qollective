<template>
  <UButton
    :icon="isBookmarked(trail.id) ? 'i-heroicons-star-solid' : 'i-heroicons-star'"
    :color="isBookmarked(trail.id) ? 'yellow' : 'gray'"
    :loading="isLoading"
    :aria-label="isBookmarked(trail.id) ? 'Remove bookmark' : 'Add bookmark'"
    @click.stop="toggleBookmark"
    variant="ghost"
    size="sm"
  />
</template>

<script setup lang="ts">
import { ref } from 'vue'
import type { TrailListItem } from '~/composables/useBookmarks'

const props = defineProps<{
  trail: TrailListItem
}>()

const { isBookmarked, addBookmark, removeBookmark } = useBookmarks()
const { showSuccess, handleError } = useErrorHandling()
const { selectedTenant } = useTenantContext()
const isLoading = ref(false)

async function toggleBookmark() {
  isLoading.value = true

  try {
    if (isBookmarked(props.trail.id)) {
      // Pass tenant context when removing bookmark
      await removeBookmark(props.trail.id, selectedTenant.value)
      showSuccess('Bookmark removed')
    } else {
      // Pass tenant context when adding bookmark
      await addBookmark(props.trail, undefined, selectedTenant.value)
      showSuccess('Bookmark added')
    }
  } catch (e) {
    handleError(e, 'Failed to update bookmark')
  } finally {
    isLoading.value = false
  }
}
</script>
