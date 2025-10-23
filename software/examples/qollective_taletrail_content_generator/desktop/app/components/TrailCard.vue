<script setup lang="ts">
import type { TrailListItem } from '~/types/trails'
import { saveRecentTrail } from '~/utils/trailStorage'
import { getTenantColor, getTenantDisplayName } from '~/utils/tenantColors'

interface Props {
  trail: TrailListItem
}

const props = defineProps<Props>()
const emit = defineEmits(['delete'])
const router = useRouter()

const showDeleteConfirm = ref(false)
const deleting = ref(false)

const tenantColor = computed(() => getTenantColor(props.trail.tenantId))
const tenantDisplay = computed(() => getTenantDisplayName(props.trail.tenantId))

const statusColor = computed(() => {
  switch (props.trail.status) {
    case 'completed':
      return 'green'
    case 'failed':
      return 'red'
    case 'partial':
      return 'yellow'
    default:
      return 'gray'
  }
})

const formattedDate = computed(() => {
  try {
    const date = new Date(props.trail.generated_at)
    return date.toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  } catch {
    return props.trail.generated_at
  }
})

const truncatedDescription = computed(() => {
  if (props.trail.description.length > 150) {
    return props.trail.description.substring(0, 150) + '...'
  }
  return props.trail.description
})

const displayTags = computed(() => {
  if (props.trail.tags.length === 0) return []
  return props.trail.tags.slice(0, 3)
})

const remainingTagsCount = computed(() => {
  if (props.trail.tags.length > 3) {
    return props.trail.tags.length - 3
  }
  return 0
})

function handleClick() {
  // Save trail to recent trails before navigating
  console.log('[TrailCard] Saving trail to recent trails:', {
    id: props.trail.id,
    title: props.trail.title,
    file_path: props.trail.file_path
  })
  saveRecentTrail(props.trail)
  router.push(`/viewer/${props.trail.id}`)
}

function openDeleteConfirm() {
  showDeleteConfirm.value = true
}

function cancelDelete() {
  showDeleteConfirm.value = false
}

function confirmDelete() {
  deleting.value = true
  showDeleteConfirm.value = false
  emit('delete', props.trail.id)
}
</script>

<template>
  <div>
    <UCard
      class="cursor-pointer hover:shadow-lg transition-all duration-200 hover:scale-[1.02] relative"
      :class="{ 'opacity-50 pointer-events-none': deleting }"
      @click="handleClick"
    >
      <!-- Action buttons in top-right corner -->
      <div class="absolute top-3 right-3 z-10 flex flex-col gap-2">
        <BookmarkButton :trail="trail" />
        <UButton
          color="red"
          variant="ghost"
          icon="i-heroicons-trash"
          size="sm"
          :loading="deleting"
          @click.stop="openDeleteConfirm"
        />
      </div>

      <div class="space-y-3">
        <!-- Status and Date Row -->
        <div class="flex items-start justify-between pr-8">
          <UBadge :color="statusColor" variant="subtle">
            {{ trail.status }}
          </UBadge>
          <span class="text-xs text-gray-500 dark:text-gray-400">
            {{ formattedDate }}
          </span>
        </div>

        <!-- Title -->
        <div>
          <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 line-clamp-2">
            {{ trail.title }}
          </h3>
        </div>

        <!-- Description -->
        <p class="text-sm text-gray-600 dark:text-gray-400 line-clamp-3">
          {{ truncatedDescription }}
        </p>

        <!-- Metadata Row -->
        <div class="flex flex-wrap gap-2 text-xs">
          <!-- Tenant Badge (if present) -->
          <UBadge
            v-if="trail.tenantId"
            :color="tenantColor"
            variant="soft"
            class="tenant-badge"
          >
            <template #leading>
              <UIcon name="i-heroicons-user" class="w-3 h-3" />
            </template>
            {{ tenantDisplay }}
          </UBadge>
          <UBadge color="blue" variant="soft">
            {{ trail.theme }}
          </UBadge>
          <UBadge color="purple" variant="soft">
            {{ trail.age_group }}
          </UBadge>
          <UBadge color="gray" variant="soft">
            {{ trail.language.toUpperCase() }}
          </UBadge>
          <UBadge color="indigo" variant="soft">
            {{ trail.node_count }} nodes
          </UBadge>
        </div>

        <!-- Tags Row -->
        <div v-if="displayTags.length > 0" class="flex flex-wrap gap-2 pt-2 border-t border-gray-200 dark:border-gray-700">
          <UBadge
            v-for="tag in displayTags"
            :key="tag"
            color="gray"
            variant="outline"
            size="xs"
          >
            {{ tag }}
          </UBadge>
          <UBadge
            v-if="remainingTagsCount > 0"
            color="gray"
            variant="outline"
            size="xs"
          >
            +{{ remainingTagsCount }} more
          </UBadge>
        </div>
      </div>
    </UCard>

    <!-- Delete Confirmation Modal -->
    <Teleport to="body">
      <UModal v-model="showDeleteConfirm">
        <UCard>
          <template #header>
            <div class="flex items-center gap-3">
              <UIcon name="i-heroicons-exclamation-triangle" class="w-6 h-6 text-red-500" />
              <h3 class="text-lg font-semibold">Delete Trail</h3>
            </div>
          </template>

          <p class="text-gray-700 dark:text-gray-300 mb-2">
            Are you sure you want to delete this trail?
          </p>
          <p class="font-semibold text-gray-900 dark:text-gray-100 mb-4">
            "{{ trail.title }}"
          </p>
          <p class="text-sm text-red-600 dark:text-red-400">
            This action cannot be undone.
          </p>

          <template #footer>
            <div class="flex justify-end gap-2">
              <UButton color="gray" variant="ghost" @click="cancelDelete">
                Cancel
              </UButton>
              <UButton color="red" @click="confirmDelete" :loading="deleting">
                Delete
              </UButton>
            </div>
          </template>
        </UCard>
      </UModal>
    </Teleport>
  </div>
</template>

<style scoped>
.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.line-clamp-3 {
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
