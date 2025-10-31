<template>
	<div class="flex flex-col h-full">
		<!-- Header -->
		<div class="mb-4">
			<div class="flex items-center justify-between">
				<h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
					Live Event Stream
				</h3>
				<UBadge color="blue" variant="subtle">
					{{ events.length }} events
				</UBadge>
			</div>
			<p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
				Real-time generation events (newest first)
			</p>
		</div>

		<!-- Event List Container -->
		<div
			ref="eventListRef"
			class="flex-1 overflow-y-auto space-y-2 pr-2"
			:class="{ 'scroll-smooth': autoScroll }"
		>
			<!-- Filtered Out Empty State -->
			<div
				v-if="props.totalEventsCount > 0 && props.events.length === 0"
				class="flex items-center justify-center h-full"
			>
				<div class="text-center p-8 max-w-md">
					<UIcon name="i-heroicons-funnel" class="w-16 h-16 text-yellow-500 mx-auto mb-4" />
					<h4 class="text-lg font-semibold text-gray-700 dark:text-gray-300 mb-2">
						All Events Filtered Out
					</h4>
					<p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
						{{ props.totalEventsCount }} {{ props.totalEventsCount === 1 ? 'event is' : 'events are' }} hidden by active filters.
					</p>
					<div class="bg-yellow-50 dark:bg-yellow-900/20 p-4 rounded-lg border-2 border-yellow-300 dark:border-yellow-700 text-left mb-4">
						<p class="font-semibold text-yellow-800 dark:text-yellow-200 mb-2">
							💡 Tip:
						</p>
						<p class="text-sm text-yellow-700 dark:text-yellow-300">
							Look for the yellow warning alert above and click
							<span class="font-bold">"Show All Events"</span> to clear the request ID filter.
						</p>
					</div>
				</div>
			</div>

			<!-- No Events Empty State -->
			<div v-if="props.totalEventsCount === 0" class="flex items-center justify-center h-full">
				<div class="text-center p-8">
					<UIcon name="i-heroicons-inbox" class="w-16 h-16 text-gray-400 mx-auto mb-4" />
					<h4 class="text-lg font-semibold text-gray-700 dark:text-gray-300 mb-2">
						No events yet
					</h4>
					<p class="text-sm text-gray-600 dark:text-gray-400">
						Waiting for generation events to arrive...
					</p>
				</div>
			</div>

			<!-- Event Cards -->
			<div
				v-for="(event, index) in displayEvents"
				:key="`${event.requestId}-${event.timestamp}-${index}`"
				class="group relative"
			>
				<UCard class="transition-all duration-200 hover:shadow-md">
					<div class="space-y-2">
						<!-- Event Header -->
						<div class="flex items-start justify-between">
							<div class="flex items-center gap-2 flex-1 min-w-0">
								<UIcon
									:name="getPhaseIcon(event.servicePhase)"
									class="text-gray-500 flex-shrink-0"
								/>
								<div class="flex-1 min-w-0">
									<div class="flex items-center gap-2">
										<span class="text-sm font-semibold text-gray-900 dark:text-gray-100">
											{{ formatPhaseLabel(event.servicePhase) }}
										</span>
										<UBadge
											:color="getStatusColor(event.status)"
											variant="soft"
											size="xs"
										>
											{{ event.status }}
										</UBadge>
									</div>
									<div class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
										{{ formatTimestamp(event.timestamp) }}
									</div>
								</div>
							</div>
						</div>

						<!-- Event Details -->
						<div class="grid grid-cols-2 gap-2 text-xs">
							<div class="flex items-center gap-1">
								<UIcon name="i-heroicons-hashtag" class="text-gray-400 w-3 h-3" />
								<span class="text-gray-600 dark:text-gray-400 truncate">
									Request: {{ event.requestId.substring(0, 8) }}...
								</span>
							</div>
							<div class="flex items-center gap-1">
								<UIcon name="i-heroicons-building-office" class="text-gray-400 w-3 h-3" />
								<span class="text-gray-600 dark:text-gray-400 truncate">
									Tenant: {{ event.tenantId }}
								</span>
							</div>
						</div>

						<!-- Progress Bar (if in progress and has progress value) -->
						<div v-if="event.status === 'in_progress' && event.progress !== undefined" class="space-y-1">
							<div class="flex items-center justify-between text-xs">
								<span class="text-gray-600 dark:text-gray-400">Progress</span>
								<span class="font-semibold text-gray-700 dark:text-gray-300">
									{{ formatProgress(event.progress) }}
								</span>
							</div>
							<UProgress
								:value="Math.round((event.progress ?? 0) * 100)"
								:color="getStatusColor(event.status)"
								size="xs"
							/>
						</div>

						<!-- Error Message -->
						<div v-if="event.errorMessage" class="mt-2">
							<UAlert
								color="red"
								variant="subtle"
								:description="event.errorMessage"
								class="text-xs"
							/>
						</div>

						<!-- Event Type Badge -->
						<div class="flex items-center gap-2 pt-2 border-t border-gray-200 dark:border-gray-700">
							<UBadge color="gray" variant="outline" size="xs">
								{{ event.eventType }}
							</UBadge>
						</div>
					</div>
				</UCard>
			</div>
		</div>

		<!-- Scroll Indicator -->
		<div
			v-if="events.length > 5 && !autoScroll"
			class="mt-2 text-center"
		>
			<UButton
				variant="outline"
				size="xs"
				icon="i-heroicons-arrow-up"
				@click="eventListRef?.scrollTo({ top: 0, behavior: 'smooth' })"
			>
				Scroll to latest
			</UButton>
		</div>
	</div>
</template>

<script setup lang="ts">
	import type { GenerationEvent } from "~/types/trails";

	interface Props {
		events: GenerationEvent[]
		autoScroll?: boolean
		totalEventsCount?: number
	}

	const props = withDefaults(defineProps<Props>(), {
		autoScroll: true,
		totalEventsCount: 0
	});

	// Event list container ref for auto-scrolling
	const eventListRef = ref<HTMLElement | null>(null);

	// Watch for new events and auto-scroll to top (newest first)
	watch(
		() => props.events.length,
		() => {
			if (props.autoScroll && eventListRef.value) {
				nextTick(() => {
					// Scroll to top since events are displayed newest-first
					eventListRef.value?.scrollTo({
						top: 0,
						behavior: "smooth"
					});
				});
			}
		}
	);

	// Status color mapping
	function getStatusColor(status: string): string {
		switch (status) {
		case "completed":
			return "green";
		case "failed":
			return "red";
		case "in_progress":
			return "blue";
		default:
			return "gray";
		}
	}

	// Phase icon mapping
	function getPhaseIcon(phase: string): string {
		switch (phase) {
		case "prompt-helper":
			return "i-heroicons-sparkles";
		case "story-generator":
			return "i-heroicons-book-open";
		case "quality-control":
			return "i-heroicons-shield-check";
		case "constraint-enforcer":
			return "i-heroicons-check-badge";
		default:
			return "i-heroicons-cog";
		}
	}

	// Format phase label
	function formatPhaseLabel(phase: string): string {
		return phase
			.split("-")
			.map((word) => word.charAt(0).toUpperCase() + word.slice(1))
			.join(" ");
	}

	// Format timestamp
	function formatTimestamp(timestamp: string): string {
		try {
			const date = new Date(timestamp);
			return date.toLocaleTimeString("en-US", {
				hour: "2-digit",
				minute: "2-digit",
				second: "2-digit",
				fractionalSecondDigits: 3
			});
		} catch {
			return timestamp;
		}
	}

	// Format progress percentage
	function formatProgress(progress?: number): string {
		if (progress === undefined) return "N/A";
		return `${Math.round(progress * 100)}%`;
	}

	// Events are already in newest-first order from useNatsLiveMonitor
	// (events.unshift() adds to beginning of array)
	const displayEvents = computed(() => {
		return props.events;
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

/* Smooth scroll */
.scroll-smooth {
  scroll-behavior: smooth;
}
</style>
