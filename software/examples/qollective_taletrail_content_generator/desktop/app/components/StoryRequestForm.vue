<template>
	<UCard class="max-w-2xl mx-auto">
		<template #header>
			<div class="flex items-center justify-between">
				<div>
					<h2 class="text-2xl font-bold font-heading">
						Create New Story
					</h2>
					<p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
						Configure your interactive story generation
					</p>
				</div>
				<UButton
					v-if="showClose"
					icon="i-heroicons-x-mark"
					color="gray"
					variant="ghost"
					size="sm"
					@click="$emit('close')"
				/>
			</div>
		</template>

		<form class="space-y-6" autocomplete="off" @submit.prevent="handleSubmit">
			<!-- Tenant Selection -->
			<div>
				<label class="block text-sm font-medium mb-2">
					Tenant <span class="text-red-500">*</span>
				</label>
				<TenantSelector
					v-model="form.tenantId"
					:available-tenants="availableTenants"
				/>
				<p v-if="errors.tenantId" class="text-sm text-red-600 dark:text-red-400 mt-1">
					{{ errors.tenantId }}
				</p>
			</div>

			<!-- Story Theme -->
			<div>
				<label for="theme" class="block text-sm font-medium mb-2">
					Story Theme <span class="text-red-500">*</span>
				</label>
				<UInput
					id="theme"
					v-model="form.theme"
					placeholder="e.g., Space Adventure, Medieval Quest"
					:maxlength="100"
					:class="{ 'border-red-500': errors.theme }"
					aria-describedby="theme-error theme-help"
					@input="validateField('theme')"
				/>
				<p id="theme-help" class="text-sm text-gray-600 dark:text-gray-500 mt-1">
					The main theme or setting for your story
				</p>
				<p v-if="errors.theme" id="theme-error" class="text-sm text-red-600 dark:text-red-400 mt-1">
					{{ errors.theme }}
				</p>
			</div>

			<!-- Story Setting -->
			<div>
				<label for="setting" class="block text-sm font-medium mb-2">
					Story Setting <span class="text-red-500">*</span>
				</label>
				<UTextarea
					id="setting"
					v-model="form.setting"
					placeholder="Describe the world, time period, and atmosphere of your story..."
					:maxlength="500"
					:rows="4"
					:class="{ 'border-red-500': errors.setting }"
					aria-describedby="setting-error setting-help"
					@input="validateField('setting')"
				/>
				<p id="setting-help" class="text-sm text-gray-600 dark:text-gray-500 mt-1">
					{{ form.setting.length }}/500 characters
				</p>
				<p v-if="errors.setting" id="setting-error" class="text-sm text-red-600 dark:text-red-400 mt-1">
					{{ errors.setting }}
				</p>
			</div>

			<!-- Target Audience -->
			<div>
				<label for="targetAudience" class="block text-sm font-medium mb-2">
					Target Audience <span class="text-red-500">*</span>
				</label>
				<USelectMenu
					id="targetAudience"
					v-model="form.targetAudience"
					:options="audienceOptions"
					value-attribute="value"
					option-attribute="label"
					:class="{ 'border-red-500': errors.targetAudience }"
					aria-describedby="targetAudience-error targetAudience-help"
					@update:model-value="validateField('targetAudience')"
				>
					<template #label>
						<span v-if="form.targetAudience">{{ getAudienceLabel(form.targetAudience) }}</span>
						<span v-else class="text-gray-500">Select target audience</span>
					</template>
				</USelectMenu>
				<p id="targetAudience-help" class="text-sm text-gray-600 dark:text-gray-500 mt-1">
					Choose the age group for your story
				</p>
				<p v-if="errors.targetAudience" id="targetAudience-error" class="text-sm text-red-600 dark:text-red-400 mt-1">
					{{ errors.targetAudience }}
				</p>
			</div>

			<!-- Complexity Level -->
			<div>
				<label for="complexityLevel" class="block text-sm font-medium mb-2">
					Complexity Level: <span class="font-semibold">{{ form.complexityLevel }}</span> <span class="text-red-500">*</span>
				</label>
				<div class="flex items-center gap-4">
					<span class="text-sm text-gray-600 dark:text-gray-400">Simple</span>
					<input
						id="complexityLevel"
						v-model.number="form.complexityLevel"
						type="range"
						min="1"
						max="5"
						step="1"
						class="flex-1"
						aria-describedby="complexityLevel-help"
						@input="validateField('complexityLevel')"
					>
					<span class="text-sm text-gray-600 dark:text-gray-400">Complex</span>
				</div>
				<p id="complexityLevel-help" class="text-sm text-gray-600 dark:text-gray-500 mt-1">
					{{ getComplexityDescription(form.complexityLevel) }}
				</p>
			</div>

			<!-- Branching Factor -->
			<div>
				<label for="branchingFactor" class="block text-sm font-medium mb-2">
					Branching Factor <span class="text-red-500">*</span>
				</label>
				<UInput
					id="branchingFactor"
					v-model.number="form.branchingFactor"
					type="number"
					min="2"
					max="8"
					:class="{ 'border-red-500': errors.branchingFactor }"
					aria-describedby="branchingFactor-error branchingFactor-help"
					@input="validateField('branchingFactor')"
				/>
				<p id="branchingFactor-help" class="text-sm text-gray-600 dark:text-gray-500 mt-1">
					Number of story nodes to generate (2-8)
				</p>
				<p v-if="errors.branchingFactor" id="branchingFactor-error" class="text-sm text-red-600 dark:text-red-400 mt-1">
					{{ errors.branchingFactor }}
				</p>
			</div>

			<!-- Form Actions -->
			<div class="flex gap-3 pt-4">
				<UButton
					type="submit"
					size="lg"
					:loading="submitting"
					:disabled="!isFormValid || submitting"
				>
					<template #leading>
						<UIcon name="i-heroicons-play" />
					</template>
					Generate Story
				</UButton>
				<UButton
					v-if="showClose"
					type="button"
					variant="ghost"
					size="lg"
					@click="$emit('close')"
				>
					Cancel
				</UButton>
			</div>
		</form>
	</UCard>
</template>

<script setup lang="ts">
	import { computed, ref, watch } from "vue";

	// Props
	const props = defineProps<{
		availableTenants: string[]
		showClose?: boolean
	}>();

	// Emits
	const emit = defineEmits<{
		submit: [request: StoryRequest]
		close: []
	}>();

	// Form interface matching backend GenerationRequest
	interface StoryRequest {
		tenantId: string
		theme: string
		setting: string
		targetAudience: string
		complexityLevel: number
		branchingFactor: number
	}

	// Form data
	const form = ref<StoryRequest>({
		tenantId: props.availableTenants[0] || "",
		theme: "",
		setting: "",
		targetAudience: "",
		complexityLevel: 3,
		branchingFactor: 4
	});

	// Errors
	const errors = ref<Partial<Record<keyof StoryRequest, string>>>({});

	// Submitting state
	const submitting = ref(false);

	// Audience options matching backend AgeGroup enum
	const audienceOptions = [
		{ label: "Children (6-8)", value: "6-8", description: "Simple vocabulary and short sentences" },
		{ label: "Pre-teens (9-11)", value: "9-11", description: "Elementary reading level" },
		{ label: "Young Teens (12-14)", value: "12-14", description: "Middle school reading level" },
		{ label: "Teens (15-17)", value: "15-17", description: "High school reading level" },
		{ label: "Adults (18+)", value: "+18", description: "Advanced vocabulary and themes" }
	];

	// Get audience label helper
	function getAudienceLabel(value: string): string {
		return audienceOptions.find((opt) => opt.value === value)?.label || value;
	}

	// Complexity descriptions
	function getComplexityDescription(level: number): string {
		const descriptions = [
			"",
			"Very simple story with minimal branching",
			"Simple story with some choices",
			"Moderate complexity with multiple paths",
			"Complex story with many decision points",
			"Very complex with intricate branching"
		];
		return descriptions[level] || "";
	}

	// Field validation
	function validateField(field: keyof StoryRequest): void {
		delete errors.value[field];

		switch (field) {
		case "tenantId":
			if (!form.value.tenantId) {
				errors.value.tenantId = "Please select a tenant";
			}
			break;

		case "theme":
			if (!form.value.theme.trim()) {
				errors.value.theme = "Story theme is required";
			} else if (form.value.theme.length > 100) {
				errors.value.theme = "Theme must be 100 characters or less";
			}
			break;

		case "setting":
			if (!form.value.setting.trim()) {
				errors.value.setting = "Story setting is required";
			} else if (form.value.setting.length > 500) {
				errors.value.setting = "Setting must be 500 characters or less";
			}
			break;

		case "targetAudience":
			if (!form.value.targetAudience) {
				errors.value.targetAudience = "Please select a target audience";
			}
			break;

		case "complexityLevel":
			if (form.value.complexityLevel < 1 || form.value.complexityLevel > 5) {
				errors.value.complexityLevel = "Complexity level must be between 1 and 5";
			}
			break;

		case "branchingFactor":
			if (form.value.branchingFactor < 2) {
				errors.value.branchingFactor = "Branching factor must be at least 2";
			} else if (form.value.branchingFactor > 8) {
				errors.value.branchingFactor = "Branching factor must be at most 8";
			}
			break;
		}
	}

	// Validate all fields
	function validateForm(): boolean {
		errors.value = {};

		validateField("tenantId");
		validateField("theme");
		validateField("setting");
		validateField("targetAudience");
		validateField("complexityLevel");
		validateField("branchingFactor");

		return Object.keys(errors.value).length === 0;
	}

	// Is form valid
	const isFormValid = computed(() => {
		return (
			form.value.tenantId
			&& form.value.theme.trim()
			&& form.value.setting.trim()
			&& form.value.targetAudience
			&& form.value.complexityLevel >= 1
			&& form.value.complexityLevel <= 5
			&& form.value.branchingFactor >= 2
			&& form.value.branchingFactor <= 8
		);
	});

	// Handle form submission
	async function handleSubmit(): Promise<void> {
		if (!validateForm()) {
			return;
		}

		submitting.value = true;
		try {
			emit("submit", { ...form.value });
		} finally {
			// Don't reset submitting here - parent will handle it
		}
	}

	// Watch tenantId prop changes
	watch(() => props.availableTenants, (newTenants) => {
		if (newTenants.length > 0 && !form.value.tenantId) {
			form.value.tenantId = newTenants[0];
		}
	}, { immediate: true });

	// Expose reset method for parent
	defineExpose({
		reset: () => {
			form.value = {
				tenantId: props.availableTenants[0] || "",
				theme: "",
				setting: "",
				targetAudience: "",
				complexityLevel: 3,
				branchingFactor: 4
			};
			errors.value = {};
			submitting.value = false;
		}
	});
</script>

<style scoped>
/* Custom range input styling */
input[type="range"] {
	appearance: none;
	height: 6px;
	background: rgb(var(--color-gray-300));
	border-radius: 3px;
	outline: none;
}

input[type="range"]::-webkit-slider-thumb {
	appearance: none;
	width: 20px;
	height: 20px;
	background: rgb(var(--color-primary-500));
	border-radius: 50%;
	cursor: pointer;
}

input[type="range"]::-moz-range-thumb {
	width: 20px;
	height: 20px;
	background: rgb(var(--color-primary-500));
	border-radius: 50%;
	cursor: pointer;
	border: none;
}

.dark input[type="range"] {
	background: rgb(var(--color-gray-700));
}
</style>
