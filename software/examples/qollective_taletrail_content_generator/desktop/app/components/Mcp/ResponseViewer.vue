<template>
	<UCard class="flex flex-col h-full">
		<template #header>
			<div class="flex items-center justify-between">
				<h3 class="font-semibold">
					Response Viewer
				</h3>
				<div class="flex items-center gap-2">
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
						Waiting for response...
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

			<!-- Response Content -->
			<div v-else-if="response" class="flex-1 overflow-auto p-4">
				<UTextarea
					:model-value="JSON.stringify(response, null, 2)"
					readonly
					class="h-full font-mono text-sm"
					:rows="25"
				/>
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
	import type { McpResponseEnvelope } from "@/types/mcp";

	const props = defineProps<{
		response: McpResponseEnvelope | null
		loading: boolean
		error: string | null
	}>();

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
