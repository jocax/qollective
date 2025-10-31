<template>
	<div class="nats-console">
		<UCard>
			<template #header>
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-3">
						<UIcon name="i-heroicons-code-bracket" class="w-5 h-5 text-gray-500 dark:text-gray-400" />
						<h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
							Raw NATS Messages
						</h3>
						<UBadge :color="events.length > 0 ? 'blue' : 'gray'" variant="subtle">
							{{ events.length }} {{ events.length === 1 ? 'message' : 'messages' }}
						</UBadge>
					</div>
					<div class="flex items-center gap-2">
						<!-- Search Input -->
						<UInput
							v-model="searchQuery"
							placeholder="Search JSON..."
							size="sm"
							icon="i-heroicons-magnifying-glass"
							class="w-48"
							@input="handleSearch"
						/>
						<!-- Copy All Button -->
						<UButton
							variant="ghost"
							size="sm"
							icon="i-heroicons-document-duplicate"
							:disabled="events.length === 0"
							@click="copyAllMessages"
						>
							Copy All
						</UButton>
						<!-- Clear Button -->
						<UButton
							variant="ghost"
							size="sm"
							icon="i-heroicons-trash"
							color="red"
							:disabled="events.length === 0"
							@click="handleClear"
						>
							Clear
						</UButton>
						<!-- Collapse/Expand Button -->
						<UButton
							variant="ghost"
							size="sm"
							:icon="isExpanded ? 'i-heroicons-chevron-up' : 'i-heroicons-chevron-down'"
							@click="toggleExpanded"
						/>
					</div>
				</div>
			</template>

			<!-- Console Output Area -->
			<div v-show="isExpanded" class="console-wrapper">
				<!-- Empty State -->
				<div v-if="filteredEvents.length === 0 && events.length === 0" class="console-empty">
					<UIcon name="i-heroicons-terminal" class="w-12 h-12 text-gray-400 mx-auto mb-3" />
					<p class="text-sm text-gray-500 dark:text-gray-400">
						No messages to display
					</p>
				</div>

				<!-- No Search Results -->
				<div v-else-if="filteredEvents.length === 0 && searchQuery" class="console-empty">
					<UIcon name="i-heroicons-magnifying-glass" class="w-12 h-12 text-gray-400 mx-auto mb-3" />
					<p class="text-sm text-gray-500 dark:text-gray-400">
						No messages match "{{ searchQuery }}"
					</p>
					<UButton variant="ghost" size="xs" @click="searchQuery = ''">
						Clear search
					</UButton>
				</div>

				<!-- Console Messages -->
				<div
					v-else
					ref="consoleOutputRef"
					class="console-output"
					:class="{ 'scroll-smooth': autoScroll }"
				>
					<div
						v-for="(event, index) in filteredEvents"
						:key="`console-${event.requestId}-${event.timestamp}-${index}`"
						class="console-message"
					>
						<!-- Message Header -->
						<div class="console-message-header">
							<div class="flex items-center gap-2 flex-1 min-w-0">
								<span class="console-timestamp">
									{{ formatConsoleTimestamp(event.timestamp) }}
								</span>
								<UBadge
									:color="getStatusColor(event.status)"
									variant="soft"
									size="xs"
								>
									{{ event.eventType }}
								</UBadge>
								<span class="console-phase">
									[{{ event.servicePhase }}]
								</span>
								<span class="console-request-id">
									{{ event.requestId.substring(0, 12) }}...
								</span>
							</div>
							<UButton
								variant="ghost"
								size="xs"
								icon="i-heroicons-clipboard"
								@click="copyMessage(event, index)"
							>
								Copy
							</UButton>
						</div>

						<!-- JSON Content -->
						<div class="console-json">
							<pre v-html="syntaxHighlight(event)" />
						</div>
					</div>
				</div>
			</div>
		</UCard>

		<!-- Success Toast (using Nuxt UI toast) -->
		<UNotifications />
	</div>
</template>

<script setup lang="ts">
	import type { GenerationEvent } from "~/types/trails";

	interface Props {
		events: GenerationEvent[]
		autoScroll?: boolean
	}

	const props = withDefaults(defineProps<Props>(), {
		autoScroll: true
	});

	const toast = useToast();

	// Component state
	const isExpanded = ref(false); // Start collapsed
	const searchQuery = ref("");
	const consoleOutputRef = ref<HTMLElement | null>(null);

	// Watch for new events and auto-scroll to top (newest first)
	watch(
		() => props.events.length,
		() => {
			if (props.autoScroll && isExpanded.value && consoleOutputRef.value) {
				nextTick(() => {
					consoleOutputRef.value?.scrollTo({
						top: 0,
						behavior: "smooth"
					});
				});
			}
		}
	);

	// Filtered events based on search query
	const filteredEvents = computed(() => {
		if (!searchQuery.value.trim()) {
			return props.events;
		}

		const query = searchQuery.value.toLowerCase();
		return props.events.filter((event) => {
			const jsonStr = JSON.stringify(event).toLowerCase();
			return jsonStr.includes(query);
		});
	});

	// Toggle expanded state
	function toggleExpanded() {
		isExpanded.value = !isExpanded.value;
	}

	// Handle search input
	function handleSearch() {
		// Search is handled by computed property
		// This function exists for potential future enhancements
	}

	// Clear console
	function handleClear() {
		// eslint-disable-next-line no-alert
		const confirmed = confirm("Clear all console messages?");
		if (confirmed) {
			// Note: We can't directly modify props.events
			// The parent component (live-monitor.vue) should handle clearing
			// For now, we'll show a toast
			toast.add({
				title: "Clear Action",
				description: "Use 'Clear Events' button in the toolbar to clear all events",
				color: "blue",
				timeout: 3000
			});
		}
	}

	// Copy single message
	async function copyMessage(event: GenerationEvent, index: number) {
		try {
			const jsonStr = JSON.stringify(event, null, 2);
			await navigator.clipboard.writeText(jsonStr);
			toast.add({
				title: "Copied",
				description: `Message ${index + 1} copied to clipboard`,
				color: "green",
				timeout: 2000
			});
		} catch (err) {
			console.error("Failed to copy message:", err);
			toast.add({
				title: "Copy Failed",
				description: "Failed to copy to clipboard",
				color: "red",
				timeout: 2000
			});
		}
	}

	// Copy all messages
	async function copyAllMessages() {
		try {
			const jsonStr = JSON.stringify(props.events, null, 2);
			await navigator.clipboard.writeText(jsonStr);
			toast.add({
				title: "Copied",
				description: `${props.events.length} messages copied to clipboard`,
				color: "green",
				timeout: 2000
			});
		} catch (err) {
			console.error("Failed to copy all messages:", err);
			toast.add({
				title: "Copy Failed",
				description: "Failed to copy to clipboard",
				color: "red",
				timeout: 2000
			});
		}
	}

	// Format timestamp for console (like browser DevTools)
	function formatConsoleTimestamp(timestamp: string): string {
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

	// Syntax highlight JSON
	function syntaxHighlight(obj: any): string {
		let json = JSON.stringify(obj, null, 2);

		// Escape HTML
		json = json
			.replace(/&/g, "&amp;")
			.replace(/</g, "&lt;")
			.replace(/>/g, "&gt;");

		// Apply syntax highlighting with Tailwind classes
		return json.replace(
			/("(\\u[a-zA-Z0-9]{4}|\\[^u]|[^\\"])*"(\s*:)?|\b(true|false|null)\b|-?\d+(?:\.\d*)?(?:[eE][+-]?\d+)?)/g,
			(match) => {
				let cls = "text-yellow-400"; // numbers

				if (match.startsWith("\"")) {
					if (match.endsWith(":")) {
						// Keys
						cls = "text-blue-400 font-medium";
					} else {
						// Strings
						cls = "text-green-400";
					}
				} else if (/true|false/.test(match)) {
					// Booleans
					cls = "text-purple-400";
				} else if (/null/.test(match)) {
					// Null
					cls = "text-gray-400";
				}

				return `<span class="${cls}">${match}</span>`;
			}
		);
	}
</script>

<style scoped>
.nats-console {
	margin-top: 1.5rem;
}

/* Console wrapper */
.console-wrapper {
	margin-top: 0.75rem;
}

/* Empty state */
.console-empty {
	display: flex;
	flex-direction: column;
	align-items: center;
	justify-content: center;
	padding: 3rem 1rem;
	text-align: center;
}

/* Console output container */
.console-output {
	background-color: #1a1a1a;
	border-radius: 0.5rem;
	padding: 0.75rem;
	max-height: 600px;
	overflow-y: auto;
	font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
	font-size: 0.8125rem;
	line-height: 1.4;
	scrollbar-width: thin;
	scrollbar-color: rgba(156, 163, 175, 0.3) transparent;
}

.console-output::-webkit-scrollbar {
	width: 6px;
}

.console-output::-webkit-scrollbar-track {
	background: transparent;
}

.console-output::-webkit-scrollbar-thumb {
	background-color: rgba(156, 163, 175, 0.3);
	border-radius: 3px;
}

.console-output::-webkit-scrollbar-thumb:hover {
	background-color: rgba(156, 163, 175, 0.5);
}

.scroll-smooth {
	scroll-behavior: smooth;
}

/* Console message */
.console-message {
	margin-bottom: 1rem;
	padding-bottom: 1rem;
	border-bottom: 1px solid rgba(75, 85, 99, 0.3);
}

.console-message:last-child {
	border-bottom: none;
	margin-bottom: 0;
	padding-bottom: 0;
}

/* Message header */
.console-message-header {
	display: flex;
	align-items: center;
	justify-content: space-between;
	margin-bottom: 0.5rem;
	padding: 0.25rem 0;
}

.console-timestamp {
	color: #9ca3af;
	font-size: 0.75rem;
	font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
}

.console-phase {
	color: #6b7280;
	font-size: 0.75rem;
	font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
}

.console-request-id {
	color: #6b7280;
	font-size: 0.75rem;
	font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
}

/* JSON content */
.console-json {
	background-color: #0d0d0d;
	border-radius: 0.375rem;
	padding: 0.75rem;
	overflow-x: auto;
}

.console-json pre {
	margin: 0;
	font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
	font-size: 0.8125rem;
	line-height: 1.5;
	color: #e5e7eb;
	white-space: pre;
	word-wrap: normal;
}

/* Responsive adjustments */
@media (max-width: 768px) {
	.console-output {
		max-height: 400px;
		font-size: 0.75rem;
	}

	.console-json pre {
		font-size: 0.75rem;
	}
}
</style>
