import type { TenantStatistics, TrailListItem } from "~/types/trails";
import { computed, onMounted, ref, watch } from "vue";

export function useTenantContext() {
	const selectedTenant = ref<string | null>(null);
	const allTrails = ref<TrailListItem[]>([]);

	// Load saved tenant preference from localStorage on mount
	onMounted(() => {
		if (typeof window !== "undefined") {
			const saved = localStorage.getItem("selectedTenant");
			if (saved && saved !== "null") {
				selectedTenant.value = saved;
			}
		}
	});

	// Watch and save to localStorage
	watch(selectedTenant, (newValue) => {
		if (typeof window !== "undefined") {
			if (newValue) {
				localStorage.setItem("selectedTenant", newValue);
			} else {
				localStorage.setItem("selectedTenant", "null");
			}
		}
	});

	// Extract unique tenant IDs from loaded trails
	const availableTenants = computed(() => {
		const tenantSet = new Set<string>();
		allTrails.value.forEach((trail) => {
			if (trail.tenantId) {
				tenantSet.add(trail.tenantId);
			}
		});
		return Array.from(tenantSet).sort();
	});

	// Check if viewing all tenants
	const isAllTenants = computed(() => selectedTenant.value === null);

	// Get display name for current tenant
	const currentTenantDisplay = computed(() => {
		if (isAllTenants.value) {
			return "All Tenants";
		}
		return selectedTenant.value || "Unknown";
	});

	// Filter trails by selected tenant
	const filteredTrailsByTenant = computed(() => {
		if (isAllTenants.value) {
			return allTrails.value;
		}
		return allTrails.value.filter((trail) => trail.tenantId === selectedTenant.value);
	});

	// Calculate statistics per tenant
	const tenantStatistics = computed((): TenantStatistics[] => {
		const stats = new Map<string, { total: number, completed: number, totalNodes: number }>();

		allTrails.value.forEach((trail) => {
			const tenantId = trail.tenantId || "no-tenant";
			const existing = stats.get(tenantId) || { total: 0, completed: 0, totalNodes: 0 };

			existing.total++;
			if (trail.status === "completed") {
				existing.completed++;
			}
			existing.totalNodes += trail.node_count || 0;

			stats.set(tenantId, existing);
		});

		return Array.from(stats.entries()).map(([tenantId, data]) => ({
			tenantId,
			trailCount: data.total,
			successRate: data.total > 0 ? (data.completed / data.total) * 100 : 0,
			averageNodeCount: data.total > 0 ? Math.round(data.totalNodes / data.total) : 0
		})).sort((a, b) => b.trailCount - a.trailCount);
	});

	// Get statistics for current tenant
	const currentTenantStats = computed(() => {
		if (isAllTenants.value) {
			// Aggregate all stats
			const total = allTrails.value.length;
			const completed = allTrails.value.filter((t) => t.status === "completed").length;
			const totalNodes = allTrails.value.reduce((sum, t) => sum + (t.node_count || 0), 0);

			return {
				tenantId: "all",
				trailCount: total,
				successRate: total > 0 ? (completed / total) * 100 : 0,
				averageNodeCount: total > 0 ? Math.round(totalNodes / total) : 0
			};
		}

		return tenantStatistics.value.find((s) => s.tenantId === selectedTenant.value) || {
			tenantId: selectedTenant.value || "unknown",
			trailCount: 0,
			successRate: 0,
			averageNodeCount: 0
		};
	});

	// Set selected tenant
	function setSelectedTenant(tenantId: string | null) {
		selectedTenant.value = tenantId;
	}

	// Update trails data (called from useTrails when trails are loaded)
	function updateTrailsData(trails: TrailListItem[]) {
		allTrails.value = trails;
	}

	// Clear tenant selection
	function clearTenantSelection() {
		selectedTenant.value = null;
	}

	return {
		selectedTenant,
		availableTenants,
		isAllTenants,
		currentTenantDisplay,
		filteredTrailsByTenant,
		tenantStatistics,
		currentTenantStats,
		setSelectedTenant,
		updateTrailsData,
		clearTenantSelection
	};
}
