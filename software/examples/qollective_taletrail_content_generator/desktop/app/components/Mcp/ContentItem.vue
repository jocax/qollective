<template>
	<UCard>
		<template #header>
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-2">
					<UIcon :name="contentIcon" class="w-4 h-4" />
					<span class="text-xs font-semibold">{{ contentLabel }}</span>
				</div>
				<UBadge variant="subtle" size="xs">
					{{ item.type }}
				</UBadge>
			</div>
		</template>

		<!-- Text Content -->
		<div v-if="item.type === 'text' && item.text" class="space-y-2">
			<!-- Plain text display with line breaks preserved -->
			<div class="text-sm whitespace-pre-wrap break-words">
				{{ item.text }}
			</div>

			<!-- Copy button for text -->
			<UButton
				variant="ghost"
				size="xs"
				icon="i-heroicons-clipboard-document"
				@click="copyText"
			>
				Copy Text
			</UButton>
		</div>

		<!-- Image Content -->
		<div v-else-if="item.type === 'image' && item.data" class="space-y-2">
			<img
				:src="`data:${item.mimeType || 'image/png'};base64,${item.data}`"
				:alt="`Image ${index + 1}`"
				class="max-w-full h-auto rounded"
			>
			<div class="text-xs text-gray-500">
				MIME Type: {{ item.mimeType || 'image/png' }}
			</div>
		</div>

		<!-- Resource Content -->
		<div v-else-if="item.type === 'resource' && item.uri" class="space-y-2">
			<div class="flex items-center gap-2">
				<UIcon name="i-heroicons-link" class="w-4 h-4 text-gray-500" />
				<a
					:href="item.uri"
					target="_blank"
					rel="noopener noreferrer"
					class="text-sm text-primary-500 hover:text-primary-600 underline break-all"
				>
					{{ item.uri }}
				</a>
			</div>
			<div v-if="item.mimeType" class="text-xs text-gray-500">
				MIME Type: {{ item.mimeType }}
			</div>
		</div>

		<!-- Fallback for unknown content -->
		<div v-else class="text-xs text-gray-500">
			<pre class="font-mono">{{ JSON.stringify(item, null, 2) }}</pre>
		</div>
	</UCard>
</template>

<script lang="ts" setup>
	import type { ContentItem } from "@/types/mcp";
	import { computed } from "vue";

	const props = defineProps<{
		item: ContentItem
		index: number
	}>();

	const contentIcon = computed(() => {
		switch (props.item.type) {
		case "text":
			return "i-heroicons-document-text";
		case "image":
			return "i-heroicons-photo";
		case "resource":
			return "i-heroicons-link";
		default:
			return "i-heroicons-question-mark-circle";
		}
	});

	const contentLabel = computed(() => {
		switch (props.item.type) {
		case "text":
			return `Text Content #${props.index + 1}`;
		case "image":
			return `Image #${props.index + 1}`;
		case "resource":
			return `Resource #${props.index + 1}`;
		default:
			return `Content #${props.index + 1}`;
		}
	});

	function copyText() {
		if (props.item.type === "text" && props.item.text) {
			navigator.clipboard.writeText(props.item.text);
			console.log("Text copied to clipboard");
		}
	}
</script>
