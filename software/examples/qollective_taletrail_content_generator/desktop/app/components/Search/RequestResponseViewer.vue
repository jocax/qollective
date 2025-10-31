<template>
	<UCard class="flex-1 flex flex-col overflow-hidden">
		<!-- Loading State -->
		<div v-if="loading" class="flex items-center justify-center h-full">
			<div class="text-center p-8">
				<div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-primary-500 mb-3" />
				<p class="text-sm text-gray-600 dark:text-gray-400">
					Loading request/response files...
				</p>
			</div>
		</div>

		<!-- Empty State -->
		<div v-else-if="!requestContent && !responseContent" class="flex items-center justify-center h-full">
			<div class="text-center p-8">
				<UIcon name="i-heroicons-document-text" class="w-12 h-12 text-gray-400 mx-auto mb-3" />
				<h4 class="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-1">
					No File Selected
				</h4>
				<p class="text-xs text-gray-600 dark:text-gray-400">
					Select a request/response file from the left to view details
				</p>
			</div>
		</div>

		<!-- Two-Column Layout: Request | Response -->
		<div v-else class="flex-1 flex flex-col overflow-hidden">
			<!-- Headers Row -->
			<div class="grid grid-cols-2 gap-4 mb-3 pb-3 border-b border-gray-200 dark:border-gray-700">
				<div class="flex items-center justify-between">
					<h4 class="text-sm font-semibold text-gray-900 dark:text-gray-100 uppercase tracking-wide">
						Request
					</h4>
					<UButton
						variant="ghost"
						size="xs"
						icon="i-heroicons-clipboard-document"
						@click="handleCopyRequest"
					>
						Copy
					</UButton>
				</div>
				<div class="flex items-center justify-between">
					<h4 class="text-sm font-semibold text-gray-900 dark:text-gray-100 uppercase tracking-wide">
						Response
					</h4>
					<UButton
						variant="ghost"
						size="xs"
						icon="i-heroicons-clipboard-document"
						@click="handleCopyResponse"
					>
						Copy
					</UButton>
				</div>
			</div>

			<!-- Content Row -->
			<div class="flex-1 grid grid-cols-2 gap-4 overflow-hidden">
				<!-- Request JSON Column -->
				<div class="overflow-y-auto">
					<pre
						class="text-xs font-mono bg-gray-50 dark:bg-gray-900 p-4 rounded-lg border border-gray-200 dark:border-gray-700 overflow-x-auto"
					><code class="language-json text-gray-800 dark:text-gray-200">{{ requestContent || '{}' }}</code></pre>
				</div>

				<!-- Response JSON Column -->
				<div class="overflow-y-auto">
					<pre
						class="text-xs font-mono bg-gray-50 dark:bg-gray-900 p-4 rounded-lg border border-gray-200 dark:border-gray-700 overflow-x-auto"
					><code class="language-json text-gray-800 dark:text-gray-200">{{ responseContent || '{}' }}</code></pre>
				</div>
			</div>
		</div>
	</UCard>
</template>

<script lang="ts" setup>
	defineProps<{
		requestContent: string
		responseContent: string
		loading: boolean
	}>();

	const emit = defineEmits<{
		copyRequest: []
		copyResponse: []
	}>();

	/**
	 * Handle copy request button click
	 */
	function handleCopyRequest() {
		emit("copyRequest");
	}

	/**
	 * Handle copy response button click
	 */
	function handleCopyResponse() {
		emit("copyResponse");
	}
</script>

<style scoped>
/* Custom scrollbar styling for JSON panels */
.overflow-y-auto {
	scrollbar-width: thin;
	scrollbar-color: rgba(156, 163, 175, 0.5) transparent;
}

.overflow-y-auto::-webkit-scrollbar {
	width: 6px;
}

.overflow-y-auto::-webkit-scrollbar-track {
	background: transparent;
}

.overflow-y-auto::-webkit-scrollbar-thumb {
	background-color: rgba(156, 163, 175, 0.5);
	border-radius: 3px;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover {
	background-color: rgba(156, 163, 175, 0.7);
}

/* JSON syntax highlighting (basic) */
pre {
	margin: 0;
	white-space: pre-wrap;
	word-wrap: break-word;
}

code {
	display: block;
}
</style>
