<template>
	<UContainer class="relative overflow-hidden min-h-screen">
		<div class="flex flex-col p-6">
			<!-- Header -->
			<div class="mb-6">
				<div class="flex items-center justify-between mb-4">
					<UButton variant="ghost" icon="i-heroicons-arrow-left" @click="goBack">
						Back
					</UButton>

					<div class="flex items-center gap-2">
						<UButton
							v-if="!loading && originalTrail"
							variant="ghost"
							icon="i-heroicons-eye"
							:to="`/viewer/${originalId}`"
						>
							View Original
						</UButton>
						<UButton
							v-if="!loading && newTrail"
							variant="ghost"
							icon="i-heroicons-eye"
							:to="`/viewer/${newId}`"
						>
							View New
						</UButton>
					</div>
				</div>
			</div>

			<!-- Loading State -->
			<div v-if="loading" class="flex items-center justify-center p-12">
				<div class="text-center">
					<div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-primary-500 mb-4" />
					<p class="text-gray-600 dark:text-gray-400">
						Loading trails for comparison...
					</p>
				</div>
			</div>

			<!-- Error State -->
			<UAlert
				v-else-if="error"
				color="red"
				variant="subtle"
				icon="i-heroicons-exclamation-triangle"
				title="Error Loading Trails"
				:description="error"
				class="mb-4"
			>
				<template #actions>
					<UButton color="red" variant="ghost" to="/">
						Return to List
					</UButton>
				</template>
			</UAlert>

			<!-- Comparison View -->
			<div v-else-if="originalTrail && newTrail">
				<ComparisonView
					v-model:sync-scrolling="syncScrolling"
					:original-trail="originalTrail"
					:new-trail="newTrail"
				/>
			</div>
		</div>
	</UContainer>
</template>

<script lang="ts" setup>
	import type { GenerationResponse, TrailListItem } from "~/types/trails";
	import { invoke } from "@tauri-apps/api/core";
	import { getTrailFilePath, saveRecentTrail } from "~/utils/trailStorage";

	definePageMeta({
		layout: "default",
		showInNav: false
	});

	const route = useRoute();
	const router = useRouter();

	// Route params
	const originalId = computed(() => route.params.originalId as string);
	const newId = computed(() => route.params.newId as string);

	// State
	const loading = ref(true);
	const error = ref<string | null>(null);
	const originalTrail = ref<GenerationResponse | null>(null);
	const newTrail = ref<GenerationResponse | null>(null);
	const syncScrolling = ref(false);

	/**
	 * Load trail data with fallback to directory search
	 */
	async function loadTrail(trailId: string): Promise<GenerationResponse | null> {
		try {
			console.log("[Compare] Loading trail:", trailId);

			// Try to get file path from recent trails first
			let filePath = getTrailFilePath(trailId);

			// Fallback: If not in recent trails, search the directory
			if (!filePath) {
				console.log("[Compare] Trail not in recent cache, searching directory...");

				try {
					// Get saved directory from localStorage
					const savedDirectory = import.meta.client ? localStorage.getItem("trail_directory") : null;

					if (!savedDirectory) {
						throw new Error("No trails directory configured. Please select a directory from the home page.");
					}

					// Load all trails from directory to find the one we need
					const allTrails = await invoke<TrailListItem[]>("load_trails_from_directory", {
						directory: savedDirectory
					});

					console.log("[Compare] Loaded trails from directory:", allTrails.length);

					// Find the trail by ID
					const trail = allTrails.find((t) => t.id === trailId);

					if (trail) {
						filePath = trail.file_path;
						console.log("[Compare] Found trail in directory:", filePath);

						// Save to recent trails for future use
						saveRecentTrail(trail);
					} else {
						throw new Error(`Trail with ID "${trailId}" not found in the trails directory. Available trails: ${allTrails.length}`);
					}
				} catch (dirErr) {
					console.error("[Compare] Failed to search directory:", dirErr);
					throw new Error(`Trail "${trailId}" not found. Unable to search directory: ${(dirErr as Error).message}`);
				}
			}

			if (!filePath) {
				throw new Error(`Unable to locate trail file for ID: ${trailId}`);
			}

			console.log("[Compare] Loading trail from file:", filePath);

			// Load full trail data
			const trail = await invoke<GenerationResponse>("load_trail_full", {
				filePath
			});

			console.log("[Compare] Trail loaded successfully:", trail.trail.title);

			return trail;
		} catch (err) {
			console.error("[Compare] Failed to load trail:", err);
			throw err;
		}
	}

	/**
	 * Load both trails for comparison
	 */
	async function loadTrails() {
		try {
			loading.value = true;
			error.value = null;

			console.log("[Compare] Loading trails for comparison:", {
				originalId: originalId.value,
				newId: newId.value
			});

			// Load both trails in parallel
			const [original, newTrail] = await Promise.all([
				loadTrail(originalId.value),
				loadTrail(newId.value)
			]);

			originalTrail.value = original;
			newTrail.value = newTrail;

			console.log("[Compare] Both trails loaded successfully");
		} catch (err) {
			console.error("[Compare] Failed to load trails:", err);
			error.value = (err as Error).message;
		} finally {
			loading.value = false;
		}
	}

	/**
	 * Go back to previous page
	 */
	function goBack() {
		router.back();
	}

	// Load trails on mount
	onMounted(() => {
		loadTrails();
	});

	// Update document title
	watchEffect(() => {
		if (originalTrail.value && newTrail.value) {
			useHead({
				title: `Compare: ${originalTrail.value.trail.title} vs ${newTrail.value.trail.title} - TaleTrail Viewer`
			});
		} else {
			useHead({
				title: "Trail Comparison - TaleTrail Viewer"
			});
		}
	});
</script>
