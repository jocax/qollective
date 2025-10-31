<template>
	<div class="flex flex-col h-full">
		<!-- Header -->
		<div class="mb-4">
			<h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
				Monitoring Scope
			</h3>
			<p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
				System status and connection information
			</p>
		</div>

		<!-- Info Cards Container -->
		<div class="flex-1 overflow-y-auto space-y-3 pr-2">
			<!-- NATS Connection Card -->
			<UCard>
				<div class="space-y-3">
					<div class="flex items-center justify-between">
						<h4 class="text-sm font-semibold text-gray-700 dark:text-gray-300">
							NATS Connection
						</h4>
						<UBadge
							:color="connectionStatus.connected ? 'green' : 'red'"
							variant="subtle"
						>
							{{ connectionStatus.connected ? 'Connected' : 'Disconnected' }}
						</UBadge>
					</div>

					<div class="space-y-2 text-xs">
						<div class="flex items-center gap-2">
							<UIcon name="i-heroicons-server" class="text-gray-400" />
							<span class="text-gray-600 dark:text-gray-400 font-mono text-[10px]">
								{{ connectionStatus.url }}
							</span>
						</div>

						<div class="flex items-center gap-2">
							<UIcon name="i-heroicons-chart-bar" class="text-gray-400" />
							<span class="text-gray-600 dark:text-gray-400">
								Active requests: <strong>{{ connectionStatus.activeRequests }}</strong>
							</span>
						</div>

						<div v-if="connectionStatus.connected" class="flex items-center gap-2">
							<UIcon name="i-heroicons-check-circle" class="text-green-500" />
							<span class="text-green-600 dark:text-green-400">
								Receiving events
							</span>
						</div>
					</div>
				</div>
			</UCard>

			<!-- Subscriptions Card -->
			<UCard>
				<div class="space-y-3">
					<h4 class="text-sm font-semibold text-gray-700 dark:text-gray-300">
						NATS Subscriptions
					</h4>

					<div class="space-y-1">
						<div
							v-for="subject in natsSubjects"
							:key="subject"
							class="flex items-start gap-2 text-xs"
						>
							<UIcon name="i-heroicons-rss" class="text-blue-500 mt-0.5 flex-shrink-0" />
							<span class="text-gray-600 dark:text-gray-400 font-mono break-all">
								{{ subject }}
							</span>
						</div>
					</div>

					<div class="pt-2 border-t border-gray-200 dark:border-gray-700">
						<p class="text-xs text-gray-500 dark:text-gray-500">
							Listening to all generation events across pipeline
						</p>
					</div>
				</div>
			</UCard>

			<!-- Event Statistics Card -->
			<UCard>
				<div class="space-y-3">
					<h4 class="text-sm font-semibold text-gray-700 dark:text-gray-300">
						Event Statistics
					</h4>

					<div class="space-y-2">
						<div class="flex items-center justify-between text-xs">
							<span class="text-gray-600 dark:text-gray-400">Total events received</span>
							<span class="font-semibold text-gray-900 dark:text-gray-100">
								{{ totalEvents }}
							</span>
						</div>

						<div class="flex items-center justify-between text-xs">
							<span class="text-gray-600 dark:text-gray-400">Events in buffer</span>
							<span class="font-semibold text-gray-900 dark:text-gray-100">
								{{ currentBufferSize }} / {{ maxBufferSize }}
							</span>
						</div>

						<UProgress
							:value="bufferUsagePercent"
							:color="bufferUsagePercent > 90 ? 'red' : 'blue'"
							size="xs"
						/>

						<div v-if="lastEventTime" class="flex items-center justify-between text-xs pt-2 border-t border-gray-200 dark:border-gray-700">
							<span class="text-gray-600 dark:text-gray-400">Last event</span>
							<span class="font-semibold text-gray-900 dark:text-gray-100 font-mono text-[10px]">
								{{ formattedLastEventTime }}
							</span>
						</div>

						<div v-else class="text-xs text-gray-500 dark:text-gray-500 text-center pt-2 border-t border-gray-200 dark:border-gray-700">
							No events received yet
						</div>
					</div>

					<!-- Buffer Warning -->
					<UAlert
						v-if="bufferUsagePercent > 80"
						color="orange"
						variant="subtle"
						class="text-xs"
					>
						<template #description>
							Buffer usage high ({{ bufferUsagePercent }}%). Older events will be dropped.
						</template>
					</UAlert>
				</div>
			</UCard>

			<!-- MCP Components Card -->
			<UCard>
				<div class="space-y-3">
					<h4 class="text-sm font-semibold text-gray-700 dark:text-gray-300">
						MCP Components
					</h4>

					<div class="space-y-1">
						<div
							v-for="component in mcpComponents"
							:key="component.id"
							class="flex items-center gap-2 text-xs p-2 rounded bg-gray-50 dark:bg-gray-800"
						>
							<UIcon :name="component.icon" class="text-gray-500 flex-shrink-0" />
							<span class="text-gray-700 dark:text-gray-300 font-medium">
								{{ component.label }}
							</span>
						</div>
					</div>

					<div class="pt-2 border-t border-gray-200 dark:border-gray-700">
						<p class="text-xs text-gray-500 dark:text-gray-500">
							{{ mcpComponents.length }} components in TaleTrail pipeline
						</p>
					</div>
				</div>
			</UCard>

			<!-- System Info Card -->
			<UCard>
				<div class="space-y-3">
					<h4 class="text-sm font-semibold text-gray-700 dark:text-gray-300">
						System Information
					</h4>

					<div class="space-y-2 text-xs">
						<div class="flex items-center justify-between">
							<span class="text-gray-600 dark:text-gray-400">Buffer Size Limit</span>
							<span class="font-semibold text-gray-900 dark:text-gray-100 font-mono">
								{{ maxBufferSize }} events
							</span>
						</div>

						<div class="flex items-center justify-between">
							<span class="text-gray-600 dark:text-gray-400">Event Types</span>
							<span class="font-semibold text-gray-900 dark:text-gray-100">
								{{ eventTypes.length }} types
							</span>
						</div>

						<div class="flex items-center justify-between">
							<span class="text-gray-600 dark:text-gray-400">Auto-scroll</span>
							<UBadge :color="autoScroll ? 'green' : 'gray'" variant="subtle" size="xs">
								{{ autoScroll ? 'Enabled' : 'Disabled' }}
							</UBadge>
						</div>
					</div>
				</div>
			</UCard>
		</div>
	</div>
</template>

<script setup lang="ts">
	import { EVENT_TYPES, MAX_EVENT_BUFFER_SIZE, NATS_SUBJECTS } from "~/config/constants";

	interface ConnectionStatus {
		connected: boolean
		url: string
		activeRequests: number
	}

	interface Props {
		connectionStatus: ConnectionStatus
		totalEvents: number
		currentBufferSize: number
		lastEventTime: string | null
		autoScroll?: boolean
	}

	const props = withDefaults(defineProps<Props>(), {
		autoScroll: true
	});

	// NATS subjects being monitored
	const natsSubjects = [
		NATS_SUBJECTS.GENERATION_REQUESTS,
		NATS_SUBJECTS.GENERATION_EVENTS_PATTERN
	];

	// MCP Components with icons
	const mcpComponents = [
		{ id: "orchestrator", label: "Orchestrator", icon: "i-heroicons-command-line" },
		{ id: "prompt-helper", label: "Prompt Helper", icon: "i-heroicons-sparkles" },
		{ id: "story-generator", label: "Story Generator", icon: "i-heroicons-book-open" },
		{ id: "quality-control", label: "Quality Control", icon: "i-heroicons-shield-check" },
		{ id: "constraint-enforcer", label: "Constraint Enforcer", icon: "i-heroicons-check-badge" }
	];

	// Event types
	const eventTypes = EVENT_TYPES;

	// Max buffer size
	const maxBufferSize = MAX_EVENT_BUFFER_SIZE;

	// Buffer usage percentage
	const bufferUsagePercent = computed(() => {
		if (maxBufferSize === 0) return 0;
		return Math.round((props.currentBufferSize / maxBufferSize) * 100);
	});

	// Formatted last event time
	const formattedLastEventTime = computed(() => {
		if (!props.lastEventTime) return "N/A";

		try {
			const date = new Date(props.lastEventTime);
			return date.toLocaleString("en-US", {
				month: "short",
				day: "numeric",
				hour: "2-digit",
				minute: "2-digit",
				second: "2-digit",
				fractionalSecondDigits: 3
			});
		} catch {
			return props.lastEventTime;
		}
	});
</script>

<style scoped>
/* Custom scrollbar styling */
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
</style>
