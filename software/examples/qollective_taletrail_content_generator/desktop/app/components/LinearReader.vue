<template>
	<div class="linear-reader">
		<!-- Page Header -->
		<div class="page-header flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-900 rounded-lg mb-6">
			<div class="flex items-center gap-4">
				<p class="text-sm font-medium text-gray-700 dark:text-gray-300">
					Page {{ currentPage + 1 }} of {{ totalPages }}
				</p>

				<!-- Jump to page selector -->
				<USelectMenu
					v-model="currentPage"
					:options="pageOptions"
					placeholder="Jump to page..."
					size="sm"
					class="w-40"
				/>
			</div>

			<div class="flex items-center gap-2">
				<UButton
					variant="ghost"
					color="gray"
					icon="i-heroicons-home"
					size="sm"
					title="Go to first page (Home)"
					@click="jumpToPage(0)"
				>
					First
				</UButton>
				<UButton
					variant="ghost"
					color="gray"
					icon="i-heroicons-arrow-path"
					size="sm"
					title="Go to last page (End)"
					@click="jumpToPage(totalPages - 1)"
				>
					Last
				</UButton>
			</div>
		</div>

		<!-- Node Content -->
		<div v-if="currentNode" class="node-content mb-6">
			<StoryNode
				:node="currentNode"
				:show-metadata="showMetadata"
				:is-convergence-point="isCurrentNodeConvergence"
			/>
		</div>

		<!-- Empty State -->
		<div v-else class="empty-state p-12 text-center">
			<UCard>
				<div class="p-8">
					<svg class="w-16 h-16 mx-auto mb-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
					</svg>
					<h3 class="text-xl font-semibold mb-2">
						No Content
					</h3>
					<p class="text-gray-600 dark:text-gray-400">
						Unable to load story content for this page.
					</p>
				</div>
			</UCard>
		</div>

		<!-- Navigation Controls -->
		<div class="navigation-controls flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-900 rounded-lg">
			<UButton
				:disabled="!canGoPrev"
				variant="solid"
				color="gray"
				icon="i-heroicons-arrow-left"
				size="lg"
				@click="prevPage"
			>
				Previous
			</UButton>

			<!-- Page indicator (center) -->
			<div class="text-sm font-medium text-gray-600 dark:text-gray-400">
				{{ currentPage + 1 }} / {{ totalPages }}
			</div>

			<UButton
				:disabled="!canGoNext"
				variant="solid"
				color="primary"
				trailing
				icon="i-heroicons-arrow-right"
				size="lg"
				@click="nextPage"
			>
				Next
			</UButton>
		</div>

		<!-- Reading Progress -->
		<div class="progress-section mt-6">
			<div class="flex items-center justify-between mb-2">
				<span class="text-sm font-medium text-gray-700 dark:text-gray-300">
					Reading Progress
				</span>
				<span class="text-sm text-gray-600 dark:text-gray-400">
					<span class="font-semibold text-primary-600 dark:text-primary-400">
						{{ progressPercent }}%
					</span>
				</span>
			</div>
			<!-- Static progress bar (no animation) -->
			<div class="h-2 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
				<div
					class="h-full bg-primary-500 dark:bg-primary-400"
					:style="{ width: `${progressPercent}%` }"
				/>
			</div>
		</div>

		<!-- Settings -->
		<div class="settings-section mt-4 p-4 bg-gray-50 dark:bg-gray-900 rounded-lg">
			<div class="flex items-center justify-between">
				<label class="text-sm font-medium">Show Node Metadata</label>
				<UToggle v-model="showMetadata" />
			</div>
		</div>
	</div>
</template>

<script lang="ts" setup>
	import type { ContentNode, Trail } from "~/types/trails";

	interface Props {
		trail: Trail
	}

	const props = defineProps<Props>();

	// State
	const currentPage = ref(0);
	const showMetadata = ref(false);

	/**
	 * Traverse DAG to create linear sequence of nodes
	 * Uses depth-first traversal starting from start_node_id
	 */
	function traverseDAG(startNodeId: string): ContentNode[] {
		const visited = new Set<string>();
		const result: ContentNode[] = [];

		function visit(nodeId: string) {
			// Skip if already visited (handles convergence points)
			if (visited.has(nodeId)) return;

			const node = props.trail.dag.nodes[nodeId];
			if (!node) {
				console.warn(`Node ${nodeId} not found in DAG`);
				return;
			}

			visited.add(nodeId);
			result.push(node);

			// Follow edges to child nodes in order
			const edges = props.trail.dag.edges.filter((e) => e.from_node_id === nodeId);

			// Sort edges by choice_id for consistent ordering
			edges.sort((a, b) => a.choice_id.localeCompare(b.choice_id));

			edges.forEach((edge) => visit(edge.to_node_id));
		}

		visit(startNodeId);
		return result;
	}

	// Computed properties
	const linearNodes = computed(() => {
		const startNodeId = props.trail.metadata.start_node_id;
		return traverseDAG(startNodeId);
	});

	const totalPages = computed(() => linearNodes.value.length);

	const currentNode = computed<ContentNode | null>(() => {
		if (currentPage.value >= linearNodes.value.length) return null;
		return linearNodes.value[currentPage.value] || null;
	});

	const canGoNext = computed(() => currentPage.value < totalPages.value - 1);

	const canGoPrev = computed(() => currentPage.value > 0);

	const progressPercent = computed(() => {
		if (totalPages.value === 0) return 0;
		return Math.round(((currentPage.value + 1) / totalPages.value) * 100);
	});

	const convergencePoints = computed(() => {
		return props.trail.dag.convergence_points || [];
	});

	const isCurrentNodeConvergence = computed(() => {
		if (!currentNode.value) return false;
		return convergencePoints.value.includes(currentNode.value.id);
	});

	/**
	 * Generate page options for jump selector
	 */
	const pageOptions = computed(() => {
		return Array.from({ length: totalPages.value }, (_, i) => ({
			label: `Page ${i + 1}${linearNodes.value[i] ? ` (Node ${linearNodes.value[i].id})` : ""}`,
			value: i
		}));
	});

	// Navigation methods
	function nextPage() {
		if (canGoNext.value) {
			currentPage.value++;
			scrollToTop();
		}
	}

	function prevPage() {
		if (canGoPrev.value) {
			currentPage.value--;
			scrollToTop();
		}
	}

	function jumpToPage(page: number) {
		const targetPage = Math.max(0, Math.min(page, totalPages.value - 1));
		if (targetPage !== currentPage.value) {
			currentPage.value = targetPage;
			scrollToTop();
		}
	}

	function scrollToTop() {
		// Smooth scroll to top of content
		window.scrollTo({ top: 0, behavior: "smooth" });
	}

	// Keyboard navigation
	function handleKeydown(event: KeyboardEvent) {
		switch (event.key) {
		case "ArrowLeft":
			if (!event.ctrlKey && !event.metaKey) {
				prevPage();
				event.preventDefault();
			}
			break;
		case "ArrowRight":
			if (!event.ctrlKey && !event.metaKey) {
				nextPage();
				event.preventDefault();
			}
			break;
		case "Home":
			jumpToPage(0);
			event.preventDefault();
			break;
		case "End":
			jumpToPage(totalPages.value - 1);
			event.preventDefault();
			break;
		}
	}

	// Set up keyboard listener
	onMounted(() => {
		window.addEventListener("keydown", handleKeydown);
	});

	onUnmounted(() => {
		window.removeEventListener("keydown", handleKeydown);
	});

	// Reset to first page when trail changes
	watch(() => props.trail, () => {
		currentPage.value = 0;
	}, { immediate: false });
</script>

<style scoped>
.linear-reader {
  max-width: 900px;
  margin: 0 auto;
}

.navigation-controls {
  position: sticky;
  bottom: 20px;
  backdrop-filter: blur(10px);
  box-shadow: 0 -2px 10px rgba(0, 0, 0, 0.1);
}
</style>
