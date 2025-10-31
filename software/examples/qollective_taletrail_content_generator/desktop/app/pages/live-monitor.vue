<template>
	<UContainer class="relative overflow-hidden h-screen">
		<div class="flex flex-col h-full p-6">
			<!-- Header Section -->
			<div class="mb-6">
				<h1 class="text-3xl font-bold font-heading mb-2">
					Live Generation Monitor
				</h1>
				<p class="text-gray-600 dark:text-gray-400">
					Real-time monitoring of TaleTrail generation pipeline via NATS
				</p>
			</div>

			<!-- Toolbar Section -->
			<div class="mb-6 space-y-4">
				<!-- Connection Controls Row -->
				<div class="flex gap-3 flex-wrap">
					<!-- Subscribe Button -->
					<UButton
						v-if="!connectionStatus.subscribed"
						variant="solid"
						color="green"
						size="lg"
						icon="i-heroicons-play"
						:loading="loading"
						@click="handleSubscribe"
					>
						Subscribe
					</UButton>

					<!-- Unsubscribe Button -->
					<UButton
						v-if="connectionStatus.subscribed"
						variant="solid"
						color="red"
						size="lg"
						icon="i-heroicons-stop"
						:loading="loading"
						@click="handleUnsubscribe"
					>
						Unsubscribe
					</UButton>

					<!-- Pause/Resume Button -->
					<UButton
						v-if="connectionStatus.subscribed"
						:variant="isPaused ? 'solid' : 'outline'"
						:color="isPaused ? 'orange' : 'blue'"
						size="lg"
						:icon="isPaused ? 'i-heroicons-play' : 'i-heroicons-pause'"
						@click="handlePauseResume"
					>
						{{ isPaused ? 'Resume' : 'Pause' }}
					</UButton>

					<!-- Cancel Button -->
					<UButton
						v-if="connectionStatus.subscribed"
						variant="outline"
						color="red"
						size="lg"
						icon="i-heroicons-x-mark"
						@click="handleCancel"
					>
						Cancel & Clear
					</UButton>

					<!-- Clear Events Button -->
					<UButton
						v-if="events.length > 0 && !connectionStatus.subscribed"
						variant="ghost"
						size="lg"
						icon="i-heroicons-trash"
						@click="clearAll"
					>
						Clear Events
					</UButton>

					<!-- Back to Home -->
					<UButton
						variant="ghost"
						size="lg"
						icon="i-heroicons-arrow-left"
						to="/"
					>
						Back to Home
					</UButton>
				</div>

				<!-- Request ID Filter Indicator -->
				<UAlert
					v-if="filterRequestId"
					color="yellow"
					variant="solid"
					icon="i-heroicons-exclamation-triangle"
					class="border-2 border-yellow-500"
				>
					<template #title>
						<span class="font-bold">Active Filter: Tracking Request {{ filterRequestId }}</span>
					</template>
					<template #description>
						<div class="space-y-1">
							<p>Showing only messages related to this generation request.</p>
							<p class="font-semibold">
								Displaying {{ filteredEventsComputed.length }} of {{ events.length }} total events
							</p>
							<p v-if="filteredEventsComputed.length === 0 && events.length > 0" class="text-yellow-900 dark:text-yellow-100 font-semibold">
								⚠️ No events match this request ID. All {{ events.length }} events are hidden by this filter.
							</p>
						</div>
					</template>
					<template #actions>
						<UButton
							size="sm"
							color="yellow"
							variant="solid"
							icon="i-heroicons-x-mark"
							class="font-semibold"
							@click="clearFilter"
						>
							Show All {{ events.length }} Events
						</UButton>
					</template>
				</UAlert>

				<!-- Comprehensive Filters Section -->
				<div v-if="connectionStatus.subscribed || events.length > 0" class="space-y-3">
					<div class="flex items-center justify-between">
						<h3 class="text-sm font-semibold text-gray-700 dark:text-gray-300">
							Filters
						</h3>
						<div class="flex items-center gap-2">
							<UBadge v-if="activeFilterCount > 0" color="primary" variant="subtle">
								{{ activeFilterCount }} active {{ activeFilterCount === 1 ? 'filter' : 'filters' }}
							</UBadge>
							<UButton
								v-if="activeFilterCount > 0"
								variant="ghost"
								size="xs"
								icon="i-heroicons-x-mark"
								@click="handleClearFilters"
							>
								Clear Filters
							</UButton>
						</div>
					</div>

					<!-- Filter Controls Grid -->
					<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-3">
						<!-- Event Type Filter -->
						<div class="filter-group">
							<label class="text-xs font-medium text-gray-600 dark:text-gray-400 mb-1 block">
								Event Types
							</label>
							<USelectMenu
								v-model="selectedEventTypes"
								:options="eventTypeOptions"
								multiple
								placeholder="All event types"
								class="w-full"
							>
								<template #leading>
									<UIcon name="i-heroicons-tag" />
								</template>
							</USelectMenu>
						</div>

						<!-- Tenant Filter -->
						<div class="filter-group">
							<label class="text-xs font-medium text-gray-600 dark:text-gray-400 mb-1 block">
								Tenants
							</label>
							<USelectMenu
								v-model="selectedTenants"
								:options="tenantOptions"
								multiple
								placeholder="All tenants"
								class="w-full"
							>
								<template #leading>
									<UIcon name="i-heroicons-building-office" />
								</template>
							</USelectMenu>
						</div>

						<!-- Component Filter -->
						<div class="filter-group">
							<label class="text-xs font-medium text-gray-600 dark:text-gray-400 mb-1 block">
								Components
							</label>
							<USelectMenu
								v-model="selectedComponents"
								:options="componentOptions"
								multiple
								placeholder="All components"
								class="w-full"
							>
								<template #leading>
									<UIcon name="i-heroicons-cog" />
								</template>
							</USelectMenu>
						</div>

						<!-- Time Range Filter -->
						<div class="filter-group">
							<label class="text-xs font-medium text-gray-600 dark:text-gray-400 mb-1 block">
								Time Range
							</label>
							<USelectMenu
								v-model="selectedTimeRange"
								:options="timeRangeOptions"
								placeholder="All time"
								class="w-full"
							>
								<template #leading>
									<UIcon name="i-heroicons-clock" />
								</template>
							</USelectMenu>
						</div>
					</div>

					<!-- Request ID Filter (per component) -->
					<div v-if="selectedComponents.length > 0" class="space-y-2">
						<label class="text-xs font-medium text-gray-600 dark:text-gray-400 block">
							Request ID (per component)
						</label>
						<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-2">
							<div v-for="component in selectedComponents" :key="component" class="flex items-center gap-2">
								<span class="text-xs text-gray-500 dark:text-gray-400 min-w-[120px]">{{ component }}:</span>
								<UInput
									:model-value="requestIdByComponent[component] || ''"
									:placeholder="`Request ID for ${component}`"
									size="xs"
									class="flex-1"
									@update:model-value="(val) => setRequestIdForComponent(component, val)"
								/>
							</div>
						</div>
					</div>
				</div>

				<!-- Connection Status Banner -->
				<div class="space-y-2">
					<div class="flex items-center gap-2 text-sm">
						<UBadge
							:color="connectionStatus.connected ? 'green' : 'gray'"
							variant="subtle"
						>
							<template #leading>
								<UIcon
									:name="connectionStatus.connected ? 'i-heroicons-check-circle' : 'i-heroicons-x-circle'"
								/>
							</template>
							{{ connectionStatus.connected ? 'Connected' : 'Disconnected' }}
						</UBadge>

						<UBadge
							v-if="connectionStatus.subscribed"
							color="blue"
							variant="subtle"
						>
							<template #leading>
								<UIcon name="i-heroicons-radio" class="animate-pulse" />
							</template>
							Subscribed
						</UBadge>

						<UBadge
							v-if="isPaused"
							color="orange"
							variant="subtle"
						>
							<template #leading>
								<UIcon name="i-heroicons-pause" />
							</template>
							Paused
						</UBadge>

						<span v-if="connectionStatus.tenantId" class="text-gray-600 dark:text-gray-400 text-xs">
							Tenant: {{ connectionStatus.tenantId }}
						</span>
					</div>

					<!-- Statistics -->
					<div class="grid grid-cols-2 md:grid-cols-5 gap-2 text-xs">
						<div class="bg-gray-100 dark:bg-gray-800 p-2 rounded">
							<div class="text-gray-500 dark:text-gray-400">
								Total Events
							</div>
							<div class="text-lg font-semibold text-gray-900 dark:text-gray-100">
								{{ events.length }}
							</div>
						</div>
						<div class="bg-blue-50 dark:bg-blue-900/20 p-2 rounded">
							<div class="text-blue-600 dark:text-blue-400">
								Filtered
							</div>
							<div class="text-lg font-semibold text-blue-700 dark:text-blue-300">
								{{ filteredEventsComputed.length }}
							</div>
						</div>
						<div class="bg-indigo-50 dark:bg-indigo-900/20 p-2 rounded">
							<div class="text-indigo-600 dark:text-indigo-400">
								Active Requests
							</div>
							<div class="text-lg font-semibold text-indigo-700 dark:text-indigo-300">
								{{ activeRequestsData.length }}
							</div>
						</div>
						<div class="bg-green-50 dark:bg-green-900/20 p-2 rounded">
							<div class="text-green-600 dark:text-green-400">
								Completed
							</div>
							<div class="text-lg font-semibold text-green-700 dark:text-green-300">
								{{ statistics.completedRequests }}
							</div>
						</div>
						<div class="bg-red-50 dark:bg-red-900/20 p-2 rounded">
							<div class="text-red-600 dark:text-red-400">
								Failed
							</div>
							<div class="text-lg font-semibold text-red-700 dark:text-red-300">
								{{ statistics.failedRequests }}
							</div>
						</div>
					</div>
				</div>
			</div>

			<!-- Content Section with Three Columns -->
			<div class="flex-1 overflow-hidden">
				<!-- Error State -->
				<UAlert
					v-if="error"
					color="red"
					variant="subtle"
					title="Connection Error"
					:description="error"
					class="mb-4"
				/>

				<div class="grid grid-cols-1 lg:grid-cols-3 gap-4 h-full">
					<!-- Left Column: Active Requests -->
					<div class="flex flex-col overflow-hidden">
						<h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-3">
							Active Requests
							<UBadge color="blue" variant="subtle" class="ml-2">
								{{ activeRequestsData.length }}
							</UBadge>
						</h3>
						<div class="flex-1 overflow-y-auto space-y-3 pr-2">
							<!-- Empty State -->
							<div v-if="activeRequestsData.length === 0" class="flex items-center justify-center h-full">
								<div class="text-center p-8">
									<UIcon name="i-heroicons-clock" class="w-12 h-12 text-gray-400 mx-auto mb-3" />
									<h4 class="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-1">
										No Active Requests
									</h4>
									<p class="text-xs text-gray-600 dark:text-gray-400">
										Waiting for generation requests...
									</p>
								</div>
							</div>

							<!-- Active Request Cards -->
							<GenerationProgressCard
								v-for="request in activeRequestsForDisplay"
								:key="request.requestId"
								:request="request"
							/>
						</div>
					</div>

					<!-- Middle Column: Event Stream -->
					<div class="flex flex-col overflow-hidden">
						<!-- Filter Status Badge -->
						<div v-if="events.length > 0 && filteredEventsComputed.length < events.length" class="mb-2">
							<UBadge color="yellow" variant="subtle" size="lg" class="w-full justify-center py-2">
								<UIcon name="i-heroicons-funnel" class="mr-2" />
								Showing {{ filteredEventsComputed.length }} of {{ events.length }} events
								<UButton
									size="2xs"
									color="yellow"
									variant="ghost"
									class="ml-3"
									@click="handleClearFilters"
								>
									Clear All Filters
								</UButton>
							</UBadge>
						</div>

						<LiveMonitor
							:events="filteredEventsComputed"
							:auto-scroll="autoScroll"
							:total-events-count="events.length"
						/>
					</div>

					<!-- Right Column: Monitoring Scope Info -->
					<div class="flex flex-col overflow-hidden">
						<MonitoringScopeInfo
							:connection-status="monitoringConnectionStatus"
							:total-events="events.length"
							:current-buffer-size="events.length"
							:last-event-time="lastEventTime"
						/>
					</div>
				</div>

				<!-- Completed/Failed Requests Section (Collapsible) -->
				<div v-if="completedRequests.length > 0 || failedRequests.length > 0" class="mt-4">
					<UAccordion :items="historyAccordionItems">
						<template #completed>
							<div class="space-y-2 p-4">
								<GenerationProgressCard
									v-for="request in completedRequests"
									:key="request.requestId"
									:request="request"
								/>
							</div>
						</template>
						<template #failed>
							<div class="space-y-2 p-4">
								<GenerationProgressCard
									v-for="request in failedRequests"
									:key="request.requestId"
									:request="request"
								/>
							</div>
						</template>
					</UAccordion>
				</div>
			</div>

			<!-- Raw NATS Console - Always visible for debugging -->
			<div class="mt-6 px-6 pb-6">
				<NatsConsole
					:events="events"
					:auto-scroll="autoScroll"
				/>
			</div>
		</div>
	</UContainer>
</template>

<script lang="ts" setup>
	import type { PhaseProgress, TrackedRequest } from "~/types/trails";
	import { invoke } from "@tauri-apps/api/core";
	import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";
	import { EVENT_TYPES, MCP_SERVERS, NETWORK, TIME_RANGE_OPTIONS, TIME_RANGES } from "~/config/constants";

	definePageMeta({
		layout: "default",
		name: "Live Monitor",
		description: "Real-time NATS generation monitoring",
		icon: "i-heroicons-radio",
		category: "monitoring",
		showInNav: true
	});

	// Router for navigation
	const route = useRoute();
	const router = useRouter();

	// Use the NATS live monitor composable
	const {
		events,
		requests,
		connectionStatus,
		error,
		loading,
		isPaused,
		autoScroll,
		completedRequests,
		failedRequests,
		statistics,
		uniqueTenantIds,
		subscribe,
		unsubscribe,
		clearAll,
		pause,
		resume,
		cancel
	} = useNatsLiveMonitor();

	// Notification permission state
	const notificationPermissionGranted = ref(false);

	// Request ID filter from URL
	const filterRequestId = ref<string | null>(null);

	// Filter states
	const selectedEventTypes = ref<string[]>([]);
	const selectedTenants = ref<string[]>([]);
	const selectedComponents = ref<string[]>([]);
	const selectedTimeRange = ref<string>("all");
	const requestIdByComponent = ref<Record<string, string>>({});

	// Active requests from backend
	const activeRequestsData = ref<TrackedRequest[]>([]);

	// Convert TrackedRequest to GenerationRequest for display compatibility
	const activeRequestsForDisplay = computed(() => {
		return activeRequestsData.value.map((tracked) => {
			// Create a GenerationRequest-compatible object
			const phases = new Map<string, PhaseProgress>();
			phases.set(tracked.currentPhase, {
				phase: tracked.currentPhase,
				status: tracked.status,
				progress: tracked.progress,
				startTime: tracked.startTime,
				endTime: tracked.status === "completed" || tracked.status === "failed" ? tracked.lastUpdate : undefined,
				errorMessage: tracked.errorMessage
			});

			return {
				requestId: tracked.requestId,
				tenantId: tracked.tenantId,
				startTime: tracked.startTime,
				phases,
				status: tracked.status,
				errorMessage: tracked.errorMessage
			};
		});
	});

	// Event type options
	const eventTypeOptions = computed(() => {
		return EVENT_TYPES.map((type) => ({ label: type, value: type }));
	});

	// Tenant filter options
	const tenantOptions = computed(() => {
		return uniqueTenantIds.value.map((id) => ({ label: id, value: id }));
	});

	// Component filter options
	const componentOptions = computed(() => {
		return MCP_SERVERS.map((server) => ({ label: server, value: server }));
	});

	// Time range options
	const timeRangeOptions = TIME_RANGE_OPTIONS;

	// Active filter count
	const activeFilterCount = computed(() => {
		let count = 0;
		if (filterRequestId.value) count++;
		if (selectedEventTypes.value.length > 0) count++;
		if (selectedTenants.value.length > 0) count++;
		if (selectedComponents.value.length > 0) count++;
		if (selectedTimeRange.value !== "all") count++;
		if (Object.keys(requestIdByComponent.value).some((key) => requestIdByComponent.value[key])) count++;
		return count;
	});

	// Filtered events computed property
	const filteredEventsComputed = computed(() => {
		let filtered = events.value;

		// Filter by request ID from URL (takes priority)
		if (filterRequestId.value) {
			filtered = filtered.filter((e) => {
				try {
					// Check if the event's requestId matches
					if (e.requestId && e.requestId.includes(filterRequestId.value!)) {
						return true;
					}

					// Also check payload for request-id in case it's in the data
					const payload = e.payload || e.data || e;
					const payloadStr = typeof payload === "string"
						? payload
						: JSON.stringify(payload);
					return payloadStr.includes(filterRequestId.value!);
				} catch (err) {
					return false;
				}
			});
		}

		// Filter by event type
		if (selectedEventTypes.value.length > 0) {
			filtered = filtered.filter((e) => selectedEventTypes.value.includes(e.eventType));
		}

		// Filter by tenant
		if (selectedTenants.value.length > 0) {
			filtered = filtered.filter((e) => selectedTenants.value.includes(e.tenantId));
		}

		// Filter by component (servicePhase)
		if (selectedComponents.value.length > 0) {
			filtered = filtered.filter((e) => selectedComponents.value.includes(e.servicePhase));
		}

		// Filter by time range
		if (selectedTimeRange.value !== "all") {
			const now = Date.now();
			const timeRangeKey = selectedTimeRange.value.toUpperCase().replace("-", "_") as keyof typeof TIME_RANGES;
			const cutoff = now - TIME_RANGES[timeRangeKey];

			filtered = filtered.filter((e) => {
				const eventTime = new Date(e.timestamp).getTime();
				return eventTime >= cutoff;
			});
		}

		// Filter by request ID per component
		const requestIdFilters = Object.entries(requestIdByComponent.value).filter(([, id]) => id);
		if (requestIdFilters.length > 0) {
			filtered = filtered.filter((e) => {
				const requestId = requestIdByComponent.value[e.servicePhase];
				return !requestId || e.requestId.startsWith(requestId);
			});
		}

		return filtered;
	});

	// Last event time
	const lastEventTime = computed(() => {
		if (events.value.length === 0) return null;
		return events.value[0].timestamp;
	});

	// Monitoring connection status
	const monitoringConnectionStatus = computed(() => ({
		connected: connectionStatus.value.connected,
		url: NETWORK.NATS_URL,
		activeRequests: activeRequestsData.value.length
	}));

	// History accordion items
	const historyAccordionItems = computed(() => {
		const items = [];

		if (completedRequests.value.length > 0) {
			items.push({
				label: `Completed Requests (${completedRequests.value.length})`,
				icon: "i-heroicons-check-circle",
				slot: "completed",
				defaultOpen: false
			});
		}

		if (failedRequests.value.length > 0) {
			items.push({
				label: `Failed Requests (${failedRequests.value.length})`,
				icon: "i-heroicons-x-circle",
				slot: "failed",
				defaultOpen: false
			});
		}

		return items;
	});

	// Watch for completed/failed requests to send notifications
	watch(requests, (newRequests) => {
		for (const [_requestId, request] of newRequests.entries()) {
			if (request.status === "completed" || request.status === "failed") {
				sendGenerationNotification(request.requestId, request.status, request.tenantId);
			}
		}
	}, { deep: true });

	// Handle subscribe
	async function handleSubscribe() {
		await subscribe();
	}

	// Handle unsubscribe
	async function handleUnsubscribe() {
		await unsubscribe();
	}

	// Handle pause/resume
	function handlePauseResume() {
		if (isPaused.value) {
			resume();
		} else {
			pause();
		}
	}

	// Handle cancel
	async function handleCancel() {
		// eslint-disable-next-line no-alert
		const confirmed = confirm("Are you sure you want to cancel the subscription and clear all data?");
		if (confirmed) {
			await cancel();
		}
	}

	// Clear request ID filter
	function clearFilter() {
		filterRequestId.value = null;
		router.replace("/live-monitor"); // Remove query param from URL
	}

	// Handle clear filters
	function handleClearFilters() {
		filterRequestId.value = null;
		selectedEventTypes.value = [];
		selectedTenants.value = [];
		selectedComponents.value = [];
		selectedTimeRange.value = "all";
		requestIdByComponent.value = {};
		router.replace("/live-monitor"); // Remove query param from URL
	}

	// Set request ID for component
	function setRequestIdForComponent(component: string, requestId: string) {
		if (requestId) {
			requestIdByComponent.value[component] = requestId;
		} else {
			delete requestIdByComponent.value[component];
		}
	}

	// Load active requests from backend
	async function loadActiveRequests() {
		try {
			activeRequestsData.value = await invoke<TrackedRequest[]>("get_active_requests");
		} catch (err) {
			console.error("[LiveMonitor] Failed to load active requests:", err);
		}
	}

	// Send desktop notification for generation completion/failure
	const notifiedRequests = new Set<string>();

	async function sendGenerationNotification(requestId: string, status: string, tenantId: string) {
		// Skip if already notified
		if (notifiedRequests.has(requestId)) return;
		notifiedRequests.add(requestId);

		// Check notification permission
		if (!notificationPermissionGranted.value) return;

		try {
			const title = status === "completed" ? "Generation Complete" : "Generation Failed";
			const body = `Request ${requestId.substring(0, 8)}... (Tenant: ${tenantId})`;

			await sendNotification({
				title,
				body
			});
		} catch (err) {
			console.error("[LiveMonitor] Failed to send notification:", err);
		}
	}

	// Refresh active requests every 5 seconds
	let refreshInterval: NodeJS.Timeout | null = null;

	onMounted(async () => {
		// TEMPORARILY DISABLED: Request ID filter from URL
		// There's a backend bug where the orchestrator generates new UUIDs instead of preserving
		// the frontend-generated request_id. This causes the filter to never match any events.
		// TODO: Re-enable once backend preserves request_id throughout the pipeline
		/*
		const requestId = route.query.requestId;
		if (requestId && typeof requestId === "string") {
			filterRequestId.value = requestId;
			console.log("[LiveMonitor] Filtering by request ID from URL:", requestId);
		}
		*/
		console.warn("[LiveMonitor] Request ID filtering disabled due to backend ID mismatch issue");

		// Request notification permission
		try {
			let granted = await isPermissionGranted();
			if (!granted) {
				const permission = await requestPermission();
				granted = permission === "granted";
			}
			notificationPermissionGranted.value = granted;
			console.log("[LiveMonitor] Notification permission:", granted);
		} catch (err) {
			console.error("[LiveMonitor] Failed to request notification permission:", err);
		}

		// Start refreshing active requests
		await loadActiveRequests();
		refreshInterval = setInterval(loadActiveRequests, 5000);
	});

	onUnmounted(() => {
		if (refreshInterval) {
			clearInterval(refreshInterval);
		}
	});
</script>

<style scoped>
/* Custom scrollbar styling for the main container */
.overflow-y-auto {
  scrollbar-width: thin;
  scrollbar-color: rgba(156, 163, 175, 0.5) transparent;
}

.overflow-y-auto::-webkit-scrollbar {
  width: 8px;
}

.overflow-y-auto::-webkit-scrollbar-track {
  background: transparent;
}

.overflow-y-auto::-webkit-scrollbar-thumb {
  background-color: rgba(156, 163, 175, 0.5);
  border-radius: 4px;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover {
  background-color: rgba(156, 163, 175, 0.7);
}

.filter-group {
  min-width: 0;
}
</style>
