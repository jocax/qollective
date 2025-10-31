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
			<UTabs v-model="selectedServerName" :items="mcpServerTabs" class="flex-1 overflow-hidden">
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
										:loading="store.isLoadingResponse"
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

		<!-- Debug Console -->
		<DebugConsole />
	</UContainer>
</template>

<script lang="ts" setup>
	import type { CallToolResult, McpResponseEnvelope, ServerName, TemplateData } from "@/types/mcp";
	import { invoke } from "@tauri-apps/api/core";
	import { computed, nextTick, ref, watch } from "vue";
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

	const selectedServerName = ref<ServerName>("orchestrator");
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
	watch(selectedServerName, (newServerName, oldServerName) => {
		console.log(`[MCP Tester] selectedServerName changed: ${oldServerName} → ${newServerName}`);
		if (newServerName) {
			console.log(`[MCP Tester] Calling store.setServer("${newServerName}")`);
			store.setServer(newServerName);
			console.log(`[MCP Tester] After setServer, store.selectedServer =`, store.selectedServer);
		}
	});

	// Watch store changes to sync tab
	watch(() => store.selectedServer, (newServer, oldServer) => {
		console.log(`[MCP Tester] store.selectedServer changed: ${oldServer} → ${newServer}`);
		if (newServer && newServer !== selectedServerName.value) {
			console.log(`[MCP Tester] Updating selectedServerName: ${selectedServerName.value} → ${newServer}`);
			selectedServerName.value = newServer;
		}
	}, { immediate: true });

	// Watch for panel tab changes
	watch(activePanelTab, (newVal, oldVal) => {
		console.log(`[MCP Tester] Panel tab changed: ${oldVal} → ${newVal}`);
	});

	// ============================================================================
	// Event Handlers
	// ============================================================================

	async function handleTemplateSelect() {
		console.log("[MCP Tester] Template selected, switching to Request Editor");
		console.log("[MCP Tester] Template content:", store.templateContent);
		console.log("[MCP Tester] Current panel tab before switch:", activePanelTab.value);

		// Auto-switch to editor tab when template selected
		await nextTick(() => {
			activePanelTab.value = 1;
			console.log("[MCP Tester] Panel tab switched to Request Editor (index 1)");
			console.log("[MCP Tester] New panel tab value:", activePanelTab.value);
		});
	}

	async function handleSendRequest(req: any) {
		const startTime = Date.now();

		// Set loading state in store
		store.setLoading(true);
		store.setLoadingResponse(true);
		store.setCurrentResponse(null); // Clear previous response

		// Extract data from the full template
		const template = req.template;
		const toolName = template.envelope?.payload?.tool_call?.params?.name || "unknown";
		const toolArguments = template.envelope?.payload?.tool_call?.params?.arguments || {};
		const requestId = template.envelope?.meta?.request_id || crypto.randomUUID();
		const tenantId = Number.parseInt(template.envelope?.meta?.tenant || "1");

		// Build the NATS subject based on selected server (or use from template)
		const subject = template.subject || `mcp.${store.selectedServer}.request`;

		// Extract server name from subject (e.g., "mcp.prompt-helper.request" → "prompt-helper")
		const subjectParts = subject.split(".");
		const targetServer = (subjectParts.length === 3 && subjectParts[0] === "mcp" && subjectParts[2] === "request")
			? subjectParts[1]
			: store.selectedServer; // Fallback to UI tab if subject doesn't match pattern

		// Build proper CallToolRequest for history
		const tool_call_request = {
			method: "tools/call",
			params: {
				name: toolName,
				arguments: toolArguments
			},
			extensions: {}
		};

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

			console.log(`[MCP Tester] Subject: ${subject}, Target Server: ${targetServer}, Selected Server: ${store.selectedServer}`);

			// Save request file before sending
			try {
				await invoke("save_request_file", {
					requestId,
					server: targetServer,
					content: JSON.stringify({
						tool_name: toolName,
						arguments: toolArguments,
						request_id: requestId,
						timestamp: new Date().toISOString()
					}, null, 2)
				});
				console.log("[MCP] Request file saved:", requestId);
			} catch (e) {
				console.warn("[MCP] Failed to save request file:", e);
				// Continue anyway
			}

			// Call Tauri command to send envelope directly - preserves template metadata
			const result = await invoke<McpResponseEnvelope>("send_envelope_direct", {
				subject,
				envelopeJson: template.envelope
			});

			const durationMs = Date.now() - startTime;

			// Update store with response (now the full envelope)
			store.setResponse(result);

			// Save response file after successful response
			try {
				await invoke("save_response_file", {
					requestId,
					server: targetServer,
					content: JSON.stringify({
						response: result, // Full envelope
						duration_ms: durationMs,
						success: !result.payload.tool_response?.isError,
						timestamp: new Date().toISOString()
					}, null, 2)
				});
				console.log("[MCP] Response file saved:", requestId);
			} catch (e) {
				console.warn("[MCP] Failed to save response file:", e);
				// Continue anyway
			}

			// Save to history with correct flat parameters
			// Extract the inner CallToolResult for backward compatibility with history
			try {
				const toolResponse = result.payload.tool_response || {
					content: [],
					isError: false
				};

				await invoke("save_request_to_history", {
					serverName: store.selectedServer,
					tenantId: 1,
					durationMs,
					request: tool_call_request,
					response: toolResponse
				});
			} catch (historyError) {
				console.error("Failed to save to history:", historyError);
				// Don't fail the main request if history save fails
			}

			// Auto-switch to response tab after sending
			console.log("[MCP Tester] Request successful, switching to Response tab");
			console.log("[MCP Tester] Current panel tab before switch:", activePanelTab.value);

			await nextTick(() => {
				activePanelTab.value = 2;
				console.log("[MCP Tester] Panel tab switched to Response (index 2)");
				console.log("[MCP Tester] New panel tab value:", activePanelTab.value);
			});

			console.log("MCP request completed successfully", {
				tool: toolName,
				duration: durationMs,
				itemCount: result.payload.tool_response?.content?.length || 0,
				requestId,
				tenant: result.meta.tenant,
				trace_id: result.meta.tracing?.trace_id
			});
		} catch (e: any) {
			const durationMs = Date.now() - startTime;
			const errorMessage = e.toString();
			store.setError(errorMessage);

			console.error("MCP request failed:", {
				tool: toolName,
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
					server: targetServer,
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

				// Build error request with proper CallToolRequest structure
				const error_tool_call_request = {
					method: "tools/call",
					params: {
						name: toolName,
						arguments: toolArguments
					},
					extensions: {}
				};

				// Save error to history with correct flat parameters
				await invoke("save_request_to_history", {
					serverName: store.selectedServer,
					tenantId: 1,
					durationMs,
					request: error_tool_call_request,
					response: errorResponse
				});
			} catch (historyError) {
				console.error("Failed to save error to history:", historyError);
			}

			// Auto-switch to response tab even on error
			console.log("[MCP Tester] Request failed, switching to Response tab to show error");
			console.log("[MCP Tester] Current panel tab before switch:", activePanelTab.value);

			await nextTick(() => {
				activePanelTab.value = 2;
				console.log("[MCP Tester] Panel tab switched to Response (index 2)");
				console.log("[MCP Tester] New panel tab value:", activePanelTab.value);
			});
		} finally {
			store.setLoading(false);
			store.setLoadingResponse(false);
		}
	}

	function handleReplay(historyEntry: any) {
		// Reconstruct template data from history entry using envelope structure
		const templateData: TemplateData = {
			subject: `mcp.${historyEntry.server}.request`,
			envelope: {
				meta: {
					request_id: crypto.randomUUID(),
					tenant: "1"
				},
				payload: {
					tool_call: {
						method: "tools/call",
						params: {
							name: historyEntry.tool_name,
							arguments: historyEntry.request
						}
					}
				}
			}
		};

		// Set the server to match the history entry
		store.setServer(historyEntry.server as ServerName);

		// Update store with request data
		store.setTemplateContent(templateData);

		// Switch to editor tab to show the replayed request
		activePanelTab.value = 1;
	}
</script>
