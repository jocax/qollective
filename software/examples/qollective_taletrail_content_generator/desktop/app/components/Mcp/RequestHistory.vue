<template>
	<UCard class="flex flex-col h-full">
		<template #header>
			<div class="flex items-center justify-between">
				<h3 class="font-semibold">
					Request History
				</h3>
			</div>
		</template>

		<div class="flex flex-col flex-1 overflow-hidden">
			<!-- Search input only -->
			<div class="p-3 border-b">
				<UInput
					v-model="searchQuery"
					icon="i-heroicons-magnifying-glass"
					placeholder="Search by tool name..."
					size="sm"
				/>
			</div>

			<!-- Loading State -->
			<div v-if="loading" class="flex items-center justify-center py-8">
				<div class="flex flex-col items-center gap-2">
					<UIcon name="i-heroicons-arrow-path" class="w-6 h-6 animate-spin text-gray-400" />
					<p class="text-sm text-gray-500">
						Loading history...
					</p>
				</div>
			</div>

			<!-- Error State -->
			<div v-else-if="error" class="p-4 text-center">
				<UIcon name="i-heroicons-exclamation-triangle" class="w-8 h-8 text-error-500 mx-auto mb-2" />
				<p class="text-sm text-error-600 dark:text-error-400">
					{{ error }}
				</p>
				<UButton variant="ghost" size="sm" class="mt-2" @click="loadHistory">
					Retry
				</UButton>
			</div>

			<!-- Empty State -->
			<div v-else-if="!hasEntries" class="flex items-center justify-center py-8">
				<div class="text-center text-gray-400">
					<UIcon name="i-heroicons-clock" class="w-12 h-12 mx-auto mb-2 opacity-50" />
					<p class="text-sm">
						No history yet
					</p>
					<p class="text-xs mt-1">
						Send a request to start building history
					</p>
				</div>
			</div>

			<!-- History List -->
			<div v-else class="flex-1 overflow-y-auto">
				<div class="divide-y divide-gray-200 dark:divide-gray-700">
					<McpHistoryItem
						v-for="entry in filteredEntries"
						:key="entry.id"
						:entry="entry"
						@replay="handleReplay"
						@delete="handleDeleteEntry"
					/>
				</div>

				<!-- Pagination -->
				<div v-if="historyPage && historyPage.has_more" class="p-3 border-t">
					<UButton
						variant="ghost"
						size="sm"
						:loading="loadingMore"
						block
						@click="loadMore"
					>
						Load More
					</UButton>
				</div>
			</div>

			<!-- Footer with count -->
			<div v-if="hasEntries" class="p-3 border-t text-xs text-gray-500 text-center">
				{{ filteredEntries.length }} of {{ historyPage?.total_count || 0 }} entries
			</div>
		</div>
	</UCard>
</template>

<script lang="ts" setup>
	import type { HistoryEntry, HistoryPage, HistoryQuery, ServerName } from "@/types/mcp";
	import { invoke } from "@tauri-apps/api/core";
	import { computed, onMounted, ref, watch } from "vue";

	const props = defineProps<{
		server: ServerName
	}>();

	const emit = defineEmits<{
		replay: [entry: HistoryEntry]
	}>();

	// ============================================================================
	// State
	// ============================================================================

	const historyPage = ref<HistoryPage | null>(null);
	const loading = ref(false);
	const loadingMore = ref(false);
	const error = ref<string | null>(null);
	const searchQuery = ref("");

	// ============================================================================
	// Computed Properties
	// ============================================================================

	const hasEntries = computed(() => {
		return historyPage.value && historyPage.value.entries.length > 0;
	});

	const filteredEntries = computed(() => {
		if (!historyPage.value) return [];

		let entries = historyPage.value.entries;

		// Filter by search query (tool name)
		if (searchQuery.value) {
			const query = searchQuery.value.toLowerCase();
			entries = entries.filter((e) =>
				e.tool_name.toLowerCase().includes(query)
			);
		}

		return entries;
	});

	// ============================================================================
	// Lifecycle
	// ============================================================================

	onMounted(() => {
		loadHistory();
	});

	// ============================================================================
	// Watchers
	// ============================================================================

	// Reload when server prop changes
	watch(() => props.server, () => {
		loadHistory();
	});

	// ============================================================================
	// Functions
	// ============================================================================

	async function loadHistory() {
		loading.value = true;
		error.value = null;

		try {
			const query: HistoryQuery = {
				page: 0,
				page_size: 50,
				server_filter: props.server
			};

			const page = await invoke<HistoryPage>("load_request_history", {
				query
			});

			// Compute has_more for UI
			page.has_more = page.page < page.total_pages - 1;
			historyPage.value = page;
		} catch (e: any) {
			error.value = e.toString();
			console.error("Failed to load history:", e);
		} finally {
			loading.value = false;
		}
	}

	async function loadMore() {
		if (!historyPage.value || !historyPage.value.has_more) return;

		loadingMore.value = true;

		try {
			const query: HistoryQuery = {
				page: historyPage.value.page + 1,
				page_size: 50,
				server_filter: props.server
			};

			const nextPage = await invoke<HistoryPage>("load_request_history", {
				query
			});

			// Compute has_more for UI
			nextPage.has_more = nextPage.page < nextPage.total_pages - 1;

			// Append entries
			historyPage.value = {
				...nextPage,
				entries: [...historyPage.value.entries, ...nextPage.entries]
			};
		} catch (e: any) {
			error.value = e.toString();
		} finally {
			loadingMore.value = false;
		}
	}

	function handleReplay(entry: HistoryEntry) {
		emit("replay", entry);
	}

	async function handleDeleteEntry(entry: HistoryEntry) {
		try {
			await invoke("delete_history_entry", { entryId: entry.id });

			// Remove from local list
			if (historyPage.value) {
				historyPage.value.entries = historyPage.value.entries.filter(
					(e) => e.id !== entry.id
				);
				historyPage.value.total -= 1;
			}
		} catch (e: any) {
			error.value = `Failed to delete entry: ${e}`;
		}
	}

	async function handleClearHistory() {
		if (
			!confirm(
				"Are you sure you want to clear all history? This cannot be undone."
			)
		) {
			return;
		}

		try {
			await invoke("clear_request_history");
			historyPage.value = null;
			loadHistory(); // Reload to show empty state
		} catch (e: any) {
			error.value = `Failed to clear history: ${e}`;
		}
	}
</script>
