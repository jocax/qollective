<template>
	<UModal :model-value="show" prevent-close title="Generating Your Story">
		<UCard>
			<!-- Success Message -->
			<div v-if="showSuccessMessage" class="p-8 text-center">
				<div class="mb-4 text-green-500">
					<UIcon name="i-heroicons-check-circle" class="w-16 h-16 mx-auto" />
				</div>
				<h3 class="text-xl font-semibold text-green-700 dark:text-green-400 mb-2">
					Story Generated Successfully!
				</h3>
				<p class="text-sm text-gray-600 dark:text-gray-400">
					Redirecting to viewer...
				</p>
			</div>

			<!-- Progress Monitor -->
			<div v-else class="space-y-6">
				<!-- Header -->
				<div class="border-b border-gray-200 dark:border-gray-700 pb-4">
					<h2 class="text-2xl font-bold mb-2">
						Generating Your Story...
					</h2>
					<div class="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
						<UIcon name="i-heroicons-hashtag" class="w-4 h-4" />
						<span class="font-mono">{{ requestId.substring(0, 12) }}...</span>
						<UBadge :color="statusColor" variant="subtle" class="ml-2">
							{{ currentRequest?.status || 'pending' }}
						</UBadge>
					</div>
				</div>

				<!-- Phase Tracker -->
				<div class="space-y-4">
					<h3 class="text-sm font-semibold text-gray-700 dark:text-gray-300">
						Generation Pipeline
					</h3>
					<div class="flex items-center justify-between gap-2">
						<div
							v-for="(phase, index) in SERVICE_PHASES"
							:key="phase.id"
							class="flex-1"
						>
							<div class="flex flex-col items-center gap-2">
								<!-- Phase Icon -->
								<div
									class="w-12 h-12 rounded-full flex items-center justify-center transition-all duration-300" :class="[
										{
											'bg-green-100 dark:bg-green-900 text-green-600 dark:text-green-400': getPhaseStatus(phase.id) === 'completed',
											'bg-blue-100 dark:bg-blue-900 text-blue-600 dark:text-blue-400 ring-2 ring-blue-500 ring-offset-2': getPhaseStatus(phase.id) === 'active',
											'bg-red-100 dark:bg-red-900 text-red-600 dark:text-red-400': getPhaseStatus(phase.id) === 'failed',
											'bg-gray-100 dark:bg-gray-800 text-gray-400': getPhaseStatus(phase.id) === 'pending'
										}
									]"
								>
									<UIcon :name="phase.icon" class="w-6 h-6" />
								</div>

								<!-- Phase Label -->
								<span
									class="text-xs text-center font-medium" :class="[
										{
											'text-green-700 dark:text-green-400': getPhaseStatus(phase.id) === 'completed',
											'text-blue-700 dark:text-blue-400': getPhaseStatus(phase.id) === 'active',
											'text-red-700 dark:text-red-400': getPhaseStatus(phase.id) === 'failed',
											'text-gray-500 dark:text-gray-500': getPhaseStatus(phase.id) === 'pending'
										}
									]"
								>
									{{ phase.label }}
								</span>

								<!-- Progress Percentage -->
								<span
									v-if="getPhaseStatus(phase.id) !== 'pending'"
									class="text-xs font-semibold"
									:class="{
										'text-green-600 dark:text-green-400': getPhaseStatus(phase.id) === 'completed',
										'text-blue-600 dark:text-blue-400': getPhaseStatus(phase.id) === 'active',
										'text-red-600 dark:text-red-400': getPhaseStatus(phase.id) === 'failed'
									}"
								>
									{{ getPhaseProgressPercent(phase.id) }}%
								</span>
							</div>

							<!-- Connector Line -->
							<div
								v-if="index < SERVICE_PHASES.length - 1"
								class="h-0.5 w-full mt-6 transition-all duration-300" :class="[
									{
										'bg-green-500': getPhaseStatus(phase.id) === 'completed',
										'bg-blue-500': getPhaseStatus(phase.id) === 'active',
										'bg-gray-300 dark:bg-gray-700': getPhaseStatus(phase.id) === 'pending'
									}
								]"
							/>
						</div>
					</div>
				</div>

				<!-- Current Phase Card -->
				<UCard v-if="currentPhase" class="bg-blue-50 dark:bg-blue-950">
					<div class="flex items-start gap-4">
						<div class="flex-shrink-0">
							<UIcon :name="currentPhase.icon" class="w-8 h-8 text-blue-600 dark:text-blue-400" />
						</div>
						<div class="flex-1">
							<h4 class="font-semibold text-blue-900 dark:text-blue-100 mb-1">
								{{ currentPhase.label }}
							</h4>
							<p class="text-sm text-blue-700 dark:text-blue-300">
								{{ currentActivity }}
							</p>
							<div class="mt-3 space-y-1">
								<div class="flex items-center justify-between text-xs text-blue-600 dark:text-blue-400">
									<span>Progress</span>
									<span class="font-semibold">{{ getPhaseProgressPercent(currentPhase.id) }}%</span>
								</div>
								<UProgress
									:value="getPhaseProgressPercent(currentPhase.id)"
									color="blue"
									size="sm"
								/>
							</div>
						</div>
					</div>
				</UCard>

				<!-- Overall Progress -->
				<div class="space-y-2">
					<div class="flex items-center justify-between text-sm">
						<span class="font-semibold text-gray-700 dark:text-gray-300">
							Overall Progress
						</span>
						<div class="flex items-center gap-3">
							<span v-if="estimatedTimeRemaining" class="text-xs text-gray-500 dark:text-gray-400">
								<UIcon name="i-heroicons-clock" class="w-3 h-3 inline mr-1" />
								{{ estimatedTimeRemaining }}
							</span>
							<span class="font-bold text-gray-900 dark:text-gray-100">
								{{ overallProgress }}%
							</span>
						</div>
					</div>
					<UProgress
						:value="overallProgress"
						:color="statusColor"
						size="md"
					/>
				</div>

				<!-- Error Message -->
				<UAlert
					v-if="currentRequest?.errorMessage"
					color="red"
					variant="subtle"
					title="Generation Error"
					:description="currentRequest.errorMessage"
				/>

				<!-- Cancel Button -->
				<div class="flex justify-end pt-4 border-t border-gray-200 dark:border-gray-700">
					<UButton
						variant="outline"
						color="gray"
						@click="handleCancel"
					>
						Cancel Generation
					</UButton>
				</div>
			</div>
		</UCard>
	</UModal>
</template>

<script setup lang="ts">
	import { computed, onMounted, watch } from "vue";
	import { useNatsLiveMonitor } from "~/composables/useNatsLiveMonitor";
	import { PATHS } from "~/config/constants";

	interface Props {
		requestId: string
		show: boolean
	}

	const props = defineProps<Props>();
	const emit = defineEmits<{
		complete: [trailId: string]
		cancel: []
	}>();

	// Use existing NATS live monitor composable
	const {
		events,
		getRequest,
		subscribe,
		cancel: cancelMonitor
	} = useNatsLiveMonitor();

	// Service phases configuration
	const SERVICE_PHASES = [
		{ id: "prompt-helper", label: "Prompt Helper", icon: "i-heroicons-sparkles", weight: 5 },
		{ id: "story-generator", label: "Story Generator", icon: "i-heroicons-book-open", weight: 60 },
		{ id: "quality-control", label: "Quality Control", icon: "i-heroicons-shield-check", weight: 25 },
		{ id: "constraint-enforcer", label: "Constraint Enforcer", icon: "i-heroicons-check-badge", weight: 10 }
	] as const;

	// Get current request data
	const currentRequest = computed(() => {
		if (!props.requestId) return null;
		return getRequest(props.requestId);
	});

	// Filter events for this request
	const requestEvents = computed(() => {
		return events.value.filter((e) => e.requestId === props.requestId);
	});

	// Current active phase
	const currentPhase = computed(() => {
		if (!currentRequest.value) return null;

		// Find the phase that is currently in progress
		for (const phase of SERVICE_PHASES) {
			const phaseProgress = currentRequest.value.phases.get(phase.id);
			if (phaseProgress && phaseProgress.status === "in_progress") {
				return {
					...phase,
					progress: phaseProgress
				};
			}
		}

		// If no phase is in progress, find the last completed phase
		for (let i = SERVICE_PHASES.length - 1; i >= 0; i--) {
			const phase = SERVICE_PHASES[i];
			const phaseProgress = currentRequest.value.phases.get(phase.id);
			if (phaseProgress && phaseProgress.status === "completed") {
				return {
					...phase,
					progress: phaseProgress
				};
			}
		}

		return null;
	});

	// Overall progress calculation (weighted by phase importance)
	const overallProgress = computed(() => {
		if (!currentRequest.value) return 0;

		let totalWeight = 0;
		let completedWeight = 0;

		SERVICE_PHASES.forEach((phase) => {
			totalWeight += phase.weight;
			const phaseProgress = currentRequest.value!.phases.get(phase.id);
			if (phaseProgress) {
				if (phaseProgress.status === "completed") {
					completedWeight += phase.weight;
				} else if (phaseProgress.status === "in_progress") {
					completedWeight += phase.weight * phaseProgress.progress;
				}
			}
		});

		return Math.min(100, Math.round((completedWeight / totalWeight) * 100));
	});

	// Status color mapping
	const statusColor = computed(() => {
		if (!currentRequest.value) return "gray";

		switch (currentRequest.value.status) {
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

	// Time estimate (approximate 2-5 minutes total)
	const estimatedTimeRemaining = computed(() => {
		if (!currentRequest.value || currentRequest.value.status !== "in_progress") {
			return null;
		}

		const progress = overallProgress.value;
		if (progress === 0) return "2-5 minutes";

		const startTime = new Date(currentRequest.value.startTime);
		const now = new Date();
		const elapsedSeconds = Math.floor((now.getTime() - startTime.getTime()) / 1000);

		// Estimate based on current progress
		const estimatedTotalSeconds = (elapsedSeconds / progress) * 100;
		const remainingSeconds = Math.max(0, estimatedTotalSeconds - elapsedSeconds);

		if (remainingSeconds < 60) {
			return `~${Math.ceil(remainingSeconds)} seconds`;
		}

		const minutes = Math.ceil(remainingSeconds / 60);
		return `~${minutes} minute${minutes !== 1 ? "s" : ""}`;
	});

	// Current activity description
	const currentActivity = computed(() => {
		if (!currentPhase.value) return "Initializing...";

		const phase = currentPhase.value;
		const progress = phase.progress;

		if (progress.status === "completed") {
			return `${phase.label} completed`;
		}

		if (progress.status === "failed") {
			return `${phase.label} failed`;
		}

		// Generate descriptive message based on phase
		switch (phase.id) {
		case "prompt-helper":
			return "Analyzing request and optimizing prompt...";
		case "story-generator":
			return "Generating interactive story content...";
		case "quality-control":
			return "Validating story quality and coherence...";
		case "constraint-enforcer":
			return "Ensuring constraints and age appropriateness...";
		default:
			return `Processing ${phase.label}...`;
		}
	});

	// Show success message
	const showSuccessMessage = ref(false);

	/**
	 * Extract trail ID from file_path
	 * Format: test-trails/{tenant_id}_{request_id}.json
	 * Returns: {tenant_id}_{request_id}
	 */
	function extractTrailIdFromPath(filePath: string | undefined): string {
		if (!filePath) {
			// Fallback to request ID if no file path
			console.warn("[GenerationProgressMonitor] No file_path in completion event, using request_id");
			return props.requestId;
		}

		// Extract from path: test-trails/1_req-123.json → 1_req-123
		const match = filePath.match(new RegExp(`${PATHS.DEFAULT_TRAILS_DIR}/(.+)\\.json$`));
		if (match && match[1]) {
			console.log("[GenerationProgressMonitor] Extracted trail ID:", match[1], "from", filePath);
			return match[1];
		}

		console.warn("[GenerationProgressMonitor] Could not parse file_path:", filePath, ", using request_id");
		return props.requestId;
	}

	// Watch for completion
	watch(() => currentRequest.value?.status, (newStatus) => {
		if (newStatus === "completed") {
			console.log("[GenerationProgressMonitor] Generation completed, finding completion event...");

			// Find the completion event that should contain file_path
			const completionEvent = requestEvents.value.find(
				(e) => e.status === "completed" && e.requestId === props.requestId
			);

			console.log("[GenerationProgressMonitor] Completion event:", completionEvent);

			// Extract trail ID from file_path
			const trailId = extractTrailIdFromPath(completionEvent?.filePath);

			console.log("[GenerationProgressMonitor] Will navigate to trail:", trailId);

			showSuccessMessage.value = true;

			// Auto-dismiss after 2 seconds and emit with trail ID
			setTimeout(() => {
				showSuccessMessage.value = false;
				console.log("[GenerationProgressMonitor] Emitting complete event with trail ID:", trailId);
				emit("complete", trailId);
			}, 2000);
		}
	});

	// Get phase status indicator
	function getPhaseStatus(phaseId: string): "pending" | "active" | "completed" | "failed" {
		if (!currentRequest.value) return "pending";

		const phaseProgress = currentRequest.value.phases.get(phaseId);
		if (!phaseProgress) return "pending";

		switch (phaseProgress.status) {
		case "completed":
			return "completed";
		case "failed":
			return "failed";
		case "in_progress":
			return "active";
		default:
			return "pending";
		}
	}

	// Get phase progress percentage
	function getPhaseProgressPercent(phaseId: string): number {
		if (!currentRequest.value) return 0;

		const phaseProgress = currentRequest.value.phases.get(phaseId);
		return phaseProgress ? Math.round(phaseProgress.progress * 100) : 0;
	}

	// Handle cancel
	async function handleCancel() {
		await cancelMonitor();
		emit("cancel");
	}

	// Subscribe on mount if showing
	onMounted(async () => {
		if (props.show && currentRequest.value) {
			// Already subscribed through parent component
			console.log("[GenerationProgressMonitor] Monitoring request:", props.requestId);
		}
	});
</script>

<style scoped>
/* Animation for active phase ring */
@keyframes pulse-ring {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

.ring-2 {
  animation: pulse-ring 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}
</style>
