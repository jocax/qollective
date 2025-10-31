<template>
	<div class="space-y-2">
		<!-- Header -->
		<div class="flex items-center justify-between py-1">
			<h2 class="text-xl font-bold">
				{{ isReplay ? 'Replay Request' : 'New Request' }}
			</h2>
			<!-- Mode Toggle -->
			<div class="flex items-center gap-2">
				<UButton
					:variant="editorMode === 'form' ? 'solid' : 'ghost'"
					icon="i-heroicons-document-text"
					size="xs"
					@click="toggleMode('form')"
				>
					Form
				</UButton>
				<UButton
					:variant="editorMode === 'json' ? 'solid' : 'ghost'"
					icon="i-heroicons-code-bracket"
					size="xs"
					@click="toggleMode('json')"
				>
					JSON
				</UButton>
			</div>
		</div>

		<!-- Validation Error Summary -->
		<UAlert
			v-if="Object.keys(validationErrors).length > 0"
			color="red"
			variant="subtle"
			icon="i-heroicons-exclamation-triangle"
			title="Validation Errors"
		>
			<template #description>
				<ul class="list-disc list-inside space-y-1 text-xs">
					<li v-for="(error, field) in validationErrors" :key="field">
						<strong class="capitalize">{{ String(field).replace(/_/g, ' ') }}:</strong> {{ error }}
					</li>
				</ul>
			</template>
		</UAlert>

		<!-- Form Mode -->
		<div v-if="editorMode === 'form'" class="space-y-2">
			<!-- Row 1: Two columns (Request ID/Tenant + Story Parameters) -->
			<div class="grid grid-cols-2 gap-2">
				<!-- Request Identification -->
				<UCard :ui="{ body: { padding: 'p-3' } }">
					<template #header>
						<h3 class="text-base font-semibold">
							Request Identification
						</h3>
					</template>

					<div class="space-y-2">
						<div>
							<label class="block text-xs font-medium mb-1">Request ID</label>
							<UInput
								v-model="formData.request_id"
								placeholder="req-xxxxx"
								:error="!!validationErrors.request_id"
								disabled
								size="xs"
							/>
							<p v-if="validationErrors.request_id" class="text-red-500 text-xs mt-1">
								{{ validationErrors.request_id }}
							</p>
						</div>

						<div>
							<label class="block text-xs font-medium mb-1">Tenant ID</label>
							<UInput
								v-model="formData.tenant_id"
								placeholder="tenant-default"
								:error="!!validationErrors.tenant_id"
								size="xs"
							/>
							<p v-if="validationErrors.tenant_id" class="text-red-500 text-xs mt-1">
								{{ validationErrors.tenant_id }}
							</p>
						</div>
					</div>
				</UCard>

				<!-- Story Parameters -->
				<UCard :ui="{ body: { padding: 'p-3' } }">
					<template #header>
						<h3 class="text-base font-semibold">
							Story Parameters
						</h3>
					</template>

					<div class="space-y-2">
						<div>
							<label class="block text-xs font-medium mb-1">Theme *</label>
							<UInput
								v-model="formData.theme"
								placeholder="e.g., Space Adventure"
								:error="!!validationErrors.theme"
								size="xs"
							/>
							<p v-if="validationErrors.theme" class="text-red-500 text-xs mt-1">
								{{ validationErrors.theme }}
							</p>
						</div>

						<div class="grid grid-cols-3 gap-1">
							<div>
								<label class="block text-xs font-medium mb-1">Age *</label>
								<USelect
									v-model="formData.age_group"
									:items="ageGroups"
									size="xs"
								/>
							</div>

							<div>
								<label class="block text-xs font-medium mb-1">Vocab</label>
								<USelect
									v-model="formData.vocabulary_level"
									:items="vocabularyLevels"
									size="xs"
								/>
							</div>

							<div>
								<label class="block text-xs font-medium mb-1">Lang *</label>
								<USelect
									v-model="formData.language"
									:items="languages"
									size="xs"
								/>
							</div>
						</div>

						<div>
							<label class="block text-xs font-medium mb-1">Educational Focus</label>
							<div class="flex gap-1 mb-1">
								<UInput
									v-model="newTag"
									placeholder="Add tag"
									size="xs"
									@keyup.enter="addTag"
								/>
								<UButton icon="i-heroicons-plus" size="xs" @click="addTag" />
							</div>
							<div v-if="formData.educational_focus && formData.educational_focus.length > 0" class="flex flex-wrap gap-1">
								<UBadge
									v-for="(tag, index) in formData.educational_focus"
									:key="index"
									color="primary"
									variant="subtle"
									size="xs"
									class="cursor-pointer"
									@click="removeTag(index)"
								>
									{{ tag }}
									<UIcon name="i-heroicons-x-mark" class="ml-1" />
								</UBadge>
							</div>
						</div>
					</div>
				</UCard>
			</div>

			<!-- Row 2: Story Structure Presets -->
			<UCard :ui="{ body: { padding: 'p-3' } }">
				<template #header>
					<h3 class="text-base font-semibold">
						Story Structure
					</h3>
				</template>

				<div class="space-y-2">
					<!-- Preset Options - 4 columns -->
					<div class="grid grid-cols-5 gap-2">
						<UCard
							v-for="preset in storyStructures"
							:key="preset.value"
							:class="{ 'ring-2 ring-primary-500 bg-primary-50 dark:bg-primary-950': selectedPreset === preset.value }"
							class="cursor-pointer transition-all p-2"
							:ui="{ body: { padding: 'p-2' } }"
							@click="selectedPreset = preset.value"
						>
							<div class="flex flex-col items-center gap-1">
								<input
									v-model="selectedPreset"
									type="radio"
									:value="preset.value"
									class="h-3 w-3 text-primary-600 border-gray-300 focus:ring-primary-500"
								>
								<div class="text-center">
									<h4 class="text-xs font-semibold">
										{{ preset.label }}
									</h4>
									<p class="text-[10px] text-primary-600 dark:text-primary-400">
										{{ preset.node_count }}n
									</p>
								</div>
							</div>
						</UCard>

						<!-- Custom Option -->
						<UCard
							:class="{ 'ring-2 ring-primary-500 bg-primary-50 dark:bg-primary-950': selectedPreset === 'custom' }"
							class="cursor-pointer transition-all p-2"
							:ui="{ body: { padding: 'p-2' } }"
							@click="selectedPreset = 'custom'"
						>
							<div class="flex flex-col items-center gap-1">
								<input
									v-model="selectedPreset"
									type="radio"
									value="custom"
									class="h-3 w-3 text-primary-600 border-gray-300 focus:ring-primary-500"
								>
								<div class="text-center">
									<h4 class="text-xs font-semibold">
										Custom
									</h4>
									<p class="text-[10px] text-gray-500">
										8-32n
									</p>
								</div>
							</div>
						</UCard>
					</div>

					<!-- Custom Node Count Slider (compact inline) -->
					<div v-if="selectedPreset === 'custom'" class="flex items-center gap-2 pt-2 border-t">
						<label class="text-xs font-medium whitespace-nowrap">
							Nodes: {{ formData.node_count || 16 }}
						</label>
						<input
							v-model.number="formData.node_count"
							type="range"
							min="8"
							max="32"
							step="1"
							class="flex-1"
						>
						<span class="text-[10px] text-gray-500">8-32</span>
					</div>
				</div>
			</UCard>

			<!-- Row 3: Constraints (Optional, Collapsible, Collapsed by Default) -->
			<UCard :ui="{ body: { padding: 'p-3' } }">
				<template #header>
					<div class="flex items-center justify-between">
						<h3 class="text-base font-semibold">
							Constraints (Optional)
						</h3>
						<UButton
							variant="ghost"
							size="xs"
							:icon="showConstraints ? 'i-heroicons-chevron-up' : 'i-heroicons-chevron-down'"
							@click="showConstraints = !showConstraints"
						>
							{{ showConstraints ? 'Hide' : 'Show' }}
						</UButton>
					</div>
				</template>

				<div v-if="showConstraints" class="space-y-2">
					<div class="grid grid-cols-2 gap-2">
						<div>
							<label class="block text-xs font-medium mb-1">Max Choices Per Node</label>
							<UInput
								v-model.number="formData.constraints!.maxChoicesPerNode"
								type="number"
								min="2"
								max="10"
								placeholder="2-10"
								size="xs"
							/>
						</div>

						<div>
							<label class="block text-xs font-medium mb-1">Min Story Length</label>
							<UInput
								v-model.number="formData.constraints!.minStoryLength"
								type="number"
								min="100"
								max="10000"
								placeholder="100-10000"
								size="xs"
							/>
						</div>
					</div>
				</div>
			</UCard>
		</div>

		<!-- JSON Mode -->
		<div v-else class="space-y-2">
			<UAlert
				v-if="jsonError"
				color="error"
				variant="subtle"
				icon="i-heroicons-exclamation-triangle"
				title="JSON Error"
				:description="jsonError"
			/>

			<div>
				<label class="block text-xs font-medium mb-1">JSON Editor</label>
				<textarea
					v-model="jsonText"
					class="w-full h-80 p-3 font-mono text-xs border rounded-lg dark:bg-gray-800 dark:border-gray-700"
					:class="{ 'border-red-500': jsonError }"
				/>
				<p class="text-xs text-gray-500 mt-1">
					Edit JSON directly. Ensure valid JSON before submitting.
				</p>
			</div>
		</div>

		<!-- Actions -->
		<div class="flex items-center justify-end gap-2 pt-2">
			<UButton
				variant="ghost"
				size="xs"
				@click="handleCancel"
			>
				Cancel
			</UButton>
			<UButton
				color="primary"
				icon="i-heroicons-paper-airplane"
				size="xs"
				@click="handleSubmit"
			>
				{{ isReplay ? 'Submit Replay' : 'Submit Request' }}
			</UButton>
		</div>
	</div>
</template>

<script setup lang="ts">
	import type { StoryStructure, StoryStructureOption, SubmitGenerationRequest } from "~/types/trails";
	import { z } from "zod";

	interface Props {
		initialRequest?: Partial<SubmitGenerationRequest>
		isReplay?: boolean
		originalRequestId?: string
	}

	const props = defineProps<Props>();

	const emit = defineEmits<{
		submit: [request: SubmitGenerationRequest]
		cancel: []
	}>();

	const { generateRequestId } = useRequests();

	// Mode state
	const editorMode = ref<"form" | "json">("form");

	// FIXED: Only 2 languages matching schema (using value/label for USelect)
	const languages = [
		{ value: "en", label: "English" },
		{ value: "de", label: "German (Deutsch)" }
	];

	// FIXED: Correct age groups with value/label objects for USelect
	const ageGroups = [
		{ value: "6-8", label: "6-8 years" },
		{ value: "9-11", label: "9-11 years" },
		{ value: "12-14", label: "12-14 years" },
		{ value: "15-17", label: "15-17 years" },
		{ value: "+18", label: "18+ years" }
	];

	// FIXED: Correct vocabulary levels with value/label objects for USelect
	const vocabularyLevels = [
		{ value: "basic", label: "Basic" },
		{ value: "intermediate", label: "Intermediate" },
		{ value: "advanced", label: "Advanced" }
	];

	// NEW: Story structure presets
	const storyStructures: StoryStructureOption[] = [
		{
			value: "guided",
			label: "Guided Story",
			description: "Linear story with occasional choices",
			node_count: 12
		},
		{
			value: "adventure",
			label: "Adventure Story",
			description: "Branching paths with multiple convergence",
			node_count: 16
		},
		{
			value: "epic",
			label: "Epic Story",
			description: "Complex branching that converges at end",
			node_count: 24
		},
		{
			value: "choose_your_path",
			label: "Choose Your Path",
			description: "Pure branching with multiple endings",
			node_count: 16
		}
	];

	// NEW: Selected preset state (or 'custom')
	const selectedPreset = ref<StoryStructure | "custom">(
		props.initialRequest?.story_structure || "guided"
	);

	// Form state
	const formData = ref<SubmitGenerationRequest>({
		request_id: props.initialRequest?.request_id || generateRequestId(),
		tenant_id: props.initialRequest?.tenant_id || "1",
		theme: props.initialRequest?.theme || "Space Adventure", // FIXED: provide reasonable default (min 5 chars required)
		age_group: props.initialRequest?.age_group || "6-8", // FIXED: correct default
		language: props.initialRequest?.language || "en",
		vocabulary_level: props.initialRequest?.vocabulary_level || "basic", // FIXED: correct default
		node_count: props.initialRequest?.node_count || 12, // FIXED: default to guided preset's node count (backend requires this)
		story_structure: props.initialRequest?.story_structure || "guided", // NEW: default to guided
		educational_focus: props.initialRequest?.educational_focus || [],
		constraints: props.initialRequest?.constraints || {},
		metadata: props.initialRequest?.metadata || {
			submitted_at: new Date().toISOString(),
			original_request_id: props.originalRequestId
		}
	});

	// Watch preset changes to update form data
	watch(selectedPreset, (newPreset) => {
		if (newPreset !== "custom") {
			// Set story_structure and set node_count to preset's value
			formData.value.story_structure = newPreset;
			// Set node_count based on preset (backend requires this field)
			const presetConfig = storyStructures.find((p) => p.value === newPreset);
			formData.value.node_count = presetConfig?.node_count || 16;
		} else {
			// Remove story_structure and use custom node_count
			delete formData.value.story_structure;
			if (!formData.value.node_count) {
				formData.value.node_count = 16; // Default for custom
			}
		}
	});

	// JSON editor state
	const jsonText = ref(JSON.stringify(formData.value, null, 2));
	const jsonError = ref<string | null>(null);

	// Show constraints section - collapsed by default
	const showConstraints = ref(false);

	// Educational focus tag input
	const newTag = ref("");

	// FIXED: Zod validation schema with correct enums and optional fields
	const requestSchema = z.object({
		request_id: z.string().min(1, "Request ID is required"),
		tenant_id: z.string().min(1, "Tenant ID is required"),
		theme: z.string().min(5, "Theme must be at least 5 characters").max(200, "Theme too long"),
		age_group: z.enum(["6-8", "9-11", "12-14", "15-17", "+18"]), // FIXED
		language: z.enum(["de", "en"]), // FIXED: only 2 languages
		vocabulary_level: z.enum(["basic", "intermediate", "advanced"]).optional(), // FIXED
		story_structure: z.enum(["guided", "adventure", "epic", "choose_your_path"]).optional(), // NEW
		node_count: z.number().min(8, "Minimum 8 nodes").max(32, "Maximum 32 nodes").optional(),
		educational_focus: z.array(z.string()).optional(),
		constraints: z.object({
			maxChoicesPerNode: z.number().min(2).max(10).optional(),
			minStoryLength: z.number().min(100).max(10000).optional(),
			forbiddenTopics: z.array(z.string()).optional(),
			requiredTopics: z.array(z.string()).optional()
		}).optional()
	});

	// Validation errors
	const validationErrors = ref<Record<string, string>>({});

	/**
	 * Validate form data
	 */
	function validateForm(): boolean {
		validationErrors.value = {};

		try {
			requestSchema.parse(formData.value);
			return true;
		} catch (error) {
			if (error instanceof z.ZodError) {
				error.errors.forEach((err) => {
					const path = err.path.join(".");
					validationErrors.value[path] = err.message;
				});
			}
			return false;
		}
	}

	/**
	 * Add educational focus tag
	 */
	function addTag() {
		if (newTag.value.trim() && !formData.value.educational_focus?.includes(newTag.value.trim())) {
			if (!formData.value.educational_focus) {
				formData.value.educational_focus = [];
			}
			formData.value.educational_focus.push(newTag.value.trim());
			newTag.value = "";
		}
	}

	/**
	 * Remove educational focus tag
	 */
	function removeTag(index: number) {
		formData.value.educational_focus?.splice(index, 1);
	}

	/**
	 * Toggle editor mode and sync data
	 */
	function toggleMode(mode: "form" | "json") {
		if (mode === "json" && editorMode.value === "form") {
			// Form to JSON
			jsonText.value = JSON.stringify(formData.value, null, 2);
			jsonError.value = null;
		} else if (mode === "form" && editorMode.value === "json") {
			// JSON to Form
			try {
				const parsed = JSON.parse(jsonText.value);
				formData.value = parsed;
				jsonError.value = null;
			} catch (err) {
				jsonError.value = `Invalid JSON: ${err instanceof Error ? err.message : String(err)}`;
				return;
			}
		}
		editorMode.value = mode;
	}

	/**
	 * Handle form submission
	 */
	function handleSubmit() {
		// Sync from JSON if in JSON mode
		if (editorMode.value === "json") {
			try {
				const parsed = JSON.parse(jsonText.value);
				formData.value = parsed;
				jsonError.value = null;
			} catch (err) {
				jsonError.value = `Invalid JSON: ${err instanceof Error ? err.message : String(err)}`;
				return;
			}
		}

		// Validate
		if (!validateForm()) {
			return;
		}

		// Update metadata
		formData.value.metadata = {
			...formData.value.metadata,
			submittedAt: new Date().toISOString(),
			originalRequestId: props.originalRequestId
		};

		emit("submit", formData.value);
	}

	/**
	 * Handle cancel
	 */
	function handleCancel() {
		emit("cancel");
	}

	// Watch JSON text for real-time validation
	watch(jsonText, () => {
		if (editorMode.value === "json") {
			try {
				JSON.parse(jsonText.value);
				jsonError.value = null;
			} catch (err) {
				jsonError.value = `Invalid JSON: ${err instanceof Error ? err.message : String(err)}`;
			}
		}
	});
</script>
