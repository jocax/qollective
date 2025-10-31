<template>
	<UCard class="flex flex-col h-full">
		<template #header>
			<div class="flex items-center justify-between">
				<h3 class="font-semibold">
					Request Editor
				</h3>
				<div class="flex items-center gap-3">
					<!-- Timeout input inline -->
					<div class="flex items-center gap-2">
						<label class="text-xs text-gray-600 dark:text-gray-400 whitespace-nowrap">
							Timeout (s):
						</label>
						<UInput
							v-model.number="timeout"
							type="number"
							:min="1"
							:max="600"
							size="sm"
							class="w-20"
						/>
					</div>
					<!-- Send button -->
					<UButton
						icon="i-heroicons-paper-airplane"
						size="sm"
						:disabled="!mcpStore.canSend"
						:loading="sending"
						@click="send"
					>
						Send Request
					</UButton>
				</div>
			</div>
		</template>

		<div class="flex flex-col flex-1 overflow-hidden">
			<!-- JSON Editor -->
			<div v-if="mcpStore.templateContent" class="flex-1 flex flex-col overflow-hidden">
				<!-- Tool Name Display -->
				<div class="mb-2 px-2">
					<div class="text-xs text-gray-500">
						Tool
					</div>
					<div class="font-mono text-sm">
						{{ mcpStore.templateContent.tool_name }}
					</div>
				</div>

				<!-- JSON Editor -->
				<div class="flex-1 overflow-hidden">
					<UTextarea
						v-model="localJson"
						placeholder="{&quot;key&quot;: &quot;value&quot;}"
						class="h-full font-mono text-sm"
						:rows="20"
						@input="handleJsonEdit"
					/>
				</div>

				<!-- Validation Errors -->
				<div v-if="validationError" class="mt-2 p-2 bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-300 text-xs rounded">
					<strong>JSON Error:</strong> {{ validationError }}
				</div>
			</div>

			<!-- Empty State -->
			<div v-else class="flex-1 flex items-center justify-center text-center p-8">
				<div>
					<UIcon name="i-heroicons-document-text" class="text-gray-300 text-4xl mb-2" />
					<p class="text-sm text-gray-500">
						Select a template from the browser
					</p>
				</div>
			</div>
		</div>
	</UCard>
</template>

<script lang="ts" setup>
	import type { ServerName } from "@/types/mcp";
	import { ref, watch } from "vue";
	import { useMcpTesterStore } from "@/stores/mcpTester";
	import { MCP_DEFAULT_TIMEOUT_MS } from "~/config/constants";

	const props = defineProps<{
		server: ServerName
	}>();

	const emit = defineEmits<{
		send: [request: { tool_name: string, arguments: Record<string, any>, timeout?: number }]
	}>();

	const mcpStore = useMcpTesterStore();

	// Local state for editor
	const localJson = ref("");
	const validationError = ref<string | null>(null);
	const sending = ref(false);
	const timeout = ref(MCP_DEFAULT_TIMEOUT_MS / 1000); // Convert to seconds

	// Watch store changes and sync to local state
	watch(() => mcpStore.templateContent, (newContent) => {
		if (newContent) {
			localJson.value = JSON.stringify(newContent.arguments, null, 2);
			validationError.value = null;
		} else {
			localJson.value = "";
			validationError.value = null;
		}
	}, { immediate: true, deep: true });

	// Handlers
	function handleJsonEdit() {
		try {
			JSON.parse(localJson.value); // Validate JSON
			validationError.value = null;

			// Update store
			mcpStore.updateRequestJson(localJson.value);
		} catch (e: any) {
			validationError.value = e.message;
		}
	}

	async function send() {
		if (!mcpStore.canSend || !mcpStore.templateContent) {
			return;
		}

		sending.value = true;

		try {
			const requestData = JSON.parse(localJson.value);

			emit("send", {
				tool_name: mcpStore.templateContent.tool_name,
				arguments: requestData,
				timeout: timeout.value
			});
		} catch (e: any) {
			validationError.value = `Failed to send: ${e.message}`;
		} finally {
			sending.value = false;
		}
	}
</script>
