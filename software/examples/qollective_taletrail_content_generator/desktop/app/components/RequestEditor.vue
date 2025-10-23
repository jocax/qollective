<script setup lang="ts">
import { z } from 'zod'
import type { SubmitGenerationRequest, AgeGroup, VocabularyLevel, StoryStructure, StoryStructureOption } from '~/types/trails'

interface Props {
  initialRequest?: Partial<SubmitGenerationRequest>
  isReplay?: boolean
  originalRequestId?: string
}

const props = defineProps<Props>()

const emit = defineEmits<{
  submit: [request: SubmitGenerationRequest]
  cancel: []
}>()

const { generateRequestId } = useRequests()

// Mode state
const editorMode = ref<'form' | 'json'>('form')

// FIXED: Only 2 languages matching schema (using value/label for USelect)
const languages = [
  { value: 'en', label: 'English' },
  { value: 'de', label: 'German (Deutsch)' }
]

// FIXED: Correct age groups with value/label objects for USelect
const ageGroups = [
  { value: '6-8', label: '6-8 years' },
  { value: '9-11', label: '9-11 years' },
  { value: '12-14', label: '12-14 years' },
  { value: '15-17', label: '15-17 years' },
  { value: '+18', label: '18+ years' }
]

// FIXED: Correct vocabulary levels with value/label objects for USelect
const vocabularyLevels = [
  { value: 'basic', label: 'Basic' },
  { value: 'intermediate', label: 'Intermediate' },
  { value: 'advanced', label: 'Advanced' }
]

// NEW: Story structure presets
const storyStructures: StoryStructureOption[] = [
  {
    value: 'guided',
    label: 'Guided Story',
    description: 'Linear story with occasional choices',
    node_count: 12
  },
  {
    value: 'adventure',
    label: 'Adventure Story',
    description: 'Branching paths with multiple convergence',
    node_count: 16
  },
  {
    value: 'epic',
    label: 'Epic Story',
    description: 'Complex branching that converges at end',
    node_count: 24
  },
  {
    value: 'choose_your_path',
    label: 'Choose Your Path',
    description: 'Pure branching with multiple endings',
    node_count: 16
  }
]

// NEW: Selected preset state (or 'custom')
const selectedPreset = ref<StoryStructure | 'custom'>(
  props.initialRequest?.story_structure || 'guided'
)

// Form state
const formData = ref<SubmitGenerationRequest>({
  request_id: props.initialRequest?.request_id || generateRequestId(),
  tenant_id: props.initialRequest?.tenant_id || '1',
  theme: props.initialRequest?.theme || 'Space Adventure',  // FIXED: provide reasonable default (min 5 chars required)
  age_group: props.initialRequest?.age_group || '6-8',  // FIXED: correct default
  language: props.initialRequest?.language || 'en',
  vocabulary_level: props.initialRequest?.vocabulary_level || 'basic',  // FIXED: correct default
  node_count: props.initialRequest?.node_count || 12,  // FIXED: default to guided preset's node count (backend requires this)
  story_structure: props.initialRequest?.story_structure || 'guided',  // NEW: default to guided
  educational_focus: props.initialRequest?.educational_focus || [],
  constraints: props.initialRequest?.constraints || {},
  metadata: props.initialRequest?.metadata || {
    submitted_at: new Date().toISOString(),
    original_request_id: props.originalRequestId
  }
})

// Watch preset changes to update form data
watch(selectedPreset, (newPreset) => {
  if (newPreset !== 'custom') {
    // Set story_structure and set node_count to preset's value
    formData.value.story_structure = newPreset
    // Set node_count based on preset (backend requires this field)
    const presetConfig = storyStructures.find(p => p.value === newPreset)
    formData.value.node_count = presetConfig?.node_count || 16
  } else {
    // Remove story_structure and use custom node_count
    delete formData.value.story_structure
    if (!formData.value.node_count) {
      formData.value.node_count = 16  // Default for custom
    }
  }
})

// JSON editor state
const jsonText = ref(JSON.stringify(formData.value, null, 2))
const jsonError = ref<string | null>(null)

// Show constraints section
const showConstraints = ref(false)

// Educational focus tag input
const newTag = ref('')

// FIXED: Zod validation schema with correct enums and optional fields
const requestSchema = z.object({
  request_id: z.string().min(1, 'Request ID is required'),
  tenant_id: z.string().min(1, 'Tenant ID is required'),
  theme: z.string().min(5, 'Theme must be at least 5 characters').max(200, 'Theme too long'),
  age_group: z.enum(['6-8', '9-11', '12-14', '15-17', '+18']),  // FIXED
  language: z.enum(['de', 'en']),  // FIXED: only 2 languages
  vocabulary_level: z.enum(['basic', 'intermediate', 'advanced']).optional(),  // FIXED
  story_structure: z.enum(['guided', 'adventure', 'epic', 'choose_your_path']).optional(),  // NEW
  node_count: z.number().min(8, 'Minimum 8 nodes').max(32, 'Maximum 32 nodes').optional(),
  educational_focus: z.array(z.string()).optional(),
  constraints: z.object({
    maxChoicesPerNode: z.number().min(2).max(10).optional(),
    minStoryLength: z.number().min(100).max(10000).optional(),
    forbiddenTopics: z.array(z.string()).optional(),
    requiredTopics: z.array(z.string()).optional()
  }).optional()
})

// Validation errors
const validationErrors = ref<Record<string, string>>({})

/**
 * Validate form data
 */
function validateForm(): boolean {
  validationErrors.value = {}

  try {
    requestSchema.parse(formData.value)
    return true
  } catch (error) {
    if (error instanceof z.ZodError) {
      error.errors.forEach((err) => {
        const path = err.path.join('.')
        validationErrors.value[path] = err.message
      })
    }
    return false
  }
}

/**
 * Add educational focus tag
 */
function addTag() {
  if (newTag.value.trim() && !formData.value.educational_focus?.includes(newTag.value.trim())) {
    if (!formData.value.educational_focus) {
      formData.value.educational_focus = []
    }
    formData.value.educational_focus.push(newTag.value.trim())
    newTag.value = ''
  }
}

/**
 * Remove educational focus tag
 */
function removeTag(index: number) {
  formData.value.educational_focus?.splice(index, 1)
}

/**
 * Toggle editor mode and sync data
 */
function toggleMode(mode: 'form' | 'json') {
  if (mode === 'json' && editorMode.value === 'form') {
    // Form to JSON
    jsonText.value = JSON.stringify(formData.value, null, 2)
    jsonError.value = null
  } else if (mode === 'form' && editorMode.value === 'json') {
    // JSON to Form
    try {
      const parsed = JSON.parse(jsonText.value)
      formData.value = parsed
      jsonError.value = null
    } catch (err) {
      jsonError.value = 'Invalid JSON: ' + (err instanceof Error ? err.message : String(err))
      return
    }
  }
  editorMode.value = mode
}

/**
 * Handle form submission
 */
function handleSubmit() {
  // Sync from JSON if in JSON mode
  if (editorMode.value === 'json') {
    try {
      const parsed = JSON.parse(jsonText.value)
      formData.value = parsed
      jsonError.value = null
    } catch (err) {
      jsonError.value = 'Invalid JSON: ' + (err instanceof Error ? err.message : String(err))
      return
    }
  }

  // Validate
  if (!validateForm()) {
    return
  }

  // Update metadata
  formData.value.metadata = {
    ...formData.value.metadata,
    submittedAt: new Date().toISOString(),
    originalRequestId: props.originalRequestId
  }

  emit('submit', formData.value)
}

/**
 * Handle cancel
 */
function handleCancel() {
  emit('cancel')
}

// Watch JSON text for real-time validation
watch(jsonText, () => {
  if (editorMode.value === 'json') {
    try {
      JSON.parse(jsonText.value)
      jsonError.value = null
    } catch (err) {
      jsonError.value = 'Invalid JSON: ' + (err instanceof Error ? err.message : String(err))
    }
  }
})
</script>

<template>
  <div class="space-y-6">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-2xl font-bold">
          {{ isReplay ? 'Replay Generation Request' : 'New Generation Request' }}
        </h2>
        <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
          {{ isReplay ? 'Modify the request parameters and submit to compare results' : 'Configure parameters for trail generation' }}
        </p>
      </div>

      <!-- Mode Toggle -->
      <div class="flex items-center gap-2">
        <UButton
          :variant="editorMode === 'form' ? 'solid' : 'ghost'"
          icon="i-heroicons-document-text"
          @click="toggleMode('form')"
        >
          Form
        </UButton>
        <UButton
          :variant="editorMode === 'json' ? 'solid' : 'ghost'"
          icon="i-heroicons-code-bracket"
          @click="toggleMode('json')"
        >
          JSON
        </UButton>
      </div>
    </div>

    <!-- Validation Error Summary -->
    <UAlert
      v-if="Object.keys(validationErrors).length > 0"
      color="red"
      variant="subtle"
      icon="i-heroicons-exclamation-triangle"
      title="Validation Errors"
      class="mt-4"
    >
      <template #description>
        <ul class="list-disc list-inside space-y-1">
          <li v-for="(error, field) in validationErrors" :key="field">
            <strong class="capitalize">{{ String(field).replace(/_/g, ' ') }}:</strong> {{ error }}
          </li>
        </ul>
      </template>
    </UAlert>

    <!-- Form Mode -->
    <div v-if="editorMode === 'form'" class="space-y-6">
      <!-- Request Identification -->
      <UCard>
        <template #header>
          <h3 class="text-lg font-semibold">Request Identification</h3>
        </template>

        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium mb-2">Request ID</label>
            <UInput
              v-model="formData.request_id"
              placeholder="req-xxxxx"
              :error="!!validationErrors.request_id"
              disabled
            />
            <p v-if="validationErrors.request_id" class="text-red-500 text-xs mt-1">
              {{ validationErrors.request_id }}
            </p>
          </div>

          <div>
            <label class="block text-sm font-medium mb-2">Tenant ID</label>
            <UInput
              v-model="formData.tenant_id"
              placeholder="tenant-default"
              :error="!!validationErrors.tenant_id"
            />
            <p v-if="validationErrors.tenant_id" class="text-red-500 text-xs mt-1">
              {{ validationErrors.tenant_id }}
            </p>
          </div>
        </div>
      </UCard>

      <!-- Story Structure Presets -->
      <UCard>
        <template #header>
          <h3 class="text-lg font-semibold">Story Structure</h3>
          <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
            Choose a preset structure or customize your own
          </p>
        </template>

        <div class="space-y-4">
          <!-- Preset Options -->
          <div class="grid grid-cols-2 gap-3">
            <UCard
              v-for="preset in storyStructures"
              :key="preset.value"
              :class="{ 'ring-2 ring-primary-500 bg-primary-50 dark:bg-primary-950': selectedPreset === preset.value }"
              class="cursor-pointer hover:shadow-md transition-all"
              @click="selectedPreset = preset.value"
            >
              <div class="flex items-start gap-3">
                <input
                  type="radio"
                  v-model="selectedPreset"
                  :value="preset.value"
                  class="mt-1 h-4 w-4 text-primary-600 border-gray-300 focus:ring-primary-500"
                />
                <div class="flex-1">
                  <h4 class="font-semibold">{{ preset.label }}</h4>
                  <p class="text-xs text-gray-600 dark:text-gray-400 mt-1">
                    {{ preset.description }}
                  </p>
                  <p class="text-xs text-primary-600 dark:text-primary-400 mt-1">
                    {{ preset.node_count }} nodes
                  </p>
                </div>
              </div>
            </UCard>

            <!-- Custom Option -->
            <UCard
              :class="{ 'ring-2 ring-primary-500 bg-primary-50 dark:bg-primary-950': selectedPreset === 'custom' }"
              class="cursor-pointer hover:shadow-md transition-all"
              @click="selectedPreset = 'custom'"
            >
              <div class="flex items-start gap-3">
                <input
                  type="radio"
                  v-model="selectedPreset"
                  value="custom"
                  class="mt-1 h-4 w-4 text-primary-600 border-gray-300 focus:ring-primary-500"
                />
                <div class="flex-1">
                  <h4 class="font-semibold">Custom</h4>
                  <p class="text-xs text-gray-600 dark:text-gray-400 mt-1">
                    Set your own node count
                  </p>
                </div>
              </div>
            </UCard>
          </div>

          <!-- Custom Node Count Slider (only shown when Custom is selected) -->
          <div v-if="selectedPreset === 'custom'" class="pt-4 border-t">
            <label class="block text-sm font-medium mb-2">
              Node Count: {{ formData.node_count || 16 }}
            </label>
            <input
              v-model.number="formData.node_count"
              type="range"
              min="8"
              max="32"
              step="1"
              class="w-full"
            />
            <div class="flex justify-between text-xs text-gray-500 mt-1">
              <span>8 nodes</span>
              <span>32 nodes</span>
            </div>
            <p v-if="validationErrors.node_count" class="text-red-500 text-xs mt-1">
              {{ validationErrors.node_count }}
            </p>
          </div>
        </div>
      </UCard>

      <!-- Story Parameters -->
      <UCard>
        <template #header>
          <h3 class="text-lg font-semibold">Story Parameters</h3>
        </template>

        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium mb-2">Theme *</label>
            <UInput
              v-model="formData.theme"
              placeholder="e.g., Space Adventure, Mystery Island, Medieval Quest"
              :error="!!validationErrors.theme"
            />
            <p v-if="validationErrors.theme" class="text-red-500 text-xs mt-1">
              {{ validationErrors.theme }}
            </p>
          </div>

          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium mb-2">Age Group *</label>
              <USelect
                v-model="formData.age_group"
                :items="ageGroups"
              />
              <p v-if="validationErrors.age_group" class="text-red-500 text-xs mt-1">
                {{ validationErrors.age_group }}
              </p>
            </div>

            <div>
              <label class="block text-sm font-medium mb-2">Vocabulary Level</label>
              <USelect
                v-model="formData.vocabulary_level"
                :items="vocabularyLevels"
              />
              <p v-if="validationErrors.vocabulary_level" class="text-red-500 text-xs mt-1">
                {{ validationErrors.vocabulary_level }}
              </p>
            </div>
          </div>

          <div>
            <label class="block text-sm font-medium mb-2">Language *</label>
            <USelect
              v-model="formData.language"
              :items="languages"
            />
            <p v-if="validationErrors.language" class="text-red-500 text-xs mt-1">
              {{ validationErrors.language }}
            </p>
          </div>

          <div>
            <label class="block text-sm font-medium mb-2">Educational Focus (Optional)</label>
            <div class="flex gap-2 mb-2">
              <UInput
                v-model="newTag"
                placeholder="Add tag (e.g., Math, Science)"
                @keyup.enter="addTag"
              />
              <UButton icon="i-heroicons-plus" @click="addTag">
                Add
              </UButton>
            </div>
            <div v-if="formData.educational_focus && formData.educational_focus.length > 0" class="flex flex-wrap gap-2">
              <UBadge
                v-for="(tag, index) in formData.educational_focus"
                :key="index"
                color="primary"
                variant="subtle"
                class="cursor-pointer"
                @click="removeTag(index)"
              >
                {{ tag }}
                <UIcon name="i-heroicons-x-mark" class="ml-1" />
              </UBadge>
            </div>
          </div>
        </div>
      </UCard>

      <!-- Constraints (Optional) -->
      <UCard>
        <template #header>
          <div class="flex items-center justify-between">
            <h3 class="text-lg font-semibold">Constraints (Optional)</h3>
            <UButton
              variant="ghost"
              size="xs"
              :icon="showConstraints ? 'i-heroicons-chevron-up' : 'i-heroicons-chevron-down'"
              @click="showConstraints = !showConstraints"
            >
              {{ showConstraints ? 'Hide' : 'Show' }}
            </UButton>
          </div>
        </template>

        <div v-if="showConstraints" class="space-y-4">
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium mb-2">Max Choices Per Node</label>
              <UInput
                v-model.number="formData.constraints!.maxChoicesPerNode"
                type="number"
                min="2"
                max="10"
                placeholder="2-10"
              />
            </div>

            <div>
              <label class="block text-sm font-medium mb-2">Min Story Length</label>
              <UInput
                v-model.number="formData.constraints!.minStoryLength"
                type="number"
                min="100"
                max="10000"
                placeholder="100-10000"
              />
            </div>
          </div>
        </div>
      </UCard>
    </div>

    <!-- JSON Mode -->
    <div v-else class="space-y-4">
      <UAlert
        v-if="jsonError"
        color="error"
        variant="subtle"
        icon="i-heroicons-exclamation-triangle"
        title="JSON Error"
        :description="jsonError"
      />

      <div>
        <label class="block text-sm font-medium mb-2">JSON Editor</label>
        <textarea
          v-model="jsonText"
          class="w-full h-96 p-4 font-mono text-sm border rounded-lg dark:bg-gray-800 dark:border-gray-700"
          :class="{ 'border-red-500': jsonError }"
        />
        <p class="text-xs text-gray-500 mt-2">
          Edit the JSON directly. Make sure it's valid JSON before submitting.
        </p>
      </div>
    </div>

    <!-- Actions -->
    <div class="flex items-center justify-end gap-3">
      <UButton
        variant="ghost"
        @click="handleCancel"
      >
        Cancel
      </UButton>
      <UButton
        color="primary"
        icon="i-heroicons-paper-airplane"
        @click="handleSubmit"
      >
        {{ isReplay ? 'Submit Replay' : 'Submit Request' }}
      </UButton>
    </div>
  </div>
</template>
