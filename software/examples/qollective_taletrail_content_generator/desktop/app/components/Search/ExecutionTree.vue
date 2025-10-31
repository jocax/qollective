<template>
	<div class="flex-1 overflow-y-auto pr-2">
		<!-- Empty State -->
		<div v-if="directories.length === 0" class="flex items-center justify-center h-full">
			<div class="text-center p-8">
				<UIcon name="i-heroicons-folder-open" class="w-12 h-12 text-gray-400 mx-auto mb-3" />
				<h4 class="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-1">
					No Execution Directories
				</h4>
				<p class="text-xs text-gray-600 dark:text-gray-400">
					No execution history found in the root directory
				</p>
			</div>
		</div>

		<!-- Execution Directories Tree -->
		<div v-else class="space-y-2">
			<div
				v-for="dir in directories"
				:key="dir.requestId"
				class="border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden"
			>
				<!-- Request ID Header (Collapsible) -->
				<div
					class="flex items-center gap-2 p-3 bg-gray-50 dark:bg-gray-800 hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer transition-colors"
					@click="toggleDirectory(dir.requestId)"
				>
					<UIcon
						:name="isExpanded(dir.requestId) ? 'i-heroicons-chevron-down' : 'i-heroicons-chevron-right'"
						class="text-gray-500 dark:text-gray-400"
					/>
					<UIcon name="i-heroicons-folder" class="text-blue-500" />
					<div class="flex-1 min-w-0">
						<div class="font-mono text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
							{{ dir.requestId }}
						</div>
						<div class="text-xs text-gray-500 dark:text-gray-400">
							{{ formatTimestamp(dir.timestamp) }}
						</div>
					</div>
					<UBadge color="gray" variant="subtle" size="xs">
						{{ dir.servers.length }} {{ dir.servers.length === 1 ? 'server' : 'servers' }}
					</UBadge>
				</div>

				<!-- Server Files List (Expandable) -->
				<div
					v-if="isExpanded(dir.requestId)"
					class="bg-white dark:bg-gray-900 border-t border-gray-200 dark:border-gray-700"
				>
					<div
						v-for="server in dir.servers"
						:key="server"
						class="flex items-center gap-2 p-2.5 pl-10 hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer transition-colors"
						:class="{
							'bg-blue-50 dark:bg-blue-900/20 border-l-2 border-blue-500':
								selectedRequestId === dir.requestId && selectedServer === server
						}"
						@click.stop="handleServerClick(dir.requestId, server)"
					>
						<UIcon name="i-heroicons-document-text" class="text-gray-400" />
						<span class="text-sm text-gray-700 dark:text-gray-300">
							{{ server }}
						</span>
						<!-- Selected Indicator -->
						<UIcon
							v-if="selectedRequestId === dir.requestId && selectedServer === server"
							name="i-heroicons-arrow-left"
							class="ml-auto text-blue-500"
						/>
					</div>
				</div>
			</div>
		</div>
	</div>
</template>

<script lang="ts" setup>
	import type { ExecutionDirectory } from "~/types/search";

	const props = defineProps<{
		directories: ExecutionDirectory[]
		selectedRequestId: string | null
		selectedServer: string | null
	}>();

	const emit = defineEmits<{
		select: [requestId: string, server: string]
	}>();

	// State for expanded directories
	const expandedDirs = ref<Set<string>>(new Set());

	/**
	 * Toggle directory expansion
	 */
	function toggleDirectory(requestId: string) {
		if (expandedDirs.value.has(requestId)) {
			expandedDirs.value.delete(requestId);
		} else {
			expandedDirs.value.add(requestId);
		}
	}

	/**
	 * Check if directory is expanded
	 */
	function isExpanded(requestId: string): boolean {
		return expandedDirs.value.has(requestId);
	}

	/**
	 * Handle server file click
	 */
	function handleServerClick(requestId: string, server: string) {
		emit("select", requestId, server);
	}

	/**
	 * Format timestamp for display
	 */
	function formatTimestamp(timestamp: string): string {
		try {
			const date = new Date(timestamp);
			return date.toLocaleString("en-US", {
				month: "short",
				day: "numeric",
				year: "numeric",
				hour: "2-digit",
				minute: "2-digit"
			});
		} catch {
			return timestamp;
		}
	}

	// Auto-expand first directory on mount
	onMounted(() => {
		if (props.directories.length > 0) {
			expandedDirs.value.add(props.directories[0].requestId);
		}
	});
</script>

<style scoped>
/* Smooth transitions for expand/collapse */
.transition-colors {
	transition: background-color 0.15s ease-in-out;
}
</style>
