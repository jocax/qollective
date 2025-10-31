<template>
	<UContainer class="relative overflow-hidden h-screen">
		<div class="flex flex-col h-full p-6">
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
							<div class="text-center p-8">
								<UIcon
									name="i-heroicons-signal-slash"
									class="w-16 h-16 mx-auto mb-4 text-gray-400"
								/>
								<h3 class="text-lg font-semibold mb-2">
									No Messages
								</h3>
								<p class="text-sm text-gray-600 dark:text-gray-400">
									{{ messages.length === 0
										? "Waiting for NATS messages..."
										: "No messages match your filters"
									}}
								</p>
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

					<!-- Reconnect Button -->
					<UButton
						variant="outline"
						color="blue"
						icon="i-heroicons-arrow-path"
						:loading="reconnecting"
						:disabled="connected"
						@click="reconnectToNats"
					>
						Reconnect to NATS
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
				</div>

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
			</div>
		</div>
	</UContainer>
</template>

<script lang="ts" setup>
	import type { UnlistenFn } from "@tauri-apps/api/event";
	import type { EndpointFilter, NatsMessage, NatsMonitorStatus } from "@/types/monitoring";
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

	// Event unlisteners
	let unlistenMessage: UnlistenFn | null = null;
	let unlistenStatus: UnlistenFn | null = null;

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

	// ============================================================================
	// Methods
	// ============================================================================

	function clearMessages() {
		messages.value = [];
	}

	async function reconnectToNats() {
		try {
			reconnecting.value = true;

			// First check actual connection status
			const status = await invoke<boolean>("get_monitoring_status");
			connected.value = status;

			// If not connected, try to reconnect
			if (!status) {
				await invoke('start_nats_monitoring');
				console.log('[Monitoring] Reconnected to NATS successfully');

				// Update status after reconnect
				const newStatus = await invoke<boolean>("get_monitoring_status");
				connected.value = newStatus;
			} else {
				console.log('[Monitoring] Already connected to NATS');
			}
		} catch (e) {
			console.error('[Monitoring] Failed to reconnect:', e);
			connected.value = false;
		} finally {
			reconnecting.value = false;
		}
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
		// Add new message
		messages.value.push(message);

		// Limit to MAX_MESSAGES (FIFO: remove oldest)
		if (messages.value.length > MAX_MESSAGES) {
			messages.value = messages.value.slice(-MAX_MESSAGES);
		}

		// Auto-scroll to bottom
		scrollToBottom();
	}

	function handleStatusUpdate(status: NatsMonitorStatus) {
		connected.value = status.connected;
		console.log("[Monitoring] Status update:", status);
	}

	// ============================================================================
	// Lifecycle
	// ============================================================================

	onMounted(async () => {
		try {
			// Check initial connection status
			const status = await invoke<boolean>("get_monitoring_status");
			connected.value = status;
			console.log("[Monitoring] Initial status:", status);

			// Listen to NATS messages
			unlistenMessage = await listen<NatsMessage>("nats-message", (event) => {
				handleNatsMessage(event.payload);
			});

			// Listen to status updates
			unlistenStatus = await listen<NatsMonitorStatus>("nats-monitor-status", (event) => {
				handleStatusUpdate(event.payload);
			});

			console.log("[Monitoring] Event listeners registered");
		} catch (error) {
			console.error("[Monitoring] Failed to initialize:", error);
		}

		// Poll connection status every 3 seconds to detect disconnects
		setInterval(async () => {
			try {
				const currentStatus = await invoke<boolean>("get_monitoring_status");
				connected.value = currentStatus;
			} catch (e) {
				connected.value = false;
			}
		}, 3000);
	});

	onUnmounted(() => {
		// Clean up event listeners
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
