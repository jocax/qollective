<template>
  <UContainer class="relative overflow-hidden h-screen">
    <div class="flex flex-col h-full p-6">
      <div class="mb-6">
        <h1 class="text-3xl font-bold font-heading mb-2">
          Settings
        </h1>
        <p class="text-gray-600 dark:text-gray-400">
          Configure your TaleTrail viewer preferences
        </p>
      </div>

      <!-- Loading state -->
      <LoadingState
        v-if="loading"
        message="Loading settings..."
      />

      <!-- Settings form -->
      <div v-else class="flex-1 overflow-auto">
        <div class="max-w-2xl space-y-6">
          <!-- Display Preferences -->
          <UCard>
            <template #header>
              <h2 class="text-xl font-semibold">
                Display Preferences
              </h2>
            </template>
            <div class="space-y-4 p-4">
              <!-- Default view mode -->
              <div>
                <label class="block text-sm font-medium mb-2">
                  Default View Mode
                </label>
                <USelect
                  v-model="preferences.default_view_mode"
                  :options="viewModeOptions"
                />
                <p class="text-sm text-gray-600 dark:text-gray-500 mt-1">
                  Which view mode to use when opening a trail
                </p>
              </div>

              <!-- Theme -->
              <div>
                <label class="block text-sm font-medium mb-2">
                  Theme
                </label>
                <USelect
                  v-model="preferences.theme"
                  :options="themeOptions"
                />
                <p class="text-sm text-gray-600 dark:text-gray-500 mt-1">
                  Choose your preferred color theme
                </p>
              </div>
            </div>
          </UCard>

          <!-- File Preferences -->
          <UCard>
            <template #header>
              <h2 class="text-xl font-semibold">
                File Preferences
              </h2>
            </template>
            <div class="space-y-4 p-4">
              <!-- Directory path -->
              <div>
                <label class="block text-sm font-medium mb-2">
                  Default Directory
                </label>
                <div class="flex gap-2">
                  <UInput
                    v-model="preferences.directory_path"
                    placeholder="/path/to/trails"
                    class="flex-1"
                  />
                  <UButton @click="selectDirectory" icon="i-heroicons-folder-open">
                    Browse
                  </UButton>
                </div>
                <p class="text-sm text-gray-600 dark:text-gray-500 mt-1">
                  Default location for trail files
                </p>
              </div>

              <!-- Auto-validate -->
              <div class="flex items-center justify-between">
                <div>
                  <label class="block text-sm font-medium">
                    Auto-validate JSON
                  </label>
                  <p class="text-sm text-gray-600 dark:text-gray-500">
                    Validate trail files when loading
                  </p>
                </div>
                <UToggle v-model="preferences.auto_validate" />
              </div>
            </div>
          </UCard>

          <!-- Actions -->
          <div class="flex gap-3">
            <UButton @click="saveSettings" :loading="saving" size="lg">
              Save Settings
            </UButton>
            <UButton @click="resetSettings" variant="ghost" size="lg">
              Reset to Defaults
            </UButton>
          </div>
        </div>
      </div>

      <div class="mt-6">
        <UButton variant="ghost" to="/">
          Back to Trails
        </UButton>
      </div>
    </div>
  </UContainer>
</template>

<script lang="ts" setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

definePageMeta({
  layout: 'default',
  name: 'Settings',
  description: 'Configure viewer preferences',
  icon: 'i-heroicons-cog-6-tooth',
  category: 'other',
  showInNav: true
})

interface UserPreferences {
  default_view_mode: string
  theme: string
  directory_path: string
  auto_validate: boolean
}

const preferences = ref<UserPreferences>({
  default_view_mode: 'Interactive',
  theme: 'System',
  directory_path: '',
  auto_validate: false,
})

const loading = ref(true)
const saving = ref(false)
const { showSuccess, handleError } = useErrorHandling()
const { applyTheme } = useTheme()

const viewModeOptions = [
  { label: 'Interactive Reader', value: 'Interactive' },
  { label: 'Linear Reader', value: 'Linear' },
  { label: 'DAG Visualization', value: 'DAG' },
  { label: 'Execution Trace', value: 'ExecutionTrace' },
]

const themeOptions = [
  { label: 'Light', value: 'Light' },
  { label: 'Dark', value: 'Dark' },
  { label: 'System', value: 'System' },
]

onMounted(async () => {
  await loadSettings()
})

async function loadSettings() {
  try {
    loading.value = true
    // Load preferences with tenant context
    const { selectedTenant } = useTenantContext()
    preferences.value = await invoke<UserPreferences>('load_preferences', {
      app: 'taletrail',
      tenantId: selectedTenant.value
    })
  } catch (e) {
    handleError(e, 'Failed to load preferences')
  } finally {
    loading.value = false
  }
}

async function saveSettings() {
  try {
    saving.value = true
    // Save preferences with tenant context
    const { selectedTenant } = useTenantContext()
    await invoke('save_preferences', {
      app: 'taletrail',
      preferences: preferences.value,
      tenantId: selectedTenant.value
    })

    // Apply theme immediately
    const themeMapping: Record<string, 'light' | 'dark' | 'system'> = {
      'Light': 'light',
      'Dark': 'dark',
      'System': 'system',
    }
    applyTheme(themeMapping[preferences.value.theme] || 'system')

    showSuccess('Settings saved', 'Your preferences have been updated')
  } catch (e) {
    handleError(e, 'Failed to save settings')
  } finally {
    saving.value = false
  }
}

async function selectDirectory() {
  try {
    const selected = await useTauriDialogOpen({
      directory: true,
      multiple: false,
    })

    if (selected) {
      preferences.value.directory_path = selected as string
    }
  } catch (e) {
    handleError(e, 'Failed to select directory')
  }
}

function resetSettings() {
  preferences.value = {
    default_view_mode: 'Interactive',
    theme: 'System',
    directory_path: '',
    auto_validate: false,
  }
  showSuccess('Settings reset', 'Preferences have been reset to defaults')
}
</script>
