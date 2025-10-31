import type { UnlistenFn } from "@tauri-apps/api/event";
import type { GenerationEvent, GenerationRequest, NatsConnectionStatus, PhaseProgress } from "~/types/trails";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { computed, onUnmounted, ref } from "vue";

/**
 * Composable for NATS Live Generation Monitoring
 * Manages real-time event streaming from NATS backend
 */
export function useNatsLiveMonitor() {
	// State
	const events = ref<GenerationEvent[]>([]);
	const requests = ref<Map<string, GenerationRequest>>(new Map());
	const connectionStatus = ref<NatsConnectionStatus>({
		connected: false,
		subscribed: false,
		tenantId: undefined
	});
	const error = ref<string | null>(null);
	const loading = ref(false);
	const isPaused = ref(false);
	const autoScroll = ref(true);

	// Event listener cleanup
	let unlistenFn: UnlistenFn | null = null;

	// Filter states
	const statusFilter = ref<string>("all");
	const tenantFilter = ref<string | null>(null);

	// Service phases for tracking
	const SERVICE_PHASES = [
		"prompt-helper",
		"story-generator",
		"quality-control",
		"constraint-enforcer"
	] as const;

	// Computed filtered events
	const filteredEvents = computed(() => {
		let filtered = events.value;

		// Status filter
		if (statusFilter.value !== "all") {
			filtered = filtered.filter((e) => e.status === statusFilter.value);
		}

		// Tenant filter
		if (tenantFilter.value) {
			filtered = filtered.filter((e) => e.tenantId === tenantFilter.value);
		}

		return filtered;
	});

	// Computed active requests
	const activeRequests = computed(() => {
		return Array.from(requests.value.values()).filter((r) => r.status === "in_progress");
	});

	// Computed completed requests
	const completedRequests = computed(() => {
		return Array.from(requests.value.values()).filter((r) => r.status === "completed");
	});

	// Computed failed requests
	const failedRequests = computed(() => {
		return Array.from(requests.value.values()).filter((r) => r.status === "failed");
	});

	// Computed statistics
	const statistics = computed(() => {
		return {
			total: events.value.length,
			inProgress: events.value.filter((e) => e.status === "in_progress").length,
			completed: events.value.filter((e) => e.status === "completed").length,
			failed: events.value.filter((e) => e.status === "failed").length,
			activeRequests: activeRequests.value.length,
			completedRequests: completedRequests.value.length,
			failedRequests: failedRequests.value.length
		};
	});

	// Computed unique tenant IDs
	const uniqueTenantIds = computed(() => {
		const tenants = new Set(events.value.map((e) => e.tenantId));
		return Array.from(tenants).sort();
	});

	/**
	 * Subscribe to NATS generation events
	 */
	async function subscribe(tenantId?: string) {
		loading.value = true;
		error.value = null;

		try {
			console.log("[useNatsLiveMonitor] Subscribing to generation events", { tenantId });

			// Check connection status first
			const status = await invoke<NatsConnectionStatus>("nats_connection_status");
			console.log("[useNatsLiveMonitor] Connection status:", status);

			// Subscribe via Tauri backend
			await invoke("subscribe_to_generations", { tenantId });

			// Set up event listener
			unlistenFn = await listen<GenerationEvent>("generation-event", (event) => {
				if (!isPaused.value) {
					handleGenerationEvent(event.payload);
				}
			});

			connectionStatus.value = {
				connected: true,
				subscribed: true,
				tenantId
			};

			console.log("[useNatsLiveMonitor] Successfully subscribed to generation events");
		} catch (err) {
			console.error("[useNatsLiveMonitor] Failed to subscribe:", err);
			error.value = err instanceof Error ? err.message : String(err);
			connectionStatus.value = {
				connected: false,
				subscribed: false,
				tenantId: undefined
			};
		} finally {
			loading.value = false;
		}
	}

	/**
	 * Unsubscribe from NATS generation events
	 */
	async function unsubscribe() {
		loading.value = true;
		error.value = null;

		try {
			console.log("[useNatsLiveMonitor] Unsubscribing from generation events");

			// Unsubscribe via Tauri backend
			await invoke("unsubscribe_from_generations");

			// Clean up event listener
			if (unlistenFn) {
				unlistenFn();
				unlistenFn = null;
			}

			connectionStatus.value = {
				connected: false,
				subscribed: false,
				tenantId: undefined
			};

			console.log("[useNatsLiveMonitor] Successfully unsubscribed");
		} catch (err) {
			console.error("[useNatsLiveMonitor] Failed to unsubscribe:", err);
			error.value = err instanceof Error ? err.message : String(err);
		} finally {
			loading.value = false;
		}
	}

	/**
	 * Disconnect from NATS
	 */
	async function disconnect() {
		loading.value = true;
		error.value = null;

		try {
			console.log("[useNatsLiveMonitor] Disconnecting from NATS");

			await unsubscribe();
			await invoke("disconnect_nats");

			connectionStatus.value = {
				connected: false,
				subscribed: false,
				tenantId: undefined
			};

			console.log("[useNatsLiveMonitor] Successfully disconnected from NATS");
		} catch (err) {
			console.error("[useNatsLiveMonitor] Failed to disconnect:", err);
			error.value = err instanceof Error ? err.message : String(err);
		} finally {
			loading.value = false;
		}
	}

	/**
	 * Check NATS connection status
	 */
	async function checkConnectionStatus() {
		try {
			const status = await invoke<NatsConnectionStatus>("nats_connection_status");
			connectionStatus.value = status;
			return status;
		} catch (err) {
			console.error("[useNatsLiveMonitor] Failed to check connection status:", err);
			error.value = err instanceof Error ? err.message : String(err);
			return null;
		}
	}

	/**
	 * Handle incoming generation event
	 */
	function handleGenerationEvent(event: GenerationEvent) {
		console.log("[useNatsLiveMonitor] Received event:", event);

		// Add event to list (newest first for display)
		events.value.unshift(event);

		// Update request tracking
		updateRequestTracking(event);

		// Enforce max buffer size (MAX_EVENT_BUFFER_SIZE from constants)
		const MAX_BUFFER = 1000; // Matches MAX_EVENT_BUFFER_SIZE constant
		if (events.value.length > MAX_BUFFER) {
			events.value = events.value.slice(0, MAX_BUFFER);
		}
	}

	/**
	 * Update request tracking with phase information
	 */
	function updateRequestTracking(event: GenerationEvent) {
		const { requestId, tenantId, timestamp, servicePhase, status, progress, errorMessage } = event;

		let request = requests.value.get(requestId);

		if (!request) {
			// Create new request tracking
			request = {
				requestId,
				tenantId,
				startTime: timestamp,
				phases: new Map(),
				status: "in_progress",
				errorMessage: undefined
			};
			requests.value.set(requestId, request);
		}

		// Update phase progress
		const phaseProgress: PhaseProgress = {
			phase: servicePhase,
			status,
			progress: progress ?? 0,
			startTime: request.phases.get(servicePhase)?.startTime ?? timestamp,
			endTime: status === "completed" || status === "failed" ? timestamp : undefined,
			errorMessage
		};

		request.phases.set(servicePhase, phaseProgress);

		// Update overall request status
		if (status === "failed") {
			request.status = "failed";
			request.errorMessage = errorMessage;
		} else if (status === "completed" && allPhasesCompleted(request)) {
			request.status = "completed";
		} else {
			request.status = "in_progress";
		}
	}

	/**
	 * Check if all phases are completed for a request
	 */
	function allPhasesCompleted(request: GenerationRequest): boolean {
		return SERVICE_PHASES.every((phase) => {
			const phaseProgress = request.phases.get(phase);
			return phaseProgress && phaseProgress.status === "completed";
		});
	}

	/**
	 * Clear all events and requests
	 */
	function clearAll() {
		events.value = [];
		requests.value.clear();
		error.value = null;
	}

	/**
	 * Pause event processing (don't add new events)
	 */
	function pause() {
		isPaused.value = true;
		autoScroll.value = false;
		console.log("[useNatsLiveMonitor] Event processing paused");
	}

	/**
	 * Resume event processing
	 */
	function resume() {
		isPaused.value = false;
		autoScroll.value = true;
		console.log("[useNatsLiveMonitor] Event processing resumed");
	}

	/**
	 * Cancel subscription and clear all data
	 */
	async function cancel() {
		await unsubscribe();
		clearAll();
		isPaused.value = false;
		autoScroll.value = true;
		console.log("[useNatsLiveMonitor] Subscription cancelled and data cleared");
	}

	/**
	 * Get request by ID
	 */
	function getRequest(requestId: string): GenerationRequest | undefined {
		return requests.value.get(requestId);
	}

	/**
	 * Set status filter
	 */
	function setStatusFilter(status: string) {
		statusFilter.value = status;
	}

	/**
	 * Set tenant filter
	 */
	function setTenantFilter(tenantId: string | null) {
		tenantFilter.value = tenantId;
	}

	/**
	 * Clear all filters
	 */
	function clearFilters() {
		statusFilter.value = "all";
		tenantFilter.value = null;
	}

	// Cleanup on unmount
	onUnmounted(async () => {
		if (connectionStatus.value.subscribed) {
			await unsubscribe();
		}
	});

	return {
		// State
		events,
		requests,
		connectionStatus,
		error,
		loading,
		isPaused,
		autoScroll,
		statusFilter,
		tenantFilter,

		// Computed
		filteredEvents,
		activeRequests,
		completedRequests,
		failedRequests,
		statistics,
		uniqueTenantIds,

		// Actions
		subscribe,
		unsubscribe,
		disconnect,
		checkConnectionStatus,
		clearAll,
		pause,
		resume,
		cancel,
		getRequest,
		setStatusFilter,
		setTenantFilter,
		clearFilters
	};
}
