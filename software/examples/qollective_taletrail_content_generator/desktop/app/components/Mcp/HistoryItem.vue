<template>
	<div
		class="px-3 py-2 hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer transition-colors border-b border-gray-100 dark:border-gray-800"
		@click="$emit('replay', entry)"
	>
		<div class="flex items-center justify-between gap-2">
			<!-- Left: Status icon + Tool name -->
			<div class="flex items-center gap-2 flex-1 min-w-0">
				<span :class="entry.success ? 'text-green-500' : 'text-red-500'">
					{{ entry.success ? '✓' : '✗' }}
				</span>
				<span class="font-medium text-sm truncate">
					{{ entry.tool_name }}
				</span>
			</div>

			<!-- Right: Timestamp + Replay button -->
			<div class="flex items-center gap-2">
				<span class="text-xs text-gray-500 whitespace-nowrap">
					{{ formatTimestamp(entry.timestamp) }}
				</span>
				<UButton
					icon="i-heroicons-arrow-path"
					variant="ghost"
					size="xs"
					title="Replay this request"
					@click.stop="$emit('replay', entry)"
				/>
			</div>
		</div>
	</div>
</template>

<script lang="ts" setup>
	import type { HistoryEntry } from "@/types/mcp";

	defineProps<{
		entry: HistoryEntry
	}>();

	defineEmits<{
		replay: [entry: HistoryEntry]
	}>();

	function formatTimestamp(timestamp: string): string {
		try {
			const date = new Date(timestamp);
			const now = new Date();
			const diffMs = now.getTime() - date.getTime();
			const diffMins = Math.floor(diffMs / 60000);

			if (diffMins < 1) return "Just now";
			if (diffMins < 60) return `${diffMins}m ago`;
			if (diffMins < 1440) return `${Math.floor(diffMins / 60)}h ago`;

			// Format as date
			return `${date.toLocaleDateString()} ${date.toLocaleTimeString()}`;
		} catch {
			return timestamp;
		}
	}
</script>
