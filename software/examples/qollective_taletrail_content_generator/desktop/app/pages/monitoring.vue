<template>
	<UContainer class="relative overflow-hidden h-screen">
		<div class="flex flex-col h-full p-6">
			<!-- Error Banner -->
			<UAlert
				v-if="connectionError"
				color="red"
				variant="solid"
				title="Connection Error"
				:description="connectionError"
				class="mb-4"
			>
				<template #actions>
					<UButton
						color="white"
						variant="ghost"
						icon="i-heroicons-arrow-path"
						:loading="reconnecting"
						@click="manualReconnect"
					>
						Reconnect
					</UButton>
				</template>
			</UAlert>

			<!-- Header Section -->
			<div class="mb-6">
				<h1 class="text-3xl font-bold font-heading mb-2">
					MONITORING - Raw NATS Messages
				</h1>
				<p class="text-gray-600 dark:text-gray-400">
					Live feed of NATS messages from all microservices
				</p>
			</div>

			<!-- Filter Section -->
			<div class="mb-4 space-y-3">
				<div class="flex gap-3">
					<!-- Endpoint Selector -->
					<div class="flex items-center gap-2">
						<UIcon name="i-heroicons-funnel" class="w-4 h-4 text-gray-500" />
						<USelectMenu
							v-model="selectedEndpoint"
							:options="endpointOptions"
							placeholder="Filter by endpoint"
							class="w-56"
						/>
					</div>

					<!-- Filter Input -->
					<UInput
						v-model="filterText"
						placeholder="Filter by text or request-id (wildcard search)"
						icon="i-heroicons-magnifying-glass"
						class="flex-1"
					/>
				</div>

				<!-- Info Bar -->
				<div class="flex items-center justify-between text-sm text-gray-600 dark:text-gray-400">
					<div class="flex items-center gap-3">
						<span>
							Showing {{ filteredMessages.length }} of {{ messages.length }} messages
						</span>
						<UBadge
							v-if="selectedEndpoint !== 'all'"
							:color="getEndpointColor(selectedEndpoint)"
							variant="subtle"
						>
							{{ selectedEndpointLabel }}
						</UBadge>
					</div>
					<div class="flex items-center gap-2">
						<span
							class="w-2 h-2 rounded-full" :class="[
								connected ? 'bg-green-500 animate-pulse' : 'bg-red-500'
							]"
						/>
						<span>{{ connected ? "Connected" : "Disconnected" }}</span>
					</div>
				</div>
			</div>

			<!-- Diagnostics Panel -->
			<UCard v-if="connected" class="mb-4">
				<template #header>
					<div class="flex items-center justify-between">
						<div class="flex items-center gap-2">
							<UIcon name="i-heroicons-chart-bar" class="w-5 h-5" />
							<h3 class="text-lg font-semibold">
								Monitoring Diagnostics
							</h3>
						</div>
						<!-- Activity Indicator -->
						<div class="flex items-center gap-2">
							<span
								class="w-3 h-3 rounded-full" :class="{
									'bg-green-500 animate-pulse': activityStatus === 'active',
									'bg-orange-500': activityStatus === 'warning',
									'bg-red-500': activityStatus === 'error'
								}"
							/>
							<span class="text-sm font-medium">
								{{ activityStatus === 'active' ? 'Live' : activityStatus === 'warning' ? 'Idle' : 'Stale' }}
							</span>
						</div>
					</div>
				</template>

				<div class="grid grid-cols-2 md:grid-cols-4 gap-4">
					<div>
						<div class="text-xs text-gray-500 dark:text-gray-400 mb-1">
							Messages Received
						</div>
						<div class="text-2xl font-mono font-bold">
							{{ diagnostics.received }}
						</div>
					</div>
					<div>
						<div class="text-xs text-gray-500 dark:text-gray-400 mb-1">
							Messages Displayed
						</div>
						<div class="text-2xl font-mono font-bold">
							{{ diagnostics.emitted }}
						</div>
					</div>
					<div>
						<div class="text-xs text-gray-500 dark:text-gray-400 mb-1">
							Emission Failures
						</div>
						<div class="text-2xl font-mono font-bold" :class="diagnostics.failures > 0 ? 'text-red-500' : ''">
							{{ diagnostics.failures }}
						</div>
					</div>
					<div>
						<div class="text-xs text-gray-500 dark:text-gray-400 mb-1">
							Message Rate
						</div>
						<div class="text-2xl font-mono font-bold">
							{{ messageRate }}/s
						</div>
					</div>
				</div>

				<div class="grid grid-cols-2 gap-4 mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
					<div>
						<div class="text-xs text-gray-500 dark:text-gray-400 mb-1">
							Last Message
						</div>
						<div class="text-sm font-medium">
							{{ formatTimestamp(diagnostics.lastMessage) }}
						</div>
					</div>
					<div>
						<div class="text-xs text-gray-500 dark:text-gray-400 mb-1">
							Connection Time
						</div>
						<div class="text-sm font-medium">
							{{ formatDuration(connectionDuration) }}
						</div>
					</div>
				</div>
			</UCard>

			<!-- Message Feed Section -->
			<div class="flex-1 overflow-hidden flex flex-col">
				<UCard class="flex-1 overflow-hidden flex flex-col">
					<template #header>
						<div class="flex items-center justify-between">
							<h3 class="text-lg font-semibold">
								NATS Messages (Live Feed)
							</h3>
							<div class="flex items-center gap-2">
								<span class="text-sm text-gray-600 dark:text-gray-400">
									Max: {{ MAX_MESSAGES }} messages
								</span>
							</div>
						</div>
					</template>

					<!-- Messages Container -->
					<div
						ref="messageContainer"
						class="flex-1 overflow-y-auto space-y-2 pr-2"
						style="max-height: calc(100vh - 350px)"
					>
						<!-- Empty State -->
						<div
							v-if="filteredMessages.length === 0"
							class="flex items-center justify-center h-full"
						>
							<div class="text-center p-8 max-w-2xl">
								<UIcon
									name="i-heroicons-signal-slash"
									class="w-16 h-16 mx-auto mb-4 text-gray-400"
								/>
								<h3 class="text-lg font-semibold mb-2">
									{{ messages.length === 0 ? "No Messages Yet" : "No Messages Match Filters" }}
								</h3>

								<!-- Connected but no messages -->
								<div v-if="messages.length === 0 && connected" class="space-y-4">
									<div class="text-sm text-gray-600 dark:text-gray-400">
										<p class="mb-2">
											Listening for messages on:
										</p>
										<div class="flex gap-2 justify-center font-mono text-xs">
											<UBadge color="blue" variant="subtle">
												mcp.&gt;
											</UBadge>
											<UBadge color="purple" variant="subtle">
												taletrail.&gt;
											</UBadge>
										</div>
									</div>

									<div class="text-sm text-gray-600 dark:text-gray-400 p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
										<p class="font-medium mb-2">
											Connection Status:
										</p>
										<ul class="space-y-1 text-left">
											<li>Connected for: {{ formatDuration(connectionDuration) }}</li>
											<li>Messages received: {{ diagnostics.received }}</li>
										</ul>
									</div>

									<div class="text-sm text-gray-600 dark:text-gray-400 p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
										<UIcon name="i-heroicons-light-bulb" class="w-5 h-5 mx-auto mb-2 text-blue-500" />
										<p class="font-medium">
											Send an MCP request from the Request tab to see it here
										</p>
									</div>
								</div>

								<!-- Filtered out -->
								<div v-else-if="messages.length > 0" class="text-sm text-gray-600 dark:text-gray-400">
									<p>Try adjusting your filters to see more messages</p>
								</div>

								<!-- Disconnected -->
								<div v-else class="space-y-4">
									<p class="text-sm text-gray-600 dark:text-gray-400">
										Monitoring is disconnected
									</p>

									<div class="text-left bg-yellow-50 dark:bg-yellow-900/20 p-4 rounded-lg">
										<p class="font-semibold text-sm mb-2 flex items-center gap-2">
											<UIcon name="i-heroicons-wrench-screwdriver" class="w-4 h-4" />
											Troubleshooting Steps:
										</p>
										<ul class="text-xs space-y-2 text-gray-700 dark:text-gray-300 list-disc list-inside">
											<li>Verify NATS server is running</li>
											<li>Check NKey authentication credentials</li>
											<li>Verify TLS certificates are valid</li>
											<li>Try manual reconnection below</li>
										</ul>
									</div>

									<UButton
										color="blue"
										icon="i-heroicons-arrow-path"
										:loading="reconnecting"
										@click="manualReconnect"
									>
										Reconnect Now
									</UButton>
								</div>
							</div>
						</div>

						<!-- Message Items -->
						<MonitoringMessageItem
							v-for="(message, index) in filteredMessages"
							:key="`${message.timestamp}-${index}`"
							:message="message"
						/>
					</div>
				</UCard>
			</div>

			<!-- Control Section -->
			<div class="mt-4 flex items-center justify-between">
				<div class="flex items-center gap-3">
					<!-- Auto-scroll Toggle -->
					<UCheckbox v-model="autoScroll" label="Auto-scroll" />

					<!-- Manual Reconnect Button -->
					<UButton
						variant="outline"
						color="blue"
						icon="i-heroicons-arrow-path"
						:loading="reconnecting"
						@click="manualReconnect"
					>
						{{ connected ? "Force Reconnect" : "Reconnect to NATS" }}
					</UButton>

					<!-- Clear Button -->
					<UButton
						variant="outline"
						color="gray"
						icon="i-heroicons-trash"
						:disabled="messages.length === 0"
						@click="clearMessages"
					>
						Clear Messages
					</UButton>

					<!-- Debug Console Button -->
					<UButton
						variant="outline"
						color="purple"
						icon="i-heroicons-command-line"
						@click="showDebugInfo"
					>
						Debug Console
					</UButton>
				</div>

				<!-- Connection Status with Activity Indicator -->
				<div class="flex items-center gap-3">
					<!-- Connection Status -->
					<div class="flex items-center gap-2">
						<div
							class="w-3 h-3 rounded-full" :class="[
								connected ? 'bg-green-500' : 'bg-red-500'
							]"
						/>
						<span class="text-sm font-medium">
							{{ connected ? "Connected" : "Disconnected" }}
						</span>
					</div>

					<!-- Message Rate -->
					<div v-if="connected && diagnostics.received > 0" class="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
						<UIcon name="i-heroicons-chart-bar" class="w-4 h-4" />
						<span class="font-mono">{{ messageRate }}/s</span>
					</div>
				</div>
			</div>
		</div>
	</UContainer>
</template>

<script lang="ts" setup>
	import type { UnlistenFn } from "@tauri-apps/api/event";
	import type { EndpointFilter, MonitoringDiagnostics, NatsMessage, NatsMonitorStatus } from "@/types/monitoring";
	import { invoke } from "@tauri-apps/api/core";
	import { listen } from "@tauri-apps/api/event";
	import { refDebounced } from "@vueuse/core";

	definePageMeta({
		layout: "default",
		name: "Raw Messages",
		description: "View live NATS messages",
		icon: "i-heroicons-signal",
		category: "monitoring",
		showInNav: true
	});

	// ============================================================================
	// Constants
	// ============================================================================

	const MAX_MESSAGES = 1000;
	const DEBOUNCE_DELAY = 300; // ms
	const ACTIVITY_WARNING_THRESHOLD = 30; // seconds
	const ACTIVITY_ERROR_THRESHOLD = 60; // seconds

	// ============================================================================
	// State
	// ============================================================================

	const messages = ref<NatsMessage[]>([]);
	const selectedEndpoint = ref<EndpointFilter>("all");
	const filterText = ref("");
	const autoScroll = ref(true);
	const connected = ref(false);
	const reconnecting = ref(false);
	const messageContainer = ref<HTMLElement | null>(null);

	// Enhanced diagnostic state
	const diagnostics = ref<MonitoringDiagnostics>({
		received: 0,
		emitted: 0,
		failures: 0,
		lastMessage: undefined,
		connected: new Date().toISOString()
	});
	const lastDiagnosticUpdate = ref<string | null>(null);
	const connectionError = ref<string | null>(null);

	// Event unlisteners
	let unlistenMessage: UnlistenFn | null = null;
	let unlistenStatus: UnlistenFn | null = null;
	let unlistenDiagnostics: UnlistenFn | null = null;
	let unlistenError: UnlistenFn | null = null;

	// Debounced filter text
	const debouncedFilterText = refDebounced(filterText, DEBOUNCE_DELAY);

	// ============================================================================
	// Computed
	// ============================================================================

	const endpointOptions = [
		{ label: "All Endpoints", value: "all" },
		{ label: "Orchestrator", value: "orchestrator" },
		{ label: "Story Generator", value: "story-generator" },
		{ label: "Quality Control", value: "quality-control" },
		{ label: "Constraint Enforcer", value: "constraint-enforcer" },
		{ label: "Prompt Helper", value: "prompt-helper" }
	];

	const selectedEndpointLabel = computed(() => {
		const option = endpointOptions.find((opt) => opt.value === selectedEndpoint.value);
		return option?.label || "All Endpoints";
	});

	// Filter messages by endpoint and text
	const filteredMessages = computed(() => {
		let filtered = messages.value;

		// Filter by endpoint
		if (selectedEndpoint.value !== "all") {
			filtered = filtered.filter((msg) => msg.endpoint === selectedEndpoint.value);
		}

		// Filter by text (wildcard match on subject, payload, and request_id)
		if (debouncedFilterText.value.trim()) {
			const searchText = debouncedFilterText.value.toLowerCase();
			filtered = filtered.filter((msg) => {
				return (
					msg.subject.toLowerCase().includes(searchText)
					|| msg.payload.toLowerCase().includes(searchText)
					|| (msg.request_id && msg.request_id.toLowerCase().includes(searchText))
				);
			});
		}

		return filtered;
	});

	// Calculate time since last message in seconds
	const timeSinceLastMessage = computed(() => {
		if (!diagnostics.value.lastMessage) {
			return null;
		}
		const lastMessageTime = new Date(diagnostics.value.lastMessage).getTime();
		const now = Date.now();
		return Math.floor((now - lastMessageTime) / 1000);
	});

	// Activity indicator status
	const activityStatus = computed<"active" | "warning" | "error">(() => {
		const timeSince = timeSinceLastMessage.value;
		if (timeSince === null || timeSince < ACTIVITY_WARNING_THRESHOLD) {
			return "active";
		}
		if (timeSince < ACTIVITY_ERROR_THRESHOLD) {
			return "warning";
		}
		return "error";
	});

	// Message rate calculation (messages per second over last 10 seconds)
	const messageRate = computed(() => {
		if (messages.value.length === 0) {
			return 0;
		}
		const now = Date.now();
		const tenSecondsAgo = now - 10000;
		const recentMessages = messages.value.filter((msg) => {
			const msgTime = new Date(msg.timestamp).getTime();
			return msgTime >= tenSecondsAgo;
		});
		return (recentMessages.length / 10).toFixed(1);
	});

	// Connection duration in seconds
	const connectionDuration = computed(() => {
		if (!connected.value || !diagnostics.value.connected) {
			return 0;
		}
		const connectedTime = new Date(diagnostics.value.connected).getTime();
		const now = Date.now();
		return Math.floor((now - connectedTime) / 1000);
	});

	// ============================================================================
	// Methods
	// ============================================================================

	function formatTimestamp(iso: string | undefined): string {
		if (!iso) return "Never";

		const date = new Date(iso);
		const now = Date.now();
		const diff = Math.floor((now - date.getTime()) / 1000);

		if (diff < 60) return `${diff} seconds ago`;
		if (diff < 3600) return `${Math.floor(diff / 60)} minutes ago`;
		if (diff < 86400) return `${Math.floor(diff / 3600)} hours ago`;
		return `${Math.floor(diff / 86400)} days ago`;
	}

	function formatDuration(seconds: number): string {
		if (seconds < 60) return `${seconds}s`;
		if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
		return `${Math.floor(seconds / 3600)}h ${Math.floor((seconds % 3600) / 60)}m`;
	}

	function clearMessages() {
		messages.value = [];
		// Reset diagnostics
		diagnostics.value = {
			received: 0,
			emitted: 0,
			failures: 0,
			lastMessage: undefined,
			connected: diagnostics.value.connected
		};
	}

	async function manualReconnect() {
		try {
			reconnecting.value = true;
			connectionError.value = null;

			// Stop monitoring
			await invoke("stop_nats_monitoring");
			console.log("[Monitoring] Stopped NATS monitoring");

			// Wait a moment before reconnecting
			await new Promise((resolve) => setTimeout(resolve, 1000));

			// Start monitoring
			await invoke("start_nats_monitoring");
			console.log("[Monitoring] Reconnected to NATS successfully");

			// Update status
			const status = await invoke<boolean>("get_monitoring_status");
			connected.value = status;

			// Reset diagnostics with new connection time
			diagnostics.value = {
				received: 0,
				emitted: 0,
				failures: 0,
				lastMessage: undefined,
				connected: new Date().toISOString()
			};
		} catch (e) {
			console.error("[Monitoring] Failed to reconnect:", e);
			connectionError.value = `Reconnection failed: ${String(e)}`;
			connected.value = false;
		} finally {
			reconnecting.value = false;
		}
	}

	function showDebugInfo() {
		const debugInfo = `
🔍 NATS MONITORING DEBUG INFO
════════════════════════════

📊 CURRENT STATE:
  • Connection Status: ${connected.value ? '✅ Connected' : '❌ Disconnected'}
  • Messages Received: ${diagnostics.value.received}
  • Messages Displayed: ${diagnostics.value.emitted}
  • Emission Failures: ${diagnostics.value.failures}
  • Last Message: ${diagnostics.value.lastMessage || 'Never'}
  • Connection Time: ${diagnostics.value.connected}

🎯 SUBSCRIBED SUBJECTS:
  • mcp.> (All MCP messages)
  • taletrail.> (All generation events)

📝 DEBUGGING STEPS:
  1. Check terminal logs for [NATS Monitor] messages
  2. Look for "Successfully emitted" or "Failed to emit" logs
  3. Verify NATS server is running: ps aux | grep nats
  4. Check browser console (F12) for JavaScript errors
  5. Try sending an MCP request from Request tab

💡 TERMINAL LOGS:
  Check the terminal where you ran 'bun run tauri:dev'
  Look for these log prefixes:
  • [NATS Monitor] - Monitoring system logs
  • [TaleTrail] - Application logs
  • [MCP Tester] - Request execution logs

🔧 KEYBOARD SHORTCUTS:
  • F12 - Open browser developer console
  • Cmd+Option+I (Mac) / Ctrl+Shift+I (Windows/Linux)
		`.trim();

		console.log(debugInfo);

		// Use browser notification instead of alert (which requires Tauri permissions)
		if (window.Notification && Notification.permission === "granted") {
			new Notification("NATS Monitoring Debug", {
				body: "Debug info logged to console (F12)"
			});
		}

		// Show in-page notification instead of alert
		connectionError.value = `Debug info logged to console. Press F12 to view.`;
		setTimeout(() => {
			connectionError.value = null;
		}, 3000);
	}

	function scrollToBottom() {
		if (autoScroll.value && messageContainer.value) {
			// Check if user is near bottom (within 100px)
			const container = messageContainer.value;
			const isNearBottom
				= container.scrollHeight - container.scrollTop - container.clientHeight < 100;

			if (isNearBottom || messages.value.length === 1) {
				nextTick(() => {
					if (container) {
						container.scrollTop = container.scrollHeight;
					}
				});
			}
		}
	}

	function getEndpointColor(endpoint: string): string {
		switch (endpoint) {
		case "orchestrator":
			return "indigo";
		case "story-generator":
			return "cyan";
		case "quality-control":
			return "amber";
		case "constraint-enforcer":
			return "red";
		case "prompt-helper":
			return "pink";
		default:
			return "gray";
		}
	}

	// ============================================================================
	// Event Handlers
	// ============================================================================

	function handleNatsMessage(message: NatsMessage) {
		console.log("[Monitoring] [DIAGNOSTIC] handleNatsMessage called");
		console.log("[Monitoring] [DIAGNOSTIC] Message details:", {
			subject: message.subject,
			endpoint: message.endpoint,
			messageType: message.message_type,
			timestamp: message.timestamp,
			requestId: message.request_id,
			payloadLength: message.payload.length
		});

		// Update diagnostics
		diagnostics.value.received++;
		diagnostics.value.lastMessage = message.timestamp;
		lastDiagnosticUpdate.value = new Date().toISOString();

		console.log("[Monitoring] [DIAGNOSTIC] Diagnostics updated (frontend):", {
			received: diagnostics.value.received,
			emitted: diagnostics.value.emitted,
			failures: diagnostics.value.failures
		});

		try {
			console.log("[Monitoring] [DIAGNOSTIC] Current messages array length:", messages.value.length);

			// Add new message
			messages.value.push(message);
			console.log("[Monitoring] [DIAGNOSTIC] Message added, new length:", messages.value.length);

			// Limit to MAX_MESSAGES (FIFO: remove oldest)
			if (messages.value.length > MAX_MESSAGES) {
				messages.value = messages.value.slice(-MAX_MESSAGES);
				console.log("[Monitoring] [DIAGNOSTIC] Messages trimmed to MAX_MESSAGES:", MAX_MESSAGES);
			}

			// Track successful emission
			diagnostics.value.emitted++;
			console.log("[Monitoring] [DIAGNOSTIC] Message successfully added to display, total emitted:", diagnostics.value.emitted);

			// Auto-scroll to bottom
			scrollToBottom();
		} catch (e) {
			console.error("[Monitoring] [ERROR] Failed to handle message:", e);
			diagnostics.value.failures++;
		}
	}

	function handleStatusUpdate(status: NatsMonitorStatus) {
		connected.value = status.connected;
		console.log("[Monitoring] Status update:", status);
	}

	// ============================================================================
	// Lifecycle
	// ============================================================================

	onMounted(async () => {
		console.log("[Monitoring] [DIAGNOSTIC] Starting monitoring page initialization");

		try {
			// Check initial connection status
			console.log("[Monitoring] [DIAGNOSTIC] Checking initial monitoring status...");
			const status = await invoke<boolean>("get_monitoring_status");
			connected.value = status;
			console.log("[Monitoring] [DIAGNOSTIC] Initial status:", status);

			// Listen to NATS messages
			console.log("[Monitoring] [DIAGNOSTIC] Registering event listener for 'nats-message'");
			unlistenMessage = await listen<NatsMessage>("nats-message", (event) => {
				console.log("[Monitoring] [DIAGNOSTIC] Received 'nats-message' event:", event.payload);
				handleNatsMessage(event.payload);
			});
			console.log("[Monitoring] [DIAGNOSTIC] 'nats-message' listener registered successfully");

			// Listen to status updates
			console.log("[Monitoring] [DIAGNOSTIC] Registering event listener for 'nats-monitor-status'");
			unlistenStatus = await listen<NatsMonitorStatus>("nats-monitor-status", (event) => {
				console.log("[Monitoring] [DIAGNOSTIC] Received 'nats-monitor-status' event:", event.payload);
				handleStatusUpdate(event.payload);
			});
			console.log("[Monitoring] [DIAGNOSTIC] 'nats-monitor-status' listener registered successfully");

			// Listen to diagnostic updates (every 5 seconds from backend)
			console.log("[Monitoring] [DIAGNOSTIC] Registering event listener for 'nats-monitor-diagnostics'");
			unlistenDiagnostics = await listen("nats-monitor-diagnostics", (event) => {
				const backendDiag = event.payload as any;
				console.log("[Monitoring] [DIAGNOSTIC] Received 'nats-monitor-diagnostics' event:", backendDiag);
				diagnostics.value = {
					received: backendDiag.messages_received || 0,
					emitted: backendDiag.messages_emitted || 0,
					failures: backendDiag.emission_failures || 0,
					lastMessage: backendDiag.last_message_timestamp,
					connected: backendDiag.connection_timestamp || new Date().toISOString()
				};
				lastDiagnosticUpdate.value = new Date().toISOString();
				console.log("[Monitoring] Diagnostics updated:", diagnostics.value);
			});
			console.log("[Monitoring] [DIAGNOSTIC] 'nats-monitor-diagnostics' listener registered successfully");

			// Listen to monitoring errors
			console.log("[Monitoring] [DIAGNOSTIC] Registering event listener for 'nats-monitor-error'");
			unlistenError = await listen("nats-monitor-error", (event) => {
				const error = event.payload as any;
				console.error("[Monitoring] [DIAGNOSTIC] Received 'nats-monitor-error' event:", error);
				connectionError.value = error.error || "Unknown monitoring error";
				console.error("[Monitoring] Error received:", error);
			});
			console.log("[Monitoring] [DIAGNOSTIC] 'nats-monitor-error' listener registered successfully");

			console.log("[Monitoring] [DIAGNOSTIC] All event listeners registered successfully!");
			console.log("[Monitoring] [DIAGNOSTIC] Event listeners count:", {
				natsMessage: !!unlistenMessage,
				status: !!unlistenStatus,
				diagnostics: !!unlistenDiagnostics,
				error: !!unlistenError
			});
		} catch (error) {
			console.error("[Monitoring] [ERROR] Failed to initialize:", error);
		}

		// Poll connection status every 3 seconds to detect disconnects
		setInterval(async () => {
			try {
				const currentStatus = await invoke<boolean>("get_monitoring_status");
				connected.value = currentStatus;
			} catch {
				connected.value = false;
			}
		}, 3000);
	});

	onUnmounted(() => {
		// Clean up event listeners
		if (unlistenDiagnostics) {
			unlistenDiagnostics();
		}
		if (unlistenError) {
			unlistenError();
		}
		if (unlistenMessage) {
			unlistenMessage();
		}
		if (unlistenStatus) {
			unlistenStatus();
		}
		console.log("[Monitoring] Event listeners cleaned up");
	});

	// Watch for auto-scroll changes
	watch(autoScroll, (newValue) => {
		if (newValue) {
			scrollToBottom();
		}
	});
</script>
