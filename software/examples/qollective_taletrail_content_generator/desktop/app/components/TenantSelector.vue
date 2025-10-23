<script setup lang="ts">
import { computed } from 'vue'
import { getTenantColor, getTenantDisplayName } from '~/utils/tenantColors'

const props = defineProps<{
  modelValue: string | null
  availableTenants: string[]
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string | null]
}>()

// Build options for the select menu
const tenantOptions = computed(() => {
  const options = [
    {
      label: 'All Tenants',
      value: null,
      icon: 'i-heroicons-user-group',
      color: 'gray'
    }
  ]

  props.availableTenants.forEach(tenantId => {
    options.push({
      label: getTenantDisplayName(tenantId),
      value: tenantId,
      icon: 'i-heroicons-user',
      color: getTenantColor(tenantId)
    })
  })

  return options
})

// Find selected option for display
const selectedOption = computed(() => {
  return tenantOptions.value.find(opt => opt.value === props.modelValue) || tenantOptions.value[0]
})

function handleSelect(option: typeof tenantOptions.value[0]) {
  emit('update:modelValue', option.value)
}
</script>

<template>
  <div class="tenant-selector">
    <USelectMenu
      v-model="selectedOption"
      :options="tenantOptions"
      value-attribute="value"
      option-attribute="label"
      class="min-w-[200px]"
      @update:model-value="handleSelect"
    >
      <template #label>
        <div class="flex items-center gap-2">
          <UIcon :name="selectedOption.icon" :class="`text-${selectedOption.color}-500`" />
          <span class="font-medium">{{ selectedOption.label }}</span>
          <UBadge
            v-if="availableTenants.length > 0"
            :color="selectedOption.color"
            variant="subtle"
            size="xs"
          >
            {{ modelValue ? '1' : availableTenants.length }}
          </UBadge>
        </div>
      </template>

      <template #option="{ option }">
        <div class="flex items-center gap-2">
          <UIcon :name="option.icon" :class="`text-${option.color}-500`" />
          <span>{{ option.label }}</span>
          <UBadge
            v-if="option.value === null"
            color="gray"
            variant="subtle"
            size="xs"
          >
            {{ availableTenants.length }}
          </UBadge>
        </div>
      </template>
    </USelectMenu>
  </div>
</template>

<style scoped>
.tenant-selector {
  display: inline-block;
}
</style>
