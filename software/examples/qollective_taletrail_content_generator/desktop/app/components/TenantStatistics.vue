<template>
	<UCard class="tenant-statistics">
		<template #header>
			<div class="flex items-center justify-between cursor-pointer" @click="isOpen = !isOpen">
				<div class="flex items-center gap-2">
					<UIcon name="i-heroicons-chart-bar" class="w-5 h-5 text-primary-500" />
					<h3 class="text-lg font-semibold">
						Tenant Statistics
					</h3>
					<UBadge color="gray" variant="subtle">
						{{ statistics.length }} tenant{{ statistics.length !== 1 ? 's' : '' }}
					</UBadge>
				</div>
				<UIcon
					:name="isOpen ? 'i-heroicons-chevron-up' : 'i-heroicons-chevron-down'"
					class="w-5 h-5 text-gray-500"
				/>
			</div>
		</template>

		<div v-if="isOpen" class="space-y-4">
			<!-- Overview Stats -->
			<div class="grid grid-cols-2 gap-4 p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
				<div>
					<div class="text-xs text-gray-600 dark:text-gray-400 mb-1">
						Total Trails
					</div>
					<div class="text-2xl font-bold text-gray-900 dark:text-gray-100">
						{{ totalTrails }}
					</div>
				</div>
				<div>
					<div class="text-xs text-gray-600 dark:text-gray-400 mb-1">
						Avg Success Rate
					</div>
					<div class="text-2xl font-bold text-gray-900 dark:text-gray-100">
						{{ averageSuccessRate }}%
					</div>
				</div>
			</div>

			<!-- Per-Tenant Statistics -->
			<div class="space-y-3">
				<div
					v-for="stat in sortedStats"
					:key="stat.tenantId"
					class="p-3 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors"
				>
					<div class="flex items-start justify-between mb-2">
						<div class="flex items-center gap-2">
							<UBadge :color="getTenantColorForStat(stat.tenantId)" variant="soft">
								<template #leading>
									<UIcon name="i-heroicons-user" class="w-3 h-3" />
								</template>
								{{ getTenantDisplayName(stat.tenantId) }}
							</UBadge>
						</div>
						<div class="text-sm font-medium text-gray-900 dark:text-gray-100">
							{{ stat.trailCount }} trail{{ stat.trailCount !== 1 ? 's' : '' }}
						</div>
					</div>

					<div class="grid grid-cols-2 gap-4 text-sm">
						<div>
							<span class="text-gray-600 dark:text-gray-400">Success Rate:</span>
							<UBadge
								:color="getSuccessRateColor(stat.successRate)"
								variant="subtle"
								size="xs"
								class="ml-2"
							>
								{{ Math.round(stat.successRate) }}%
							</UBadge>
						</div>
						<div>
							<span class="text-gray-600 dark:text-gray-400">Avg Nodes:</span>
							<span class="ml-2 font-medium text-gray-900 dark:text-gray-100">
								{{ stat.averageNodeCount }}
							</span>
						</div>
					</div>
				</div>
			</div>

			<!-- Empty State -->
			<div v-if="statistics.length === 0" class="text-center py-8">
				<div class="text-gray-400 mb-2">
					<UIcon name="i-heroicons-chart-bar" class="w-12 h-12 mx-auto" />
				</div>
				<p class="text-sm text-gray-600 dark:text-gray-400">
					No tenant statistics available
				</p>
			</div>
		</div>
	</UCard>
</template>

<script setup lang="ts">
	import type { TenantStatistics } from "~/types/trails";
	import { computed } from "vue";
	import { getTenantColor, getTenantDisplayName } from "~/utils/tenantColors";

	const props = defineProps<{
		statistics: TenantStatistics[]
		isCollapsed?: boolean
	}>();

	const isOpen = ref(!props.isCollapsed);

	const sortedStats = computed(() => {
		return [...props.statistics].sort((a, b) => b.trailCount - a.trailCount);
	});

	const totalTrails = computed(() => {
		return props.statistics.reduce((sum, stat) => sum + stat.trailCount, 0);
	});

	const averageSuccessRate = computed(() => {
		if (props.statistics.length === 0) return 0;
		const total = props.statistics.reduce((sum, stat) => sum + stat.successRate, 0);
		return Math.round(total / props.statistics.length);
	});

	function getTenantColorForStat(tenantId: string): string {
		return getTenantColor(tenantId);
	}

	function getSuccessRateColor(rate: number): string {
		if (rate >= 90) return "green";
		if (rate >= 70) return "yellow";
		return "red";
	}
</script>

<style scoped>
.tenant-statistics {
  width: 100%;
}
</style>
