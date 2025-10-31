<template>
	<UContainer class="relative overflow-hidden h-screen">
		<div class="flex flex-col h-full p-6">
			<!-- Header -->
			<div class="mb-6">
				<h1 class="text-3xl font-bold font-heading mb-2">
					MCP Testing UI
				</h1>
				<p class="text-gray-600 dark:text-gray-400">
					Test Model Context Protocol servers via NATS messaging
				</p>
			</div>

			<!-- Primary Tabs: MCP Server Selection -->
			<UTabs v-model="selectedServerIndex" :items="mcpServerTabs" class="flex-1 overflow-hidden">
				<template v-for="(serverTab, idx) in mcpServerTabs" :key="idx" #[serverTab.slot]>
					<!-- Secondary Tabs: Panel Selection for this MCP server -->
					<div class="mt-4 flex flex-col h-full overflow-hidden">
						<UTabs v-model="activePanelTab" :items="panelTabs" class="flex-1 overflow-hidden">
							<template #templates>
								<div class="mt-4 overflow-auto" style="max-height: calc(100vh - 400px)">
									<McpTemplateBrowser
										:server="store.selectedServer"
										@select="handleTemplateSelect"
									/>
								</div>
							</template>

							<template #editor>
								<div class="mt-4 overflow-auto" style="max-height: calc(100vh - 400px)">
									<McpRequestEditor
										:server="store.selectedServer"
										@send="handleSendRequest"
									/>
								</div>
							</template>

							<template #response>
								<div class="mt-4 overflow-auto" style="max-height: calc(100vh - 400px)">
									<McpResponseViewer
										:response="store.currentResponse"
										:loading="store.isLoading"
										:error="store.error"
									/>
								</div>
							</template>

							<template #history>
								<div class="mt-4 overflow-auto" style="max-height: calc(100vh - 400px)">
									<McpRequestHistory
										:server="store.selectedServer"
										@replay="handleReplay"
									/>
								</div>
							</template>
						</UTabs>
					</div>
				</template>
			</UTabs>
		</div>
	</UContainer>
</template>

<script lang="ts" setup>
	import type { CallToolResult, ServerName, TemplateData } from "@/types/mcp";
	import { invoke } from "@tauri-apps/api/core";
	import { computed, ref, watch } from "vue";
	import { useMcpTesterStore } from "@/stores/mcpTester";

	definePageMeta({
		layout: "default",
		name: "MCP Tester",
		description: "Test MCP servers via NATS",
		icon: "i-heroicons-beaker",
		category: "system",
		showInNav: true
	});

	// ============================================================================
	// Store
	// ============================================================================

	const store = useMcpTesterStore();

	// ============================================================================
	// Local State
	// ============================================================================

	const selectedServerIndex = ref(0);
	const activePanelTab = ref(0);

	// ============================================================================
	// MCP Server Tabs Configuration (Primary tabs)
	// ============================================================================

	const mcpServerTabs = computed(() => [
		{ label: "Orchestrator", value: "orchestrator", slot: "orchestrator" },
		{ label: "Story Generator", value: "story-generator", slot: "story-generator" },
		{ label: "Quality Control", value: "quality-control", slot: "quality-control" },
		{ label: "Constraint Enforcer", value: "constraint-enforcer", slot: "constraint-enforcer" },
		{ label: "Prompt Helper", value: "prompt-helper", slot: "prompt-helper" }
	]);

	// ============================================================================
	// Panel Tabs Configuration (Secondary tabs)
	// ============================================================================

	const panelTabs = [
		{ label: "Templates", icon: "i-heroicons-folder", slot: "templates" },
		{ label: "Request Editor", icon: "i-heroicons-pencil-square", slot: "editor" },
		{ label: "Response", icon: "i-heroicons-document-text", slot: "response" },
		{ label: "History", icon: "i-heroicons-clock", slot: "history" }
	];

	// ============================================================================
	// Watchers for Tab Sync
	// ============================================================================

	// Sync tab selection with store
	watch(selectedServerIndex, (newIndex) => {
		const server = mcpServerTabs.value[newIndex]?.value;
		if (server) {
			store.setServer(server as ServerName);
		}
	});

	// Watch store changes to sync tab
	watch(() => store.selectedServer, (newServer) => {
		const index = mcpServerTabs.value.findIndex((tab) => tab.value === newServer);
		if (index !== -1 && index !== selectedServerIndex.value) {
			selectedServerIndex.value = index;
		}
	}, { immediate: true });

	// ============================================================================
	// Event Handlers
	// ============================================================================

	function handleTemplateSelect() {
		// Template is already handled in TemplateBrowser via store
		// Auto-switch to editor tab when template selected
		activePanelTab.value = 1;
	}

	async function handleSendRequest(req: any) {
		const startTime = Date.now();

		// Set loading state in store
		store.setLoading(true);

		// Generate request ID if not present
		const requestId = req.request_id || crypto.randomUUID();

		try {
			// Prepare execution directory for this request
			try {
				await invoke("prepare_execution_directory", {
					requestId
				});
				console.log("[MCP] Execution directory prepared:", requestId);
			} catch (e) {
				console.warn("[MCP] Failed to prepare execution directory:", e);
				// Continue anyway - this is not critical
			}

			// Build the NATS subject based on selected server
			const subject = `mcp.${store.selectedServer}.request`;

			// Save request file before sending
			try {
				await invoke("save_request_file", {
					requestId,
					server: store.selectedServer,
					content: JSON.stringify({
						tool_name: req.tool_name || "unknown",
						arguments: req.arguments || {},
						request_id: requestId,
						timestamp: new Date().toISOString()
					}, null, 2)
				});
				console.log("[MCP] Request file saved:", requestId);
			} catch (e) {
				console.warn("[MCP] Failed to save request file:", e);
				// Continue anyway
			}

			// Call Tauri command to send MCP request
			const result = await invoke<CallToolResult>("send_mcp_request", {
				subject,
				toolName: req.tool_name || "unknown",
				arguments: req.arguments || {},
				tenantId: 1,
				timeoutMs: (req.timeout || 180) * 1000 // Convert seconds to ms
			});

			const durationMs = Date.now() - startTime;

			// Update store with response
			store.setResponse(result);

			// Save response file after successful response
			try {
				await invoke("save_response_file", {
					requestId,
					server: store.selectedServer,
					content: JSON.stringify({
						response: result,
						duration_ms: durationMs,
						success: !result.isError,
						timestamp: new Date().toISOString()
					}, null, 2)
				});
				console.log("[MCP] Response file saved:", requestId);
			} catch (e) {
				console.warn("[MCP] Failed to save response file:", e);
				// Continue anyway
			}

			// Save to history
			try {
				await invoke("save_request_to_history", {
					entry: {
						id: requestId,
						timestamp: new Date().toISOString(),
						server: store.selectedServer,
						tool_name: req.tool_name || "unknown",
						request: req.arguments || {},
						response: result,
						duration_ms: durationMs,
						success: !result.isError,
						tenant_id: 1
					}
				});
			} catch (historyError) {
				console.error("Failed to save to history:", historyError);
				// Don't fail the main request if history save fails
			}

			// Auto-switch to history tab after sending
			activePanelTab.value = 3;

			console.log("MCP request completed successfully", {
				tool: req.tool_name,
				duration: durationMs,
				itemCount: result.content?.length || 0,
				requestId
			});
		} catch (e: any) {
			const durationMs = Date.now() - startTime;
			const errorMessage = e.toString();
			store.setError(errorMessage);

			console.error("MCP request failed:", {
				tool: req.tool_name,
				error: errorMessage,
				duration: durationMs,
				requestId
			});

			// Save error response file
			try {
				const errorResponse: CallToolResult = {
					content: [
						{
							type: "text",
							text: errorMessage
						}
					],
					isError: true
				};

				await invoke("save_response_file", {
					requestId,
					server: store.selectedServer,
					content: JSON.stringify({
						response: errorResponse,
						duration_ms: durationMs,
						success: false,
						error: errorMessage,
						timestamp: new Date().toISOString()
					}, null, 2)
				});
				console.log("[MCP] Error response file saved:", requestId);
			} catch (saveError) {
				console.warn("[MCP] Failed to save error response file:", saveError);
			}

			// Save error to history as well
			try {
				const errorResponse: CallToolResult = {
					content: [
						{
							type: "text",
							text: errorMessage
						}
					],
					isError: true
				};

				await invoke("save_request_to_history", {
					entry: {
						id: requestId,
						timestamp: new Date().toISOString(),
						server: store.selectedServer,
						tool_name: req.tool_name || "unknown",
						request: req.arguments || {},
						response: errorResponse,
						duration_ms: durationMs,
						success: false,
						tenant_id: 1
					}
				});
			} catch (historyError) {
				console.error("Failed to save error to history:", historyError);
			}

			// Auto-switch to history tab even on error
			activePanelTab.value = 3;
		} finally {
			store.setLoading(false);
		}
	}

	function handleReplay(historyEntry: any) {
		// Reconstruct template data from history entry
		const templateData: TemplateData = {
			tool_name: historyEntry.tool_name,
			arguments: historyEntry.request
		};

		// Set the server to match the history entry
		store.setServer(historyEntry.server as ServerName);

		// Update store with request data
		store.setTemplateContent(templateData);

		// Switch to editor tab to show the replayed request
		activePanelTab.value = 1;
	}
</script>
