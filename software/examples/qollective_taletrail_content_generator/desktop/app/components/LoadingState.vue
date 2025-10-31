<template>
	<div class="loading-state">
		<UIcon name="i-heroicons-arrow-path" class="w-8 h-8 animate-spin text-primary" />
		<p class="text-lg text-gray-600 dark:text-gray-400 mt-4">
			{{ message || 'Loading...' }}
		</p>
		<p v-if="subMessage" class="text-sm text-gray-500 dark:text-gray-500">
			{{ subMessage }}
		</p>
	</div>
</template>

<script setup lang="ts">
	defineProps<{
		message?: string
		subMessage?: string
	}>();
</script>

<style scoped>
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 200px;
  padding: 2rem;
}
</style>
