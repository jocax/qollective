<template>
	<UCard class="flex flex-col h-full">
		<!-- Subject Mismatch Warning Banner -->
		<div v-if="showSubjectMismatch" class="bg-yellow-100 border-l-4 border-yellow-500 text-yellow-700 p-4 mb-4" role="alert">
			<div class="flex">
				<div class="flex-shrink-0">
					<svg class="h-5 w-5 text-yellow-500" viewBox="0 0 20 20" fill="currentColor">
						<path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
					</svg>
				</div>
				<div class="ml-3">
					<p class="text-sm font-medium">
						Subject Mismatch: Template targets <strong>{{ targetServerName }}</strong> but you're on the <strong>{{ currentServerName }}</strong> tab.
					</p>
					<p class="text-xs mt-1">
						The request will be sent to {{ targetServerName }} and files will be saved there.
					</p>
					<!-- DEBUG INFO (remove after testing) -->
					<p class="text-xs mt-1 opacity-50">
						Debug: showSubjectMismatch={{ showSubjectMismatch }}, store.selectedServer={{ mcpStore.selectedServer }}
					</p>
				</div>
			</div>
		</div>

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
						{{ getToolName() }}
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
		send: [request: { template: any, timeout?: number }]
	}>();

	const mcpStore = useMcpTesterStore();

	// Local state for editor
	const localJson = ref("");
	const validationError = ref<string | null>(null);
	const sending = ref(false);
	const timeout = ref(MCP_DEFAULT_TIMEOUT_MS / 1000); // Convert to seconds

	// Subject mismatch detection
	const showSubjectMismatch = ref(false);
	const targetServerName = ref("");
	const currentServerName = ref("");

	// Check for subject mismatch
	const checkSubjectMismatch = () => {
		console.log("[RequestEditor] checkSubjectMismatch() called");
		console.log("[RequestEditor] Current store.selectedServer:", mcpStore.selectedServer);

		try {
			const template = mcpStore.templateContent;
			if (!template) {
				console.log("[RequestEditor] No template, hiding warning");
				showSubjectMismatch.value = false;
				return;
			}

			const subject = template.subject || "";
			console.log("[RequestEditor] Template subject:", subject);
			const subjectParts = subject.split(".");

			if (subjectParts.length === 3 && subjectParts[0] === "mcp" && subjectParts[2] === "request") {
				targetServerName.value = subjectParts[1];
				currentServerName.value = mcpStore.selectedServer;
				showSubjectMismatch.value = targetServerName.value !== currentServerName.value;

				console.log("[RequestEditor] Subject analysis:");
				console.log("  - Target server (from subject):", targetServerName.value);
				console.log("  - Current server (from store):", currentServerName.value);
				console.log("  - Mismatch:", showSubjectMismatch.value);
			} else {
				console.log("[RequestEditor] Subject pattern did not match, hiding warning");
				showSubjectMismatch.value = false;
			}
		} catch (error) {
			console.error("[RequestEditor] Error in checkSubjectMismatch:", error);
			showSubjectMismatch.value = false;
		}
	};

	// Watch store changes and sync to local state
	watch(() => mcpStore.templateContent, (newContent) => {
		if (newContent) {
			const toolName = newContent.envelope?.payload?.tool_call?.params?.name || "unknown";
			console.log("[RequestEditor] Template content updated:", toolName);
			// Show FULL template for editing (subject, envelope with meta and payload)
			localJson.value = JSON.stringify(newContent, null, 2);
			validationError.value = null;
		} else {
			console.log("[RequestEditor] Template content cleared");
			localJson.value = "";
			validationError.value = null;
		}
		// Check for subject mismatch whenever template changes
		checkSubjectMismatch();
	}, { immediate: true, deep: true });

	// Watch for selected server changes to update mismatch detection
	watch(() => mcpStore.selectedServer, (newServer, oldServer) => {
		console.log("[RequestEditor] Watcher triggered: mcpStore.selectedServer changed from", oldServer, "to", newServer);
		checkSubjectMismatch();
	}, { immediate: true });

	// Handlers
	function getToolName() {
		try {
			const parsed = JSON.parse(localJson.value);
			return parsed.envelope?.payload?.tool_call?.params?.name || "unknown";
		} catch {
			return "unknown";
		}
	}

	function handleJsonEdit() {
		try {
			const parsed = JSON.parse(localJson.value); // Validate JSON
			validationError.value = null;

			// Update store with full template
			mcpStore.setTemplateContent(parsed);
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
			const editedTemplate = JSON.parse(localJson.value);

			// Emit the full template for sending
			emit("send", {
				template: editedTemplate,
				timeout: timeout.value
			});
		} catch (e: any) {
			validationError.value = `Failed to send: ${e.message}`;
		} finally {
			sending.value = false;
		}
	}
</script>
