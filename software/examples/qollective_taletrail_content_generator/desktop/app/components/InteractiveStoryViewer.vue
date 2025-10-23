<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import type { GenerationResponse, ContentNode, Choice } from '~/types/trails'

interface Props {
  trail: GenerationResponse
}

const props = defineProps<Props>()
const emit = defineEmits<{
  nodeSelected: [nodeId: string]
}>()

// State management
const currentNodeId = ref<string>(props.trail.trail.metadata.start_node_id || '0')
const visitedNodes = ref<Set<string>>(new Set([currentNodeId.value]))
const pathHistory = ref<string[]>([currentNodeId.value])

// Current node
const currentNode = computed<ContentNode | null>(() => {
  const node = props.trail.trail.dag.nodes[currentNodeId.value]
  return node || null
})

// Check if current node is an ending
const isEndingNode = computed(() => {
  return !currentNode.value?.content.choices || currentNode.value.content.choices.length === 0
})

// Total node count
const totalNodes = computed(() => {
  return Object.keys(props.trail.trail.dag.nodes).length
})

// Progress calculation
const progressPercent = computed(() => {
  const visited = visitedNodes.value.size
  const total = totalNodes.value
  return Math.round((visited / total) * 100)
})

// Can go back
const canGoBack = computed(() => {
  return pathHistory.value.length > 1
})

// Handle choice selection
function selectChoice(choice: Choice) {
  const nextNodeId = choice.next_node_id

  // Validate next node exists
  if (!props.trail.trail.dag.nodes[nextNodeId]) {
    console.error('Invalid next node ID:', nextNodeId)
    return
  }

  // Update state
  currentNodeId.value = nextNodeId
  visitedNodes.value.add(nextNodeId)
  pathHistory.value.push(nextNodeId)

  // Emit event for DAG visualization
  emit('nodeSelected', nextNodeId)
}

// Handle back navigation
function goBack() {
  if (canGoBack.value) {
    // Remove current node from history
    pathHistory.value.pop()

    // Get previous node
    const previousNodeId = pathHistory.value[pathHistory.value.length - 1]
    currentNodeId.value = previousNodeId

    // Emit event for DAG visualization
    emit('nodeSelected', previousNodeId)
  }
}

// Handle restart
function restart() {
  const startNodeId = props.trail.trail.metadata.start_node_id || '0'
  currentNodeId.value = startNodeId
  visitedNodes.value = new Set([startNodeId])
  pathHistory.value = [startNodeId]

  // Emit event for DAG visualization
  emit('nodeSelected', startNodeId)
}

// Check if a choice leads to a visited node
function isChoiceVisited(choice: Choice): boolean {
  return visitedNodes.value.has(choice.next_node_id)
}

// Watch for trail changes (in case user loads a different trail)
watch(() => props.trail, () => {
  restart()
}, { deep: true })
</script>

<template>
  <div class="space-y-6">
    <!-- Progress Header -->
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-4">
        <UBadge color="blue" variant="subtle" size="lg">
          <template #leading>
            <UIcon name="i-heroicons-map-pin" />
          </template>
          Node {{ currentNodeId }} of {{ totalNodes }}
        </UBadge>
        <UBadge color="gray" variant="subtle">
          {{ visitedNodes.size }} / {{ totalNodes }} explored ({{ progressPercent }}%)
        </UBadge>
      </div>

      <div class="flex items-center gap-2">
        <UButton
          v-if="canGoBack"
          variant="outline"
          size="sm"
          icon="i-heroicons-arrow-left"
          @click="goBack"
        >
          Back
        </UButton>
        <UButton
          variant="outline"
          size="sm"
          color="gray"
          icon="i-heroicons-arrow-path"
          @click="restart"
        >
          Restart
        </UButton>
      </div>
    </div>

    <!-- Story Content Area -->
    <UCard class="min-h-[300px]">
      <div v-if="currentNode" class="space-y-6">
        <!-- Node Metadata Badge -->
        <div class="flex items-center gap-2">
          <UBadge color="purple" variant="soft" size="xs">
            Node ID: {{ currentNode.id }}
          </UBadge>
          <UBadge
            v-if="isEndingNode"
            color="orange"
            variant="subtle"
            size="xs"
          >
            <template #leading>
              <UIcon name="i-heroicons-flag" />
            </template>
            Ending
          </UBadge>
          <UBadge
            v-if="trail.trail.dag.convergence_points?.includes(currentNode.id)"
            color="yellow"
            variant="subtle"
            size="xs"
          >
            <template #leading>
              <UIcon name="i-heroicons-arrows-pointing-in" />
            </template>
            Convergence Point
          </UBadge>
        </div>

        <!-- Story Text -->
        <div class="prose dark:prose-invert max-w-none">
          <p class="text-lg leading-relaxed text-gray-800 dark:text-gray-200">
            {{ currentNode.content.text }}
          </p>
        </div>

        <!-- Generation Metadata (if available) -->
        <div
          v-if="currentNode.generation_metadata"
          class="text-xs text-gray-500 dark:text-gray-400 flex items-center gap-3 pt-4 border-t border-gray-200 dark:border-gray-700"
        >
          <span v-if="currentNode.generation_metadata.llm_model" class="flex items-center gap-1">
            <UIcon name="i-heroicons-cpu-chip" class="w-3 h-3" />
            {{ currentNode.generation_metadata.llm_model }}
          </span>
          <span v-if="currentNode.generation_metadata.timestamp" class="flex items-center gap-1">
            <UIcon name="i-heroicons-clock" class="w-3 h-3" />
            {{ new Date(currentNode.generation_metadata.timestamp).toLocaleString() }}
          </span>
        </div>
      </div>

      <!-- Error State -->
      <div v-else class="flex items-center justify-center p-12">
        <UAlert
          color="red"
          variant="subtle"
          title="Node Not Found"
          description="The current node could not be found in the story data."
        />
      </div>
    </UCard>

    <!-- Choices Section -->
    <div v-if="!isEndingNode && currentNode" class="space-y-3">
      <h3 class="text-sm font-semibold text-gray-700 dark:text-gray-300 flex items-center gap-2">
        <UIcon name="i-heroicons-arrow-right-circle" class="w-4 h-4" />
        What happens next?
      </h3>

      <div class="grid gap-3">
        <button
          v-for="choice in currentNode.content.choices"
          :key="choice.id"
          :class="[
            'group relative p-4 rounded-lg border-2 text-left transition-all duration-200',
            'hover:border-blue-500 dark:hover:border-blue-400',
            'hover:shadow-md hover:scale-[1.02]',
            'focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2',
            isChoiceVisited(choice)
              ? 'border-blue-300 dark:border-blue-700 bg-blue-50 dark:bg-blue-950'
              : 'border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800'
          ]"
          @click="selectChoice(choice)"
        >
          <div class="flex items-start gap-3">
            <!-- Choice Indicator -->
            <div
              :class="[
                'flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center transition-colors',
                isChoiceVisited(choice)
                  ? 'bg-blue-100 dark:bg-blue-900 text-blue-600 dark:text-blue-400'
                  : 'bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400 group-hover:bg-blue-100 group-hover:dark:bg-blue-900 group-hover:text-blue-600 group-hover:dark:text-blue-400'
              ]"
            >
              <UIcon
                :name="isChoiceVisited(choice) ? 'i-heroicons-check-circle' : 'i-heroicons-arrow-right'"
                class="w-5 h-5"
              />
            </div>

            <!-- Choice Text -->
            <div class="flex-1">
              <p
                :class="[
                  'text-base font-medium',
                  isChoiceVisited(choice)
                    ? 'text-blue-900 dark:text-blue-100'
                    : 'text-gray-800 dark:text-gray-200'
                ]"
              >
                {{ choice.text }}
              </p>

              <!-- Visited Badge -->
              <div v-if="isChoiceVisited(choice)" class="mt-2">
                <UBadge color="blue" variant="soft" size="xs">
                  <template #leading>
                    <UIcon name="i-heroicons-eye" class="w-3 h-3" />
                  </template>
                  Previously explored
                </UBadge>
              </div>
            </div>

            <!-- Arrow Icon -->
            <div class="flex-shrink-0 text-gray-400 dark:text-gray-500 group-hover:text-blue-500 transition-colors">
              <UIcon name="i-heroicons-chevron-right" class="w-5 h-5" />
            </div>
          </div>
        </button>
      </div>
    </div>

    <!-- Ending State -->
    <UCard v-if="isEndingNode" class="bg-gradient-to-r from-orange-50 to-yellow-50 dark:from-orange-950 dark:to-yellow-950 border-orange-200 dark:border-orange-800">
      <div class="text-center space-y-4 p-6">
        <div class="flex justify-center">
          <div class="w-16 h-16 rounded-full bg-orange-100 dark:bg-orange-900 flex items-center justify-center">
            <UIcon name="i-heroicons-flag" class="w-8 h-8 text-orange-600 dark:text-orange-400" />
          </div>
        </div>

        <div>
          <h3 class="text-2xl font-bold text-orange-900 dark:text-orange-100 mb-2">
            Story Complete!
          </h3>
          <p class="text-orange-700 dark:text-orange-300">
            You've reached one of the possible endings.
          </p>
        </div>

        <div class="flex items-center justify-center gap-2 text-sm text-orange-600 dark:text-orange-400">
          <UIcon name="i-heroicons-map" class="w-4 h-4" />
          <span>You explored {{ visitedNodes.size }} out of {{ totalNodes }} nodes ({{ progressPercent }}%)</span>
        </div>

        <div class="pt-4 flex gap-3 justify-center">
          <UButton
            v-if="canGoBack"
            variant="outline"
            color="orange"
            icon="i-heroicons-arrow-left"
            @click="goBack"
          >
            Go Back
          </UButton>
          <UButton
            variant="solid"
            color="orange"
            icon="i-heroicons-arrow-path"
            @click="restart"
          >
            Start Over
          </UButton>
        </div>
      </div>
    </UCard>

    <!-- Path Visualization (Breadcrumb) -->
    <UCard v-if="pathHistory.length > 1" class="bg-gray-50 dark:bg-gray-900">
      <div class="space-y-2">
        <h4 class="text-xs font-semibold text-gray-600 dark:text-gray-400 flex items-center gap-1">
          <UIcon name="i-heroicons-map" class="w-3 h-3" />
          Your Journey
        </h4>
        <div class="flex flex-wrap items-center gap-1">
          <UBadge
            v-for="(nodeId, index) in pathHistory"
            :key="`path-${index}-${nodeId}`"
            :color="nodeId === currentNodeId ? 'blue' : 'gray'"
            :variant="nodeId === currentNodeId ? 'solid' : 'soft'"
            size="xs"
          >
            {{ nodeId }}
          </UBadge>
        </div>
      </div>
    </UCard>
  </div>
</template>

<style scoped>
/* Smooth transitions for choice buttons */
button {
  transform-origin: center;
}

/* Prose styling for story text */
.prose p {
  line-height: 1.8;
  margin-bottom: 1em;
}
</style>
