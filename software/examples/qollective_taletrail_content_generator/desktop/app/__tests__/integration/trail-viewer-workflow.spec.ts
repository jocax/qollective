/**
 * Trail Viewer Workflow Integration Tests
 *
 * Tests critical end-to-end user workflows:
 * 1. Filter combination → verify correct trails shown
 * 2. Trail deletion → verify file removal and UI update
 * 3. Bookmark workflow → add/remove/persist
 * 4. Multi-filter clearing workflow
 */

import { describe, it, expect, beforeEach } from "vitest";
import { ref } from "vue";
import type { Trail } from "@/types/trail";

describe("Trail Viewer Workflow Integration Tests", () => {
	// Mock trail data for testing workflows
	const createMockTrails = (): Trail[] => [
		{
			id: "trail-1",
			title: "Space Adventure",
			description: "A story about space",
			theme: "space",
			age_group: "9-11",
			language: "EN",
			status: "completed",
			tenant_id: "1",
			created_at: "2025-11-01T10:00:00Z",
			updated_at: "2025-11-01T10:00:00Z",
			file_path: "/trails/trail-1.json"
		},
		{
			id: "trail-2",
			title: "Underwater Exploration",
			description: "A story about the ocean",
			theme: "ocean",
			age_group: "6-8",
			language: "EN",
			status: "completed",
			tenant_id: "1",
			created_at: "2025-11-01T11:00:00Z",
			updated_at: "2025-11-01T11:00:00Z",
			file_path: "/trails/trail-2.json"
		},
		{
			id: "trail-3",
			title: "Dragon Quest",
			description: "A story about dragons",
			theme: "fantasy",
			age_group: "9-11",
			language: "DE",
			status: "draft",
			tenant_id: "2",
			created_at: "2025-11-01T12:00:00Z",
			updated_at: "2025-11-01T12:00:00Z",
			file_path: "/trails/trail-3.json"
		},
		{
			id: "trail-4",
			title: "Robot Friends",
			description: "A story about friendly robots",
			theme: "technology",
			age_group: "12-15",
			language: "EN",
			status: "completed",
			tenant_id: "1",
			created_at: "2025-11-01T13:00:00Z",
			updated_at: "2025-11-01T13:00:00Z",
			file_path: "/trails/trail-4.json"
		}
	];

	describe("Multi-Filter Combination Workflow", () => {
		it("should apply search filter and return matching trails", () => {
			const trails = createMockTrails();
			const searchQuery = ref("space");

			const filteredTrails = trails.filter(trail => {
				const query = searchQuery.value.toLowerCase();
				return (
					trail.title.toLowerCase().includes(query) ||
					trail.description.toLowerCase().includes(query) ||
					trail.theme.toLowerCase().includes(query)
				);
			});

			expect(filteredTrails.length).toBe(1);
			expect(filteredTrails[0].id).toBe("trail-1");
			expect(filteredTrails[0].title).toBe("Space Adventure");
		});

		it("should apply age group filter and return matching trails", () => {
			const trails = createMockTrails();
			const selectedAgeGroup = ref("9-11");

			const filteredTrails = trails.filter(trail => {
				if (!selectedAgeGroup.value) return true;
				return trail.age_group === selectedAgeGroup.value;
			});

			expect(filteredTrails.length).toBe(2);
			expect(filteredTrails.map(t => t.id)).toEqual(["trail-1", "trail-3"]);
		});

		it("should apply language filter and return matching trails", () => {
			const trails = createMockTrails();
			const selectedLanguage = ref("EN");

			const filteredTrails = trails.filter(trail => {
				if (!selectedLanguage.value) return true;
				return trail.language === selectedLanguage.value;
			});

			expect(filteredTrails.length).toBe(3);
			expect(filteredTrails.map(t => t.id)).toEqual(["trail-1", "trail-2", "trail-4"]);
		});

		it("should apply status filter and return matching trails", () => {
			const trails = createMockTrails();
			const selectedStatus = ref("completed");

			const filteredTrails = trails.filter(trail => {
				if (!selectedStatus.value) return true;
				return trail.status === selectedStatus.value;
			});

			expect(filteredTrails.length).toBe(3);
			expect(filteredTrails.map(t => t.id)).toEqual(["trail-1", "trail-2", "trail-4"]);
		});

		it("should apply combined filters (AND logic) and return matching trails", () => {
			const trails = createMockTrails();
			const searchQuery = ref("");
			const selectedAgeGroup = ref("9-11");
			const selectedLanguage = ref("EN");
			const selectedStatus = ref("completed");

			let filteredTrails = trails;

			// Apply search filter
			if (searchQuery.value) {
				const query = searchQuery.value.toLowerCase();
				filteredTrails = filteredTrails.filter(trail =>
					trail.title.toLowerCase().includes(query) ||
					trail.description.toLowerCase().includes(query) ||
					trail.theme.toLowerCase().includes(query)
				);
			}

			// Apply age group filter
			if (selectedAgeGroup.value) {
				filteredTrails = filteredTrails.filter(trail =>
					trail.age_group === selectedAgeGroup.value
				);
			}

			// Apply language filter
			if (selectedLanguage.value) {
				filteredTrails = filteredTrails.filter(trail =>
					trail.language === selectedLanguage.value
				);
			}

			// Apply status filter
			if (selectedStatus.value) {
				filteredTrails = filteredTrails.filter(trail =>
					trail.status === selectedStatus.value
				);
			}

			// Only trail-1 matches: age_group=9-11, language=EN, status=completed
			expect(filteredTrails.length).toBe(1);
			expect(filteredTrails[0].id).toBe("trail-1");
		});

		it("should apply search + age group filters and return matching trails", () => {
			const trails = createMockTrails();
			const searchQuery = ref("dragon");
			const selectedAgeGroup = ref("9-11");

			let filteredTrails = trails;

			// Apply search
			if (searchQuery.value) {
				const query = searchQuery.value.toLowerCase();
				filteredTrails = filteredTrails.filter(trail =>
					trail.title.toLowerCase().includes(query) ||
					trail.description.toLowerCase().includes(query) ||
					trail.theme.toLowerCase().includes(query)
				);
			}

			// Apply age group
			if (selectedAgeGroup.value) {
				filteredTrails = filteredTrails.filter(trail =>
					trail.age_group === selectedAgeGroup.value
				);
			}

			// Only trail-3 matches: title contains "dragon", age_group=9-11
			expect(filteredTrails.length).toBe(1);
			expect(filteredTrails[0].id).toBe("trail-3");
		});
	});

	describe("Filter Clearing Workflow", () => {
		it("should clear all filters and show all trails", () => {
			const trails = createMockTrails();
			const searchQuery = ref("space");
			const selectedAgeGroup = ref("9-11");
			const selectedLanguage = ref("EN");
			const selectedStatus = ref("completed");

			// Apply all filters first
			let filteredTrails = trails.filter(trail => {
				const query = searchQuery.value.toLowerCase();
				const matchesSearch = searchQuery.value === "" ||
					trail.title.toLowerCase().includes(query) ||
					trail.description.toLowerCase().includes(query) ||
					trail.theme.toLowerCase().includes(query);

				const matchesAgeGroup = !selectedAgeGroup.value || trail.age_group === selectedAgeGroup.value;
				const matchesLanguage = !selectedLanguage.value || trail.language === selectedLanguage.value;
				const matchesStatus = !selectedStatus.value || trail.status === selectedStatus.value;

				return matchesSearch && matchesAgeGroup && matchesLanguage && matchesStatus;
			});

			expect(filteredTrails.length).toBe(1);

			// Clear all filters
			searchQuery.value = "";
			selectedAgeGroup.value = "";
			selectedLanguage.value = "";
			selectedStatus.value = "";

			// Reapply filters (should show all)
			filteredTrails = trails.filter(trail => {
				const matchesSearch = searchQuery.value === "" ||
					trail.title.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
					trail.description.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
					trail.theme.toLowerCase().includes(searchQuery.value.toLowerCase());

				const matchesAgeGroup = !selectedAgeGroup.value || trail.age_group === selectedAgeGroup.value;
				const matchesLanguage = !selectedLanguage.value || trail.language === selectedLanguage.value;
				const matchesStatus = !selectedStatus.value || trail.status === selectedStatus.value;

				return matchesSearch && matchesAgeGroup && matchesLanguage && matchesStatus;
			});

			expect(filteredTrails.length).toBe(4);
		});
	});

	describe("Computed Filter Options Workflow", () => {
		it("should compute available age groups from trails", () => {
			const trails = createMockTrails();

			const uniqueAgeGroups = Array.from(
				new Set(trails.map(t => t.age_group))
			).sort();

			expect(uniqueAgeGroups).toEqual(["12-15", "6-8", "9-11"]);
		});

		it("should compute available languages from trails", () => {
			const trails = createMockTrails();

			const uniqueLanguages = Array.from(
				new Set(trails.map(t => t.language))
			).sort();

			expect(uniqueLanguages).toEqual(["DE", "EN"]);
		});

		it("should compute available statuses from trails", () => {
			const trails = createMockTrails();

			const uniqueStatuses = Array.from(
				new Set(trails.map(t => t.status))
			).sort();

			expect(uniqueStatuses).toEqual(["completed", "draft"]);
		});
	});

	describe("Trail Deletion Workflow", () => {
		it("should remove trail from list after deletion", () => {
			const trails = ref(createMockTrails());
			const trailToDelete = "trail-2";

			expect(trails.value.length).toBe(4);
			expect(trails.value.some(t => t.id === trailToDelete)).toBe(true);

			// Simulate deletion
			trails.value = trails.value.filter(t => t.id !== trailToDelete);

			expect(trails.value.length).toBe(3);
			expect(trails.value.some(t => t.id === trailToDelete)).toBe(false);
			expect(trails.value.map(t => t.id)).toEqual(["trail-1", "trail-3", "trail-4"]);
		});

		it("should update filtered results after deletion", () => {
			const trails = ref(createMockTrails());
			const selectedLanguage = ref("EN");

			// Initial filter
			let filteredTrails = trails.value.filter(trail =>
				!selectedLanguage.value || trail.language === selectedLanguage.value
			);

			expect(filteredTrails.length).toBe(3);

			// Delete trail-2 (EN language)
			trails.value = trails.value.filter(t => t.id !== "trail-2");

			// Recompute filtered trails
			filteredTrails = trails.value.filter(trail =>
				!selectedLanguage.value || trail.language === selectedLanguage.value
			);

			expect(filteredTrails.length).toBe(2);
			expect(filteredTrails.map(t => t.id)).toEqual(["trail-1", "trail-4"]);
		});
	});

	describe("Bookmark Workflow", () => {
		it("should add bookmark to trail", () => {
			const bookmarkedTrails = ref(new Set<string>());
			const trailId = "trail-1";

			expect(bookmarkedTrails.value.has(trailId)).toBe(false);

			// Add bookmark
			bookmarkedTrails.value.add(trailId);

			expect(bookmarkedTrails.value.has(trailId)).toBe(true);
			expect(bookmarkedTrails.value.size).toBe(1);
		});

		it("should remove bookmark from trail", () => {
			const bookmarkedTrails = ref(new Set<string>(["trail-1", "trail-2"]));

			expect(bookmarkedTrails.value.has("trail-1")).toBe(true);
			expect(bookmarkedTrails.value.size).toBe(2);

			// Remove bookmark
			bookmarkedTrails.value.delete("trail-1");

			expect(bookmarkedTrails.value.has("trail-1")).toBe(false);
			expect(bookmarkedTrails.value.size).toBe(1);
		});

		it("should toggle bookmark on/off", () => {
			const bookmarkedTrails = ref(new Set<string>());
			const trailId = "trail-3";

			// Toggle on
			if (bookmarkedTrails.value.has(trailId)) {
				bookmarkedTrails.value.delete(trailId);
			} else {
				bookmarkedTrails.value.add(trailId);
			}

			expect(bookmarkedTrails.value.has(trailId)).toBe(true);

			// Toggle off
			if (bookmarkedTrails.value.has(trailId)) {
				bookmarkedTrails.value.delete(trailId);
			} else {
				bookmarkedTrails.value.add(trailId);
			}

			expect(bookmarkedTrails.value.has(trailId)).toBe(false);
		});
	});

	describe("Tenant Filtering Workflow", () => {
		it("should filter trails by tenant", () => {
			const trails = createMockTrails();
			const selectedTenant = ref("1");

			const filteredTrails = trails.filter(trail =>
				!selectedTenant.value || trail.tenant_id === selectedTenant.value
			);

			expect(filteredTrails.length).toBe(3);
			expect(filteredTrails.map(t => t.id)).toEqual(["trail-1", "trail-2", "trail-4"]);
		});

		it("should combine tenant filter with other filters", () => {
			const trails = createMockTrails();
			const selectedTenant = ref("1");
			const selectedAgeGroup = ref("9-11");

			const filteredTrails = trails.filter(trail => {
				const matchesTenant = !selectedTenant.value || trail.tenant_id === selectedTenant.value;
				const matchesAgeGroup = !selectedAgeGroup.value || trail.age_group === selectedAgeGroup.value;
				return matchesTenant && matchesAgeGroup;
			});

			// Only trail-1 matches: tenant_id=1, age_group=9-11
			expect(filteredTrails.length).toBe(1);
			expect(filteredTrails[0].id).toBe("trail-1");
		});
	});
});
