<template>
	<UCard
		class="hover:shadow-md transition-shadow duration-200" :class="[
			messageTypeColor,
			isExpanded ? 'ring-2 ring-primary-500' : ''
		]"
	>
		<div class="space-y-2">
			<!-- Message Header -->
			<div class="flex items-start justify-between gap-3">
				<div class="flex items-center gap-2 flex-1 min-w-0">
					<!-- Timestamp -->
					<span class="text-xs font-mono text-gray-500 dark:text-gray-400 whitespace-nowrap">
						[{{ formattedTime }}]
					</span>

					<!-- Endpoint Badge -->
					<UBadge :color="endpointColor" variant="subtle" size="xs">
						{{ message.endpoint }}
					</UBadge>

					<!-- Message Type Badge -->
					<UBadge :color="typeColor" variant="soft" size="xs">
						{{ message.message_type }}
					</UBadge>

					<!-- Request ID (if present) -->
					<span v-if="message.request_id" class="text-xs font-mono text-gray-600 dark:text-gray-300 truncate">
						req-id: {{ message.request_id }}
					</span>
				</div>

				<!-- Expand/Collapse Toggle -->
				<UButton
					variant="ghost"
					size="xs"
					:icon="isExpanded ? 'i-heroicons-chevron-up' : 'i-heroicons-chevron-down'"
					@click="isExpanded = !isExpanded"
				/>
			</div>

			<!-- Subject -->
			<div class="text-xs">
				<span class="text-gray-500 dark:text-gray-400">Subject:</span>
				<span class="ml-2 font-mono text-gray-700 dark:text-gray-200">{{ message.subject }}</span>
			</div>

			<!-- Payload (Expandable) -->
			<div v-if="isExpanded" class="space-y-2">
				<div class="flex items-center justify-between">
					<span class="text-xs text-gray-500 dark:text-gray-400">Payload:</span>
					<UButton
						variant="ghost"
						size="xs"
						icon="i-heroicons-clipboard-document"
						@click="copyPayload"
					>
						Copy
					</UButton>
				</div>

				<!-- JSON Payload Display -->
				<div
					class="p-3 bg-gray-50 dark:bg-gray-900 rounded-md border border-gray-200 dark:border-gray-700 overflow-x-auto max-h-96"
				>
					<pre class="text-xs font-mono text-gray-800 dark:text-gray-100">{{ formattedPayload }}</pre>
				</div>
			</div>

			<!-- Collapsed Payload Preview -->
			<div v-else class="text-xs">
				<span class="text-gray-500 dark:text-gray-400">Payload:</span>
				<span class="ml-2 font-mono text-gray-600 dark:text-gray-300 truncate block">
					{{ payloadPreview }}
				</span>
			</div>
		</div>
	</UCard>
</template>

<script lang="ts" setup>
	import type { NatsMessage } from "@/types/monitoring";

	const props = defineProps<{
		message: NatsMessage
	}>();

	const toast = useToast();
	const isExpanded = ref(false);

	// Format timestamp to HH:MM:SS
	const formattedTime = computed(() => {
		try {
			const date = new Date(props.message.timestamp);
			return date.toLocaleTimeString("en-US", {
				hour12: false,
				hour: "2-digit",
				minute: "2-digit",
				second: "2-digit"
			});
		} catch {
			return props.message.timestamp;
		}
	});

	// Format payload (pretty-print JSON or show raw string)
	const formattedPayload = computed(() => {
		try {
			const parsed = JSON.parse(props.message.payload);
			return JSON.stringify(parsed, null, 2);
		} catch {
			return props.message.payload;
		}
	});

	// Create payload preview (truncated for collapsed state)
	const payloadPreview = computed(() => {
		const maxLength = 100;
		const payload = props.message.payload;
		if (payload.length > maxLength) {
			return `${payload.substring(0, maxLength)}...`;
		}
		return payload;
	});

	// Color coding for message types
	const typeColor = computed(() => {
		switch (props.message.message_type.toLowerCase()) {
		case "request":
			return "blue";
		case "response":
			return "green";
		case "event":
			return "purple";
		default:
			return "gray";
		}
	});

	// Color coding for endpoints
	const endpointColor = computed(() => {
		switch (props.message.endpoint) {
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
	});

	// Background color for message cards
	const messageTypeColor = computed(() => {
		switch (props.message.message_type.toLowerCase()) {
		case "request":
			return "bg-blue-50 dark:bg-blue-950/20";
		case "response":
			return "bg-green-50 dark:bg-green-950/20";
		case "event":
			return "bg-purple-50 dark:bg-purple-950/20";
		default:
			return "bg-gray-50 dark:bg-gray-900";
		}
	});

	// Copy payload to clipboard
	function copyPayload() {
		if (navigator.clipboard && window.isSecureContext) {
			navigator.clipboard.writeText(formattedPayload.value)
				.then(() => {
					toast.add({
						title: "Copied",
						description: "Payload copied to clipboard",
						color: "green"
					});
				})
				.catch((err) => {
					console.error("Failed to copy:", err);
					toast.add({
						title: "Copy Failed",
						description: "Failed to copy to clipboard",
						color: "red"
					});
				});
		} else {
			toast.add({
				title: "Copy Failed",
				description: "Clipboard not available",
				color: "red"
			});
		}
	}
</script>
