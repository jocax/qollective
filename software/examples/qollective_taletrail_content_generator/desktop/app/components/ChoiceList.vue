<template>
  <div class="choices-list space-y-3" @keydown="handleKeydown">
    <!-- No choices available (end node) -->
    <div v-if="choices.length === 0" class="text-center p-8">
      <UCard>
        <div class="flex flex-col items-center gap-4">
          <div class="text-gray-400">
            <svg class="w-16 h-16 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </div>
          <div>
            <h3 class="text-xl font-semibold mb-2">The End</h3>
            <p class="text-gray-600 dark:text-gray-400">
              You've reached the end of this story path. Click the Restart button to begin again or Back to explore different choices.
            </p>
          </div>
        </div>
      </UCard>
    </div>

    <!-- Choice buttons -->
    <UButton
      v-for="(choice, index) in choices"
      :key="choice.id"
      :ref="el => setButtonRef(index, el)"
      :variant="getChoiceVariant(choice)"
      :color="getChoiceColor(choice)"
      :disabled="isDisabled(choice)"
      size="lg"
      block
      class="choice-button transition-all duration-200 hover:scale-[1.02]"
      @click="selectChoice(choice)"
    >
      <div class="flex items-center justify-between w-full text-left gap-3">
        <span class="flex-1">{{ choice.text }}</span>
        <div class="flex items-center gap-2 shrink-0">
          <UBadge v-if="isExplored(choice)" color="green" size="xs" variant="subtle">
            Explored
          </UBadge>
          <UBadge v-if="leadsToConvergence(choice)" color="purple" size="xs" variant="subtle">
            <div class="flex items-center gap-1">
              <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
              </svg>
              Convergence
            </div>
          </UBadge>
        </div>
      </div>
    </UButton>

    <!-- Keyboard hint -->
    <div v-if="choices.length > 0" class="text-center text-xs text-gray-500 dark:text-gray-400 mt-4">
      Use arrow keys (↑/↓) to navigate, Enter to select
    </div>
  </div>
</template>

<script lang="ts" setup>
import type { Choice } from '~/types/trails'

interface Props {
  choices: Choice[]
  exploredChoices?: string[]
  disabledChoices?: string[]
  convergencePoints?: string[]
}

const props = withDefaults(defineProps<Props>(), {
  exploredChoices: () => [],
  disabledChoices: () => [],
  convergencePoints: () => []
})

const emit = defineEmits<{
  select: [choice: Choice]
}>()

// Keyboard navigation state
const focusedIndex = ref(0)
const buttonRefs = ref<Record<number, any>>({})

/**
 * Set button ref for keyboard navigation
 */
function setButtonRef(index: number, el: any) {
  if (el) {
    buttonRefs.value[index] = el
  }
}

/**
 * Check if choice has been explored
 */
function isExplored(choice: Choice): boolean {
  return props.exploredChoices.includes(choice.id)
}

/**
 * Check if choice is disabled
 */
function isDisabled(choice: Choice): boolean {
  return props.disabledChoices.includes(choice.id)
}

/**
 * Check if choice leads to convergence point
 */
function leadsToConvergence(choice: Choice): boolean {
  return props.convergencePoints.includes(choice.next_node_id)
}

/**
 * Get variant for choice button based on state
 */
function getChoiceVariant(choice: Choice): 'solid' | 'outline' | 'ghost' {
  if (isExplored(choice)) return 'outline'
  return 'solid'
}

/**
 * Get color for choice button based on state
 */
function getChoiceColor(choice: Choice): string {
  if (leadsToConvergence(choice)) return 'purple'
  if (isExplored(choice)) return 'gray'
  return 'primary'
}

/**
 * Handle choice selection
 */
function selectChoice(choice: Choice) {
  if (!isDisabled(choice)) {
    emit('select', choice)
  }
}

/**
 * Handle keyboard navigation
 */
function handleKeydown(event: KeyboardEvent) {
  if (props.choices.length === 0) return

  switch (event.key) {
    case 'ArrowDown':
    case 'Down':
      event.preventDefault()
      focusedIndex.value = (focusedIndex.value + 1) % props.choices.length
      focusButton(focusedIndex.value)
      break

    case 'ArrowUp':
    case 'Up':
      event.preventDefault()
      focusedIndex.value = focusedIndex.value === 0
        ? props.choices.length - 1
        : focusedIndex.value - 1
      focusButton(focusedIndex.value)
      break

    case 'Enter':
      event.preventDefault()
      const choice = props.choices[focusedIndex.value]
      if (choice && !isDisabled(choice)) {
        selectChoice(choice)
      }
      break

    case 'Home':
      event.preventDefault()
      focusedIndex.value = 0
      focusButton(0)
      break

    case 'End':
      event.preventDefault()
      focusedIndex.value = props.choices.length - 1
      focusButton(props.choices.length - 1)
      break
  }
}

/**
 * Focus a button by index
 */
function focusButton(index: number) {
  const button = buttonRefs.value[index]
  if (button && button.$el) {
    button.$el.focus()
  }
}

/**
 * Reset focus when choices change
 */
watch(() => props.choices, () => {
  focusedIndex.value = 0
}, { immediate: true })
</script>

<style scoped>
.choice-button {
  text-align: left;
}

.choice-button:focus {
  outline: 2px solid currentColor;
  outline-offset: 2px;
}
</style>
