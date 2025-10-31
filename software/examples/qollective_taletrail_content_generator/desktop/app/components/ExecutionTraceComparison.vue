<template>
	<div class="space-y-4">
		<!-- Overall Statistics -->
		<UCard>
			<template #header>
				<h3 class="text-lg font-semibold">
					Execution Performance Comparison
				</h3>
			</template>

			<div class="grid grid-cols-2 md:grid-cols-4 gap-4">
				<!-- Total Duration -->
				<div class="space-y-1">
					<p class="text-xs text-gray-500 dark:text-gray-400">
						Total Duration
					</p>
					<div class="space-y-1">
						<p class="text-sm">
							<span class="text-gray-600 dark:text-gray-300">Original:</span>
							<span class="font-mono ml-2">{{ stats.originalTotal.toFixed(0) }}ms</span>
						</p>
						<p class="text-sm">
							<span class="text-gray-600 dark:text-gray-300">New:</span>
							<span class="font-mono ml-2">{{ stats.newTotal.toFixed(0) }}ms</span>
						</p>
						<UBadge
							:color="getDurationColor(stats.totalDiff)"
							variant="subtle"
							size="sm"
						>
							{{ formatDurationDiff(stats.totalDiff) }}
						</UBadge>
					</div>
				</div>

				<!-- Success Rate -->
				<div class="space-y-1">
					<p class="text-xs text-gray-500 dark:text-gray-400">
						Success Rate
					</p>
					<div class="space-y-1">
						<p class="text-sm">
							<span class="text-green-600 dark:text-green-400">Original:</span>
							<span class="font-mono ml-2">{{ stats.originalSuccess }}</span>
						</p>
						<p class="text-sm">
							<span class="text-green-600 dark:text-green-400">New:</span>
							<span class="font-mono ml-2">{{ stats.newSuccess }}</span>
						</p>
						<UBadge
							v-if="stats.successDiff !== 0"
							:color="stats.successDiff > 0 ? 'green' : 'red'"
							variant="subtle"
							size="sm"
						>
							{{ stats.successDiff > 0 ? '+' : '' }}{{ stats.successDiff }}
						</UBadge>
					</div>
				</div>

				<!-- Failure Rate -->
				<div class="space-y-1">
					<p class="text-xs text-gray-500 dark:text-gray-400">
						Failures
					</p>
					<div class="space-y-1">
						<p class="text-sm">
							<span class="text-red-600 dark:text-red-400">Original:</span>
							<span class="font-mono ml-2">{{ stats.originalFailed }}</span>
						</p>
						<p class="text-sm">
							<span class="text-red-600 dark:text-red-400">New:</span>
							<span class="font-mono ml-2">{{ stats.newFailed }}</span>
						</p>
						<UBadge
							v-if="stats.failedDiff !== 0"
							:color="stats.failedDiff < 0 ? 'green' : 'red'"
							variant="subtle"
							size="sm"
						>
							{{ stats.failedDiff > 0 ? '+' : '' }}{{ stats.failedDiff }}
						</UBadge>
					</div>
				</div>

				<!-- Performance Summary -->
				<div class="space-y-1">
					<p class="text-xs text-gray-500 dark:text-gray-400">
						Performance
					</p>
					<div class="flex items-center gap-2 mt-2">
						<UIcon
							:name="stats.totalDiff < 0 ? 'i-heroicons-arrow-trending-down' : stats.totalDiff > 0 ? 'i-heroicons-arrow-trending-up' : 'i-heroicons-minus'"
							:class="{
								'text-green-500': stats.totalDiff < 0,
								'text-red-500': stats.totalDiff > 0,
								'text-gray-500': stats.totalDiff === 0
							}"
							class="text-2xl"
						/>
						<div>
							<p class="text-lg font-bold">
								{{ Math.abs((stats.totalDiff / stats.originalTotal) * 100).toFixed(1) }}%
							</p>
							<p class="text-xs text-gray-500">
								{{ stats.totalDiff < 0 ? 'Faster' : stats.totalDiff > 0 ? 'Slower' : 'Same' }}
							</p>
						</div>
					</div>
				</div>
			</div>
		</UCard>

		<!-- Phase-by-Phase Comparison -->
		<UCard>
			<template #header>
				<h3 class="text-lg font-semibold">
					Phase-by-Phase Breakdown
				</h3>
			</template>

			<div class="space-y-2">
				<div
					v-for="comparison in phaseComparisons"
					:key="comparison.phase"
					class="p-3 border rounded-lg dark:border-gray-700"
					:class="{
						'border-yellow-300 bg-yellow-50 dark:bg-yellow-950': comparison.successChanged,
						'bg-gray-50 dark:bg-gray-900': !comparison.successChanged
					}"
				>
					<div class="flex items-center justify-between mb-2">
						<div class="flex items-center gap-2">
							<code class="text-xs px-2 py-1 bg-gray-100 dark:bg-gray-800 rounded">
								{{ comparison.phase }}
							</code>
							<UBadge
								v-if="comparison.successChanged"
								color="yellow"
								variant="subtle"
							>
								Status Changed
							</UBadge>
						</div>

						<UBadge
							v-if="comparison.durationDiff !== undefined"
							:color="getDurationColor(comparison.durationDiff)"
							variant="subtle"
						>
							{{ formatDurationDiff(comparison.durationDiff) }}
						</UBadge>
					</div>

					<div class="grid grid-cols-2 gap-4 text-sm">
						<!-- Original -->
						<div>
							<p class="text-xs font-semibold text-gray-500 dark:text-gray-400 mb-1">
								Original
							</p>
							<div v-if="comparison.original" class="space-y-1">
								<div class="flex items-center gap-2">
									<UIcon
										:name="comparison.original.success ? 'i-heroicons-check-circle' : 'i-heroicons-x-circle'"
										:class="comparison.original.success ? 'text-green-500' : 'text-red-500'"
									/>
									<span class="font-mono">{{ comparison.original.duration_ms.toFixed(0) }}ms</span>
								</div>
								<p v-if="comparison.original.error_message" class="text-xs text-red-600 dark:text-red-400">
									{{ comparison.original.error_message }}
								</p>
							</div>
							<p v-else class="text-xs text-gray-400">
								Not executed
							</p>
						</div>

						<!-- New -->
						<div>
							<p class="text-xs font-semibold text-gray-500 dark:text-gray-400 mb-1">
								New
							</p>
							<div v-if="comparison.new" class="space-y-1">
								<div class="flex items-center gap-2">
									<UIcon
										:name="comparison.new.success ? 'i-heroicons-check-circle' : 'i-heroicons-x-circle'"
										:class="comparison.new.success ? 'text-green-500' : 'text-red-500'"
									/>
									<span class="font-mono">{{ comparison.new.duration_ms.toFixed(0) }}ms</span>
								</div>
								<p v-if="comparison.new.error_message" class="text-xs text-red-600 dark:text-red-400">
									{{ comparison.new.error_message }}
								</p>
							</div>
							<p v-else class="text-xs text-gray-400">
								Not executed
							</p>
						</div>
					</div>

					<!-- Duration Bar Chart -->
					<div v-if="comparison.original && comparison.new" class="mt-3">
						<div class="grid grid-cols-2 gap-2">
							<div class="h-2 bg-blue-200 dark:bg-blue-800 rounded" :style="{ width: `${(comparison.original.duration_ms / Math.max(comparison.original.duration_ms, comparison.new.duration_ms)) * 100}%` }" />
							<div class="h-2 bg-purple-200 dark:bg-purple-800 rounded" :style="{ width: `${(comparison.new.duration_ms / Math.max(comparison.original.duration_ms, comparison.new.duration_ms)) * 100}%` }" />
						</div>
					</div>
				</div>

				<div v-if="phaseComparisons.length === 0" class="text-center py-8 text-gray-500">
					No execution trace data available for comparison
				</div>
			</div>
		</UCard>
	</div>
</template>

<script setup lang="ts">
	import type { ServiceInvocation } from "~/types/trails";

	interface Props {
		originalTrace?: ServiceInvocation[]
		newTrace?: ServiceInvocation[]
	}

	const props = defineProps<Props>();

	interface PhaseComparison {
		phase: string
		original?: ServiceInvocation
		new?: ServiceInvocation
		durationDiff?: number
		successChanged?: boolean
	}

	/**
	 * Group service invocations by phase
	 */
	function groupByPhase(invocations: ServiceInvocation[]): Record<string, ServiceInvocation> {
		return invocations.reduce((acc, inv) => {
			const key = `${inv.phase}-${inv.service_name}`;
			acc[key] = inv;
			return acc;
		}, {} as Record<string, ServiceInvocation>);
	}

	/**
	 * Compare execution traces
	 */
	const phaseComparisons = computed<PhaseComparison[]>(() => {
		if (!props.originalTrace && !props.newTrace) return [];

		const originalByPhase = props.originalTrace ? groupByPhase(props.originalTrace) : {};
		const newByPhase = props.newTrace ? groupByPhase(props.newTrace) : {};

		const allPhases = new Set([...Object.keys(originalByPhase), ...Object.keys(newByPhase)]);
		const comparisons: PhaseComparison[] = [];

		for (const phaseKey of allPhases) {
			const original = originalByPhase[phaseKey];
			const newInv = newByPhase[phaseKey];

			const comparison: PhaseComparison = {
				phase: phaseKey,
				original,
				new: newInv
			};

			if (original && newInv) {
				comparison.durationDiff = newInv.duration_ms - original.duration_ms;
				comparison.successChanged = original.success !== newInv.success;
			}

			comparisons.push(comparison);
		}

		return comparisons.sort((a, b) => {
			// Sort by phase name
			return a.phase.localeCompare(b.phase);
		});
	});

	/**
	 * Overall statistics
	 */
	const stats = computed(() => {
		const originalTotal = props.originalTrace?.reduce((sum, inv) => sum + inv.duration_ms, 0) || 0;
		const newTotal = props.newTrace?.reduce((sum, inv) => sum + inv.duration_ms, 0) || 0;

		const originalSuccess = props.originalTrace?.filter((inv) => inv.success).length || 0;
		const originalFailed = props.originalTrace?.filter((inv) => !inv.success).length || 0;
		const newSuccess = props.newTrace?.filter((inv) => inv.success).length || 0;
		const newFailed = props.newTrace?.filter((inv) => !inv.success).length || 0;

		return {
			originalTotal,
			newTotal,
			totalDiff: newTotal - originalTotal,
			originalSuccess,
			originalFailed,
			newSuccess,
			newFailed,
			successDiff: newSuccess - originalSuccess,
			failedDiff: newFailed - originalFailed
		};
	});

	/**
	 * Format duration with color
	 */
	function getDurationColor(diff?: number): string {
		if (!diff) return "gray";
		if (diff < 0) return "green"; // Faster is better
		if (diff > 0) return "red"; // Slower is worse
		return "gray";
	}

	/**
	 * Format duration difference
	 */
	function formatDurationDiff(diff: number): string {
		const sign = diff > 0 ? "+" : "";
		return `${sign}${diff.toFixed(0)}ms`;
	}
</script>
