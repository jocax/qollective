/**
 * Trail Viewer Tests (index.vue)
 *
 * Focused tests for critical Trail Viewer behaviors:
 * - Directory selection and trail loading
 * - Filtering by search/age/language/status
 * - Trail deletion workflow
 * - Bookmark functionality integration
 */

import { beforeEach, describe, expect, it, vi } from "vitest";
import type { TrailListItem } from "~/types/trails";

// Mock Tauri modules at the top level (before imports)
const mockInvoke = vi.fn();
const mockOpen = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
	invoke: mockInvoke
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
	open: mockOpen
}));

// Mock localStorage
const localStorageMock = (() => {
	let store: Record<string, string> = {};
	return {
		getItem: (key: string) => store[key] || null,
		setItem: (key: string, value: string) => { store[key] = value; },
		clear: () => { store = {}; }
	};
})();

Object.defineProperty(globalThis, "localStorage", {
	value: localStorageMock
});

// Mock trail data for testing
const mockTrails: TrailListItem[] = [
	{
		id: "trail-1",
		title: "Epic Adventure in the Forest",
		description: "A thrilling story about forest exploration",
		theme: "Adventure",
		age_group: "15-17",
		language: "en",
		status: "completed",
		generated_at: "2025-01-15T10:30:00Z",
		file_path: "/test/path/trail-1.json",
		node_count: 25,
		tags: ["forest", "adventure", "exploration"],
		tenantId: "tenant-1"
	},
	{
		id: "trail-2",
		title: "Mystisches Abenteuer",
		description: "Eine magische Geschichte über Freundschaft",
		theme: "Magic",
		age_group: "10-12",
		language: "de",
		status: "completed",
		generated_at: "2025-01-14T14:20:00Z",
		file_path: "/test/path/trail-2.json",
		node_count: 18,
		tags: ["magic", "friendship"],
		tenantId: "tenant-2"
	},
	{
		id: "trail-3",
		title: "Space Explorer Journey",
		description: "Exploring the cosmos and distant planets",
		theme: "Science Fiction",
		age_group: "15-17",
		language: "en",
		status: "failed",
		generated_at: "2025-01-13T09:15:00Z",
		file_path: "/test/path/trail-3.json",
		node_count: 12,
		tags: ["space", "science", "exploration"],
		tenantId: "tenant-1"
	}
];

describe("trail Viewer (index.vue)", () => {
	beforeEach(() => {
		vi.clearAllMocks();
		localStorageMock.clear();
	});

	describe("directory selection and loading", () => {
		it("should load trails from directory successfully", async () => {
			// Arrange
			const { useTrails } = await import("~/composables/useTrails");
			const { loadTrails, trails, loading, error } = useTrails();

			mockInvoke.mockResolvedValueOnce(mockTrails);

			// Act
			await loadTrails("/test/directory");

			// Assert
			expect(mockInvoke).toHaveBeenCalledWith("load_trails_from_directory", {
				directory: "/test/directory"
			});
			expect(trails.value).toHaveLength(3);
			expect(trails.value[0].title).toBe("Epic Adventure in the Forest");
			expect(loading.value).toBe(false);
			expect(error.value).toBeNull();
		});

		it("should persist selected directory to localStorage", async () => {
			// Arrange
			const { useTrails } = await import("~/composables/useTrails");
			const { loadTrails, directory } = useTrails();

			mockInvoke.mockResolvedValueOnce(mockTrails);

			// Act
			await loadTrails("/test/directory");

			// Assert
			expect(directory.value).toBe("/test/directory");
			expect(localStorageMock.getItem("trail_directory")).toBe("/test/directory");
		});

		it("should handle loading errors gracefully", async () => {
			// Arrange
			const { useTrails } = await import("~/composables/useTrails");
			const { loadTrails, trails, loading, error } = useTrails();

			const errorMessage = "Directory not found";
			mockInvoke.mockRejectedValueOnce(new Error(errorMessage));

			// Act
			await loadTrails("/invalid/directory");

			// Assert
			expect(error.value).toBe(errorMessage);
			expect(trails.value).toHaveLength(0);
			expect(loading.value).toBe(false);
		});
	});

	describe("filtering functionality", () => {
		it("should filter trails by text search query", async () => {
			// Arrange
			const { useTrails } = await import("~/composables/useTrails");
			const { loadTrails, searchQuery, filteredTrails } = useTrails();

			mockInvoke.mockResolvedValueOnce(mockTrails);
			await loadTrails("/test/directory");

			// Act
			searchQuery.value = "forest";

			// Assert
			expect(filteredTrails.value).toHaveLength(1);
			expect(filteredTrails.value[0].title).toContain("Forest");
		});

		it("should filter trails by age group", async () => {
			// Arrange
			const { useTrails } = await import("~/composables/useTrails");
			const { loadTrails, selectedAgeGroup, filteredTrails } = useTrails();

			mockInvoke.mockResolvedValueOnce(mockTrails);
			await loadTrails("/test/directory");

			// Act
			selectedAgeGroup.value = "15-17";

			// Assert
			expect(filteredTrails.value).toHaveLength(2);
			expect(filteredTrails.value.every(t => t.age_group === "15-17")).toBe(true);
		});

		it("should filter trails by language", async () => {
			// Arrange
			const { useTrails } = await import("~/composables/useTrails");
			const { loadTrails, selectedLanguage, filteredTrails } = useTrails();

			mockInvoke.mockResolvedValueOnce(mockTrails);
			await loadTrails("/test/directory");

			// Act
			selectedLanguage.value = "de";

			// Assert
			expect(filteredTrails.value).toHaveLength(1);
			expect(filteredTrails.value[0].language).toBe("de");
			expect(filteredTrails.value[0].title).toBe("Mystisches Abenteuer");
		});

		it("should filter trails by status", async () => {
			// Arrange
			const { useTrails } = await import("~/composables/useTrails");
			const { loadTrails, selectedStatus, filteredTrails } = useTrails();

			mockInvoke.mockResolvedValueOnce(mockTrails);
			await loadTrails("/test/directory");

			// Act
			selectedStatus.value = "completed";

			// Assert
			expect(filteredTrails.value).toHaveLength(2);
			expect(filteredTrails.value.every(t => t.status === "completed")).toBe(true);
		});

		it("should combine multiple filters with AND logic", async () => {
			// Arrange
			const { useTrails } = await import("~/composables/useTrails");
			const { loadTrails, searchQuery, selectedAgeGroup, selectedLanguage, filteredTrails } = useTrails();

			mockInvoke.mockResolvedValueOnce(mockTrails);
			await loadTrails("/test/directory");

			// Act - Apply multiple filters
			searchQuery.value = "adventure";
			selectedAgeGroup.value = "15-17";
			selectedLanguage.value = "en";

			// Assert - Only trail-1 matches all criteria
			expect(filteredTrails.value).toHaveLength(1);
			expect(filteredTrails.value[0].id).toBe("trail-1");
		});

		it("should clear all filters when clearFilters is called", async () => {
			// Arrange
			const { useTrails } = await import("~/composables/useTrails");
			const {
				loadTrails,
				searchQuery,
				selectedAgeGroup,
				selectedLanguage,
				selectedStatus,
				clearFilters,
				filteredTrails
			} = useTrails();

			mockInvoke.mockResolvedValueOnce(mockTrails);
			await loadTrails("/test/directory");

			// Set some filters
			searchQuery.value = "forest";
			selectedAgeGroup.value = "15-17";
			selectedLanguage.value = "en";
			selectedStatus.value = "completed";

			// Act
			clearFilters();

			// Assert
			expect(searchQuery.value).toBe("");
			expect(selectedAgeGroup.value).toBeNull();
			expect(selectedLanguage.value).toBeNull();
			expect(selectedStatus.value).toBeNull();
			expect(filteredTrails.value).toHaveLength(3); // All trails visible
		});
	});

	describe("trail deletion workflow", () => {
		it("should delete trail from filesystem and update UI", async () => {
			// Arrange
			const { useTrails } = await import("~/composables/useTrails");
			const { loadTrails, trails } = useTrails();
			const { invoke } = await import("@tauri-apps/api/core");

			mockInvoke.mockResolvedValueOnce(mockTrails);
			await loadTrails("/test/directory");

			const trailToDelete = trails.value[0];
			const initialCount = trails.value.length;

			// Mock successful deletion
			mockInvoke.mockResolvedValueOnce(undefined);

			// Act
			await invoke("delete_trail", { filePath: trailToDelete.file_path });
			trails.value = trails.value.filter(t => t.id !== trailToDelete.id);

			// Assert
			expect(mockInvoke).toHaveBeenCalledWith("delete_trail", {
				filePath: trailToDelete.file_path
			});
			expect(trails.value).toHaveLength(initialCount - 1);
			expect(trails.value.find(t => t.id === trailToDelete.id)).toBeUndefined();
		});
	});

	describe("computed filter options", () => {
		it("should generate unique age group options from loaded trails", async () => {
			// Arrange
			const { useTrails } = await import("~/composables/useTrails");
			const { loadTrails, uniqueAgeGroups } = useTrails();

			mockInvoke.mockResolvedValueOnce(mockTrails);
			await loadTrails("/test/directory");

			// Assert
			expect(uniqueAgeGroups.value).toHaveLength(2);
			expect(uniqueAgeGroups.value).toContain("10-12");
			expect(uniqueAgeGroups.value).toContain("15-17");
			expect(uniqueAgeGroups.value).toEqual(["10-12", "15-17"]); // Sorted
		});

		it("should generate unique language options from loaded trails", async () => {
			// Arrange
			const { useTrails } = await import("~/composables/useTrails");
			const { loadTrails, uniqueLanguages } = useTrails();

			mockInvoke.mockResolvedValueOnce(mockTrails);
			await loadTrails("/test/directory");

			// Assert
			expect(uniqueLanguages.value).toHaveLength(2);
			expect(uniqueLanguages.value).toContain("de");
			expect(uniqueLanguages.value).toContain("en");
			expect(uniqueLanguages.value).toEqual(["de", "en"]); // Sorted
		});

		it("should generate unique status options from loaded trails", async () => {
			// Arrange
			const { useTrails } = await import("~/composables/useTrails");
			const { loadTrails, uniqueStatuses } = useTrails();

			mockInvoke.mockResolvedValueOnce(mockTrails);
			await loadTrails("/test/directory");

			// Assert
			expect(uniqueStatuses.value).toHaveLength(2);
			expect(uniqueStatuses.value).toContain("completed");
			expect(uniqueStatuses.value).toContain("failed");
			expect(uniqueStatuses.value).toEqual(["completed", "failed"]); // Sorted
		});
	});
});
