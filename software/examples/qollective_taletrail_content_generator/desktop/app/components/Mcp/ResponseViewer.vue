<template>
	<UCard class="flex flex-col h-full">
		<template #header>
			<div class="flex items-center justify-between">
				<h3 class="font-semibold">
					Response Viewer
				</h3>
				<div class="flex items-center gap-2">
					<!-- Verbose Mode Toggle -->
					<UButton
						v-if="response"
						:icon="showVerbose ? 'i-heroicons-eye-slash' : 'i-heroicons-eye'"
						variant="ghost"
						size="xs"
						:title="showVerbose ? 'Hide envelope metadata' : 'Show envelope metadata'"
						@click="showVerbose = !showVerbose"
					/>
					<!-- Copy Button -->
					<UButton
						v-if="response"
						icon="i-heroicons-clipboard-document"
						variant="ghost"
						size="xs"
						title="Copy response to clipboard"
						@click="copyResponse"
					/>
					<!-- Download Button -->
					<UButton
						v-if="response"
						icon="i-heroicons-arrow-down-tray"
						variant="ghost"
						size="xs"
						title="Download response as JSON file"
						@click="downloadResponse"
					/>
				</div>
			</div>
		</template>

		<div class="flex flex-col flex-1 overflow-hidden">
			<!-- Loading State -->
			<div v-if="loading" class="flex items-center justify-center py-8">
				<div class="flex flex-col items-center gap-2">
					<div class="w-8 h-8 border-4 border-primary-500 border-t-transparent rounded-full animate-spin" />
					<p class="text-sm text-gray-500">
						Sending request...
					</p>
				</div>
			</div>

			<!-- Error State -->
			<div v-else-if="error" class="p-4 bg-error-50 dark:bg-error-900/20 border border-error-200 dark:border-error-800 rounded m-4">
				<div class="flex items-start gap-2">
					<UIcon name="i-heroicons-exclamation-triangle" class="w-5 h-5 text-error-500 flex-shrink-0 mt-0.5" />
					<div class="flex-1">
						<p class="font-semibold text-error-700 dark:text-error-400">
							Request Failed
						</p>
						<p class="text-sm text-error-600 dark:text-error-300 mt-1 font-mono">
							{{ error }}
						</p>
					</div>
				</div>
			</div>

			<!-- Success State with Response -->
			<div v-else-if="response" class="flex-1 flex flex-col overflow-hidden">
				<!-- Status Banner -->
				<div
					class="px-4 py-2 border-b" :class="[
						isResponseError ? 'bg-error-50 dark:bg-error-900/20 border-error-200 dark:border-error-800' : 'bg-success-50 dark:bg-success-900/20 border-success-200 dark:border-success-800'
					]"
				>
					<div class="flex items-center gap-2">
						<UIcon
							:name="isResponseError ? 'i-heroicons-x-circle' : 'i-heroicons-check-circle'"
							:class="isResponseError ? 'text-error-500' : 'text-success-500'"
							class="w-5 h-5"
						/>
						<p
							class="text-sm font-medium" :class="[
								isResponseError ? 'text-error-700 dark:text-error-400' : 'text-success-700 dark:text-success-400'
							]"
						>
							{{ isResponseError ? 'Response Error' : 'Response Received' }}
						</p>
						<span class="text-xs text-gray-500 ml-auto">
							{{ contentItemCount }} item{{ contentItemCount !== 1 ? 's' : '' }}
						</span>
					</div>
				</div>

				<!-- Envelope Inspector (Verbose Mode) -->
				<div v-if="showVerbose" class="border-b p-4 bg-gray-50 dark:bg-gray-900">
					<McpEnvelopeInspector :response="response" />
				</div>

				<!-- Response Content -->
				<div class="flex-1 overflow-y-auto p-4 space-y-4">
					<!-- Content Items -->
					<div
						v-for="(item, index) in response.content"
						:key="index"
						class="content-item"
					>
						<McpContentItem :item="item" :index="index" />
					</div>
				</div>
			</div>

			<!-- Empty State -->
			<div v-else class="flex items-center justify-center py-8">
				<div class="text-center text-gray-400">
					<UIcon name="i-heroicons-document-text" class="w-12 h-12 mx-auto mb-2 opacity-50" />
					<p class="text-sm">
						No response yet
					</p>
					<p class="text-xs mt-1">
						Send a request to see the response here
					</p>
				</div>
			</div>
		</div>
	</UCard>
</template>

<script lang="ts" setup>
	import type { CallToolResult } from "@/types/mcp";
	import { computed, ref } from "vue";

	const props = defineProps<{
		response: CallToolResult | null
		loading: boolean
		error: string | null
	}>();

	// State
	const showVerbose = ref(false);

	// Computed
	const isResponseError = computed(() => props.response?.isError || false);
	const contentItemCount = computed(() => props.response?.content?.length || 0);

	// Functions
	function copyResponse() {
		if (!props.response) return;

		try {
			const jsonText = JSON.stringify(props.response, null, 2);
			navigator.clipboard.writeText(jsonText);

			// Show toast notification (if useToast is available)
			// For now, just log
			console.log("Response copied to clipboard");
		} catch (e) {
			console.error("Failed to copy response:", e);
		}
	}

	function downloadResponse() {
		if (!props.response) return;

		try {
			const jsonText = JSON.stringify(props.response, null, 2);
			const blob = new Blob([jsonText], { type: "application/json" });
			const url = URL.createObjectURL(blob);
			const a = document.createElement("a");
			a.href = url;
			a.download = `mcp-response-${Date.now()}.json`;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			URL.revokeObjectURL(url);

			console.log("Response downloaded");
		} catch (e) {
			console.error("Failed to download response:", e);
		}
	}
</script>
