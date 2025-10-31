<template>
	<div class="space-y-2">
		<div class="text-xs font-semibold text-gray-700 dark:text-gray-300 mb-2">
			Envelope Metadata
		</div>

		<!-- In verbose mode, we'd show full envelope. Since we only have CallToolResult, show what's available -->
		<div class="grid grid-cols-2 gap-2 text-xs">
			<div>
				<span class="text-gray-500">Is Error:</span>
				<span class="ml-2 font-mono">{{ response.isError ? 'true' : 'false' }}</span>
			</div>
			<div>
				<span class="text-gray-500">Content Items:</span>
				<span class="ml-2 font-mono">{{ response.content.length }}</span>
			</div>
		</div>

		<!-- Content Types Summary -->
		<div class="mt-3">
			<div class="text-xs text-gray-500 mb-1">
				Content Types:
			</div>
			<div class="flex gap-2">
				<UBadge
					v-for="(count, type) in contentTypeSummary"
					:key="type"
					variant="subtle"
					size="xs"
				>
					{{ type }}: {{ count }}
				</UBadge>
			</div>
		</div>
	</div>
</template>

<script lang="ts" setup>
	import type { CallToolResult } from "@/types/mcp";
	import { computed } from "vue";

	const props = defineProps<{
		response: CallToolResult
	}>();

	const contentTypeSummary = computed(() => {
		const summary: Record<string, number> = {};
		for (const item of props.response.content) {
			summary[item.type] = (summary[item.type] || 0) + 1;
		}
		return summary;
	});
</script>
