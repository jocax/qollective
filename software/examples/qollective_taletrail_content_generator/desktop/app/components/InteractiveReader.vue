<template>
	<div class="interactive-reader space-y-6">
		<!-- Controls Bar -->
		<div class="controls-bar flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-900 rounded-lg">
			<div class="flex items-center gap-3">
				<UButton
					:disabled="!canGoBack"
					variant="solid"
					color="gray"
					icon="i-heroicons-arrow-left"
					@click="goBack"
				>
					Back
				</UButton>
				<UButton
					variant="ghost"
					color="gray"
					icon="i-heroicons-arrow-path"
					@click="restart"
				>
					Restart
				</UButton>
			</div>

			<div class="flex items-center gap-4">
				<div class="flex items-center gap-2">
					<label class="text-sm font-medium">Educational Insights</label>
					<UToggle v-model="showInsights" />
				</div>
				<div class="flex items-center gap-2">
					<label class="text-sm font-medium">Show Metadata</label>
					<UToggle v-model="showMetadata" />
				</div>
			</div>
		</div>

		<!-- Progress Indicator -->
		<div class="progress-section">
			<div class="flex items-center justify-between mb-2">
				<span class="text-sm font-medium text-gray-700 dark:text-gray-300">
					Reading Progress
				</span>
				<span class="text-sm text-gray-600 dark:text-gray-400">
					{{ exploredNodes.size }} / {{ totalNodes }} nodes explored
					<span class="font-semibold text-primary-600 dark:text-primary-400">
						({{ progressPercent }}%)
					</span>
				</span>
			</div>
			<UProgress :value="progressPercent" color="primary" size="md" />
		</div>

		<!-- Error State -->
		<UAlert
			v-if="error"
			color="red"
			variant="subtle"
			icon="i-heroicons-exclamation-triangle"
			title="Navigation Error"
			:description="error"
		/>

		<!-- Story Content -->
		<div v-if="currentNode" class="story-content">
			<StoryNode
				:node="currentNode"
				:show-metadata="showMetadata"
				:show-insights="showInsights"
				:is-convergence-point="isCurrentNodeConvergence"
			/>
		</div>

		<!-- Choices -->
		<div v-if="currentNode" class="choices-section mt-6">
			<h3 v-if="currentNode.content.choices && currentNode.content.choices.length > 0" class="text-lg font-semibold mb-3 text-gray-800 dark:text-gray-200">
				What happens next?
			</h3>
			<ChoiceList
				:choices="currentNode.content.choices || []"
				:explored-choices="getExploredChoiceIds()"
				:convergence-points="convergencePoints"
				@select="selectChoice"
			/>
		</div>

		<!-- Navigation Stats -->
		<div class="stats-section p-4 bg-gray-50 dark:bg-gray-900 rounded-lg">
			<div class="grid grid-cols-3 gap-4 text-center">
				<div>
					<div class="text-2xl font-bold text-primary-600 dark:text-primary-400">
						{{ navigationHistory.length }}
					</div>
					<div class="text-sm text-gray-600 dark:text-gray-400">
						Steps Taken
					</div>
				</div>
				<div>
					<div class="text-2xl font-bold text-purple-600 dark:text-purple-400">
						{{ exploredConvergencePoints.size }}
					</div>
					<div class="text-sm text-gray-600 dark:text-gray-400">
						Convergence Points
					</div>
				</div>
				<div>
					<div class="text-2xl font-bold text-green-600 dark:text-green-400">
						{{ uniqueChoicesMade.size }}
					</div>
					<div class="text-sm text-gray-600 dark:text-gray-400">
						Unique Choices
					</div>
				</div>
			</div>
		</div>
	</div>
</template>

<script lang="ts" setup>
	import type { Choice, ContentNode, Trail } from "~/types/trails";

	interface Props {
		trail: Trail
	}

	const props = defineProps<Props>();

	// State
	const currentNodeId = ref<string>("");
	const navigationHistory = ref<string[]>([]);
	const exploredNodes = ref<Set<string>>(new Set());
	const exploredConvergencePoints = ref<Set<string>>(new Set());
	const uniqueChoicesMade = ref<Set<string>>(new Set());
	const showMetadata = ref(false);
	const showInsights = ref(true);
	const error = ref<string | null>(null);

	// Initialize to start node
	onMounted(() => {
		restart();
	});

	// Computed properties
	const currentNode = computed<ContentNode | null>(() => {
		if (!currentNodeId.value) return null;
		return props.trail.dag.nodes[currentNodeId.value] || null;
	});

	const canGoBack = computed(() => navigationHistory.value.length > 1);

	const totalNodes = computed(() => Object.keys(props.trail.dag.nodes).length);

	const progressPercent = computed(() => {
		if (totalNodes.value === 0) return 0;
		return Math.round((exploredNodes.value.size / totalNodes.value) * 100);
	});

	const convergencePoints = computed(() => {
		return props.trail.dag.convergence_points || [];
	});

	const isCurrentNodeConvergence = computed(() => {
		return convergencePoints.value.includes(currentNodeId.value);
	});

	// Methods
	function selectChoice(choice: Choice) {
		if (!choice.next_node_id) {
			error.value = "Invalid choice: no next node specified";
			return;
		}

		// Validate next node exists
		if (!props.trail.dag.nodes[choice.next_node_id]) {
			error.value = `Node ${choice.next_node_id} not found in trail`;
			return;
		}

		// Clear any previous errors
		error.value = null;

		// Track the choice
		uniqueChoicesMade.value.add(choice.id);

		// Save current node to history
		navigationHistory.value.push(currentNodeId.value);

		// Mark current node as explored
		exploredNodes.value.add(currentNodeId.value);

		// Track convergence points
		if (isCurrentNodeConvergence.value) {
			exploredConvergencePoints.value.add(currentNodeId.value);
		}

		// Navigate to next node
		currentNodeId.value = choice.next_node_id;
	}

	function goBack() {
		if (!canGoBack.value) return;

		const previousNodeId = navigationHistory.value.pop();
		if (previousNodeId) {
			currentNodeId.value = previousNodeId;
			error.value = null;
		}
	}

	function restart() {
		const startNodeId = props.trail.metadata.start_node_id;

		// Validate start node exists
		if (!props.trail.dag.nodes[startNodeId]) {
			error.value = "Invalid trail: start node not found";
			return;
		}

		currentNodeId.value = startNodeId;
		navigationHistory.value = [];
		exploredNodes.value = new Set();
		exploredConvergencePoints.value = new Set();
		uniqueChoicesMade.value = new Set();
		error.value = null;
	}

	/**
	 * Get IDs of choices that have been explored from current node
	 */
	function getExploredChoiceIds(): string[] {
		if (!currentNode.value?.content.choices) return [];

		return currentNode.value.content.choices
			.filter((choice) => exploredNodes.value.has(choice.next_node_id))
			.map((choice) => choice.id);
	}

	// Watch for trail changes and reinitialize
	watch(() => props.trail, () => {
		restart();
	}, { immediate: false });
</script>

<style scoped>
.interactive-reader {
  max-width: 900px;
  margin: 0 auto;
}

.controls-bar {
  position: sticky;
  top: 0;
  z-index: 10;
  backdrop-filter: blur(10px);
}
</style>
