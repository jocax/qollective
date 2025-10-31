import type { GenerationResponse, TrailListItem } from "../types/trails";
import { invoke } from "@tauri-apps/api/core";
import { computed, ref } from "vue";
import { reconstructDAG } from "../utils/dagReconstruction";
import { clearRecentTrails } from "../utils/trailStorage";

export function useTrails() {
	const trails = ref<TrailListItem[]>([]);
	const loading = ref(false);
	const error = ref<string | null>(null);
	const directory = ref<string>("");

	// Filter states
	const searchQuery = ref("");
	const selectedAgeGroup = ref<string | null>(null);
	const selectedLanguage = ref<string | null>(null);
	const selectedStatus = ref<string | null>(null);

	// Computed unique values for filters
	const uniqueAgeGroups = computed(() => {
		const groups = new Set(trails.value.map((t) => t.age_group));
		return Array.from(groups).sort();
	});

	const uniqueLanguages = computed(() => {
		const langs = new Set(trails.value.map((t) => t.language));
		return Array.from(langs).sort();
	});

	const uniqueStatuses = computed(() => {
		const statuses = new Set(trails.value.map((t) => t.status));
		return Array.from(statuses).sort();
	});

	// Computed filtered trails
	const filteredTrails = computed(() => {
		let filtered = trails.value;

		// Search query filter
		if (searchQuery.value) {
			const query = searchQuery.value.toLowerCase();
			filtered = filtered.filter((trail) =>
				trail.title.toLowerCase().includes(query)
				|| trail.description.toLowerCase().includes(query)
				|| trail.theme.toLowerCase().includes(query)
			);
		}

		// Age group filter
		if (selectedAgeGroup.value) {
			filtered = filtered.filter((trail) => trail.age_group === selectedAgeGroup.value);
		}

		// Language filter
		if (selectedLanguage.value) {
			filtered = filtered.filter((trail) => trail.language === selectedLanguage.value);
		}

		// Status filter
		if (selectedStatus.value) {
			filtered = filtered.filter((trail) => trail.status === selectedStatus.value);
		}

		return filtered;
	});

	// Load trails from directory
	async function loadTrails(dir: string) {
		loading.value = true;
		error.value = null;

		try {
			// Detect if directory has changed
			const previousDirectory = directory.value;
			const isDirectoryChange = previousDirectory && previousDirectory !== dir;

			if (isDirectoryChange) {
				console.log("[useTrails] Directory changed from", previousDirectory, "to", dir);
				console.log("[useTrails] Clearing recent trails from previous directory");
				clearRecentTrails();
			}

			console.log("[useTrails] Loading trails from directory:", dir);
			const result = await invoke<TrailListItem[]>("load_trails_from_directory", {
				directory: dir
			});

			console.log("[useTrails] Successfully loaded", result.length, "trails");
			console.log("[useTrails] Trail IDs:", result.map((t) => t.id));

			trails.value = result;
			directory.value = dir;

			// Save directory to preferences
			if (import.meta.client) {
				localStorage.setItem("trail_directory", dir);
			}
		} catch (err) {
			console.error("[useTrails] Error loading trails:", err);
			error.value = err instanceof Error ? err.message : String(err);
			trails.value = [];
		} finally {
			loading.value = false;
		}
	}

	// Load full trail data
	async function loadTrailFull(filePath: string): Promise<GenerationResponse | null> {
		try {
			const result = await invoke<GenerationResponse>("load_trail_full", {
				filePath
			});

			// Reconstruct DAG from trail_steps if dag is missing
			if (result?.trail && result.trail_steps && !result.trail.dag) {
				console.log("[useTrails] DAG missing, attempting reconstruction from trail_steps", {
					stepCount: result.trail_steps.length
				});

				// Try to get start_node_id from metadata
				const start_node_id = result.trail.metadata.start_node_id
					|| result.trail.metadata.generation_params?.start_node_id;

				if (!start_node_id) {
					console.error("[useTrails] Cannot reconstruct DAG: start_node_id missing in metadata");
					error.value = "Invalid trail data: missing start node ID";
					return null;
				}

				if (result.trail_steps.length === 0) {
					console.error("[useTrails] Cannot reconstruct DAG: trail_steps array is empty");
					error.value = "Invalid trail data: no trail steps found";
					return null;
				}

				try {
					// Reconstruct DAG and attach to trail
					result.trail.dag = reconstructDAG(result.trail_steps, start_node_id);
					console.log("[useTrails] Successfully reconstructed DAG from trail_steps:", {
						nodeCount: Object.keys(result.trail.dag.nodes).length,
						edgeCount: result.trail.dag.edges.length,
						convergencePoints: result.trail.dag.convergence_points?.length || 0,
						startNodeId: result.trail.dag.start_node_id
					});
				} catch (reconstructErr) {
					console.error("[useTrails] Failed to reconstruct DAG:", reconstructErr);
					error.value = `Failed to reconstruct trail structure: ${(reconstructErr as Error).message}`;
					return null;
				}
			} else if (result?.trail && result.trail.dag) {
				console.log("[useTrails] Trail loaded with existing DAG:", {
					nodeCount: Object.keys(result.trail.dag.nodes).length,
					edgeCount: result.trail.dag.edges.length
				});
			}

			return result;
		} catch (err) {
			error.value = err instanceof Error ? err.message : String(err);
			return null;
		}
	}

	// Clear all filters
	function clearFilters() {
		searchQuery.value = "";
		selectedAgeGroup.value = null;
		selectedLanguage.value = null;
		selectedStatus.value = null;
	}

	// Load saved directory on mount
	function loadSavedDirectory() {
		if (import.meta.client) {
			const saved = localStorage.getItem("trail_directory");
			if (saved) {
				loadTrails(saved);
			}
			// Note: No auto-load on first launch - user must select directory
			// Backend handles default directory configuration via config.toml
		}
	}

	return {
		trails,
		loading,
		error,
		directory,
		searchQuery,
		selectedAgeGroup,
		selectedLanguage,
		selectedStatus,
		uniqueAgeGroups,
		uniqueLanguages,
		uniqueStatuses,
		filteredTrails,
		loadTrails,
		loadTrailFull,
		clearFilters,
		loadSavedDirectory
	};
}
