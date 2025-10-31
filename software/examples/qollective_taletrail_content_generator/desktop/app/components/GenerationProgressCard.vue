<template>
	<UCard class="transition-all duration-200 hover:shadow-md">
		<!-- Header -->
		<div class="space-y-3">
			<!-- Status and Controls Row -->
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-2">
					<UBadge :color="statusColor" variant="subtle" size="lg">
						{{ request.status }}
					</UBadge>
					<span class="text-xs text-gray-500 dark:text-gray-400">
						{{ formattedStartTime }}
					</span>
				</div>
				<UButton
					variant="ghost"
					size="xs"
					:icon="isExpanded ? 'i-heroicons-chevron-up' : 'i-heroicons-chevron-down'"
					@click="toggleExpanded"
				>
					{{ isExpanded ? 'Hide' : 'Details' }}
				</UButton>
			</div>

			<!-- Request ID and Tenant -->
			<div class="space-y-1">
				<div class="flex items-center gap-2 text-sm">
					<UIcon name="i-heroicons-hashtag" class="text-gray-400" />
					<span class="font-mono text-gray-700 dark:text-gray-300 font-semibold">
						{{ request.requestId.substring(0, 8) }}...
					</span>
				</div>
				<div class="flex items-center gap-2 text-xs">
					<UIcon name="i-heroicons-building-office" class="text-gray-400" />
					<span class="text-gray-600 dark:text-gray-400">
						Tenant: {{ request.tenantId }}
					</span>
					<UIcon name="i-heroicons-clock" class="text-gray-400 ml-2" />
					<span class="text-gray-600 dark:text-gray-400">
						{{ duration }}
					</span>
				</div>
			</div>

			<!-- Overall Progress Bar -->
			<div v-if="request.status === 'in_progress'" class="space-y-1">
				<div class="flex items-center justify-between text-xs">
					<span class="text-gray-600 dark:text-gray-400">Overall Progress</span>
					<span class="font-semibold text-gray-700 dark:text-gray-300">{{ overallProgress }}%</span>
				</div>
				<UProgress :value="overallProgress" :color="statusColor" size="sm" />
			</div>

			<!-- Error Message -->
			<div v-if="request.errorMessage" class="mt-2">
				<UAlert
					color="red"
					variant="subtle"
					title="Error"
					:description="request.errorMessage"
					class="text-xs"
				/>
			</div>

			<!-- Expanded Details -->
			<div v-if="isExpanded" class="pt-3 border-t border-gray-200 dark:border-gray-700 space-y-3">
				<!-- Phase Progress Bars -->
				<div class="space-y-2">
					<h4 class="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
						Service Phases
					</h4>
					<div
						v-for="phase in SERVICE_PHASES"
						:key="phase.id"
						class="space-y-1"
					>
						<div class="flex items-center justify-between text-xs">
							<div class="flex items-center gap-2">
								<UIcon :name="phase.icon" class="text-gray-500" />
								<span class="text-gray-600 dark:text-gray-400">{{ phase.label }}</span>
							</div>
							<div class="flex items-center gap-2">
								<UBadge
									:color="getPhaseStatusColor(phase.id)"
									variant="soft"
									size="xs"
								>
									{{ getPhaseProgress(phase.id)?.status ?? 'pending' }}
								</UBadge>
								<span class="font-semibold text-gray-700 dark:text-gray-300 min-w-[3ch] text-right">
									{{ getPhaseProgressPercent(phase.id) }}%
								</span>
							</div>
						</div>
						<UProgress
							:value="getPhaseProgressPercent(phase.id)"
							:color="getPhaseStatusColor(phase.id)"
							size="xs"
						/>
						<!-- Phase Error Message -->
						<div v-if="getPhaseProgress(phase.id)?.errorMessage" class="mt-1">
							<p class="text-xs text-red-600 dark:text-red-400">
								{{ getPhaseProgress(phase.id)?.errorMessage }}
							</p>
						</div>
					</div>
				</div>

				<!-- Full Request ID -->
				<div class="pt-2 border-t border-gray-200 dark:border-gray-700">
					<div class="text-xs space-y-1">
						<span class="text-gray-500 dark:text-gray-500">Full Request ID:</span>
						<div class="font-mono text-[10px] text-gray-600 dark:text-gray-400 break-all bg-gray-100 dark:bg-gray-800 p-2 rounded">
							{{ request.requestId }}
						</div>
					</div>
				</div>
			</div>
		</div>
	</UCard>
</template>

<script setup lang="ts">
	import type { GenerationRequest, PhaseProgress } from "~/types/trails";

	interface Props {
		request: GenerationRequest
	}

	const props = defineProps<Props>();

	// Expanded state for details
	const isExpanded = ref(false);

	// Service phases in order
	const SERVICE_PHASES = [
		{ id: "prompt-helper", label: "Prompt Helper", icon: "i-heroicons-sparkles" },
		{ id: "story-generator", label: "Story Generator", icon: "i-heroicons-book-open" },
		{ id: "quality-control", label: "Quality Control", icon: "i-heroicons-shield-check" },
		{ id: "constraint-enforcer", label: "Constraint Enforcer", icon: "i-heroicons-check-badge" }
	];

	// Status color mapping
	const statusColor = computed(() => {
		switch (props.request.status) {
		case "completed":
			return "green";
		case "failed":
			return "red";
		case "in_progress":
			return "blue";
		default:
			return "gray";
		}
	});

	// Overall progress calculation
	const overallProgress = computed(() => {
		const phases = Array.from(props.request.phases.values());
		if (phases.length === 0) return 0;

		const totalProgress = phases.reduce((sum, phase) => sum + phase.progress, 0);
		return Math.round((totalProgress / phases.length) * 100);
	});

	// Formatted timestamps
	const formattedStartTime = computed(() => {
		try {
			const date = new Date(props.request.startTime);
			return date.toLocaleTimeString("en-US", {
				hour: "2-digit",
				minute: "2-digit",
				second: "2-digit"
			});
		} catch {
			return props.request.startTime;
		}
	});

	// Duration calculation
	const duration = computed(() => {
		const start = new Date(props.request.startTime);
		const now = new Date();
		const diffMs = now.getTime() - start.getTime();
		const diffSec = Math.floor(diffMs / 1000);

		if (diffSec < 60) return `${diffSec}s`;
		const diffMin = Math.floor(diffSec / 60);
		const remainSec = diffSec % 60;
		return `${diffMin}m ${remainSec}s`;
	});

	// Get phase progress
	function getPhaseProgress(phaseId: string): PhaseProgress | undefined {
		return props.request.phases.get(phaseId);
	}

	// Get phase status color
	function getPhaseStatusColor(phaseId: string): string {
		const phase = getPhaseProgress(phaseId);
		if (!phase) return "gray";

		switch (phase.status) {
		case "completed":
			return "green";
		case "failed":
			return "red";
		case "in_progress":
			return "blue";
		default:
			return "gray";
		}
	}

	// Get phase progress percentage
	function getPhaseProgressPercent(phaseId: string): number {
		const phase = getPhaseProgress(phaseId);
		return phase ? Math.round(phase.progress * 100) : 0;
	}

	// Toggle expanded state
	function toggleExpanded() {
		isExpanded.value = !isExpanded.value;
	}
</script>

<style scoped>
/* Additional custom styles if needed */
</style>
