<template>
	<UContainer class="relative overflow-hidden h-screen">
		<div class="flex flex-col h-full p-6">
			<!-- Header Section -->
			<div class="mb-6">
				<h1 class="text-3xl font-bold font-heading mb-2">
					TaleTrail Desktop Viewer
				</h1>
				<p class="text-gray-600 dark:text-gray-400">
					View and explore AI-generated interactive story trails
				</p>
			</div>

			<!-- Toolbar Section -->
			<div class="mb-6 space-y-4">
				<!-- Action Buttons Row -->
				<div class="flex gap-3">
					<UButton variant="solid" size="lg" icon="i-heroicons-folder-open" @click="selectDirectory">
						Load Directory
					</UButton>
					<UButton variant="outline" size="lg" icon="i-heroicons-arrow-path" :loading="loading" @click="reloadTrails">
						Reload
					</UButton>
				</div>

				<!-- Search and Filters Row -->
				<div v-if="directory" class="flex flex-wrap gap-3">
					<UInput
						v-model="searchQuery"
						placeholder="Search trails..."
						icon="i-heroicons-magnifying-glass"
						class="flex-1 min-w-[200px]"
					/>

					<!-- Tenant Selector -->
					<TenantSelector
						v-if="availableTenants.length > 0"
						v-model="selectedTenant"
						:available-tenants="availableTenants"
					/>

					<USelectMenu
						v-model="selectedAgeGroup"
						:options="ageGroupOptions"
						placeholder="Age Group"
						class="w-40"
					/>

					<USelectMenu
						v-model="selectedLanguage"
						:options="languageOptions"
						placeholder="Language"
						class="w-32"
					/>

					<USelectMenu
						v-model="selectedStatus"
						:options="statusOptions"
						placeholder="Status"
						class="w-32"
					/>

					<UButton
						variant="ghost"
						icon="i-heroicons-x-mark"
						@click="clearFilters"
					>
						Clear Filters
					</UButton>
				</div>

				<!-- Info Banner -->
				<div v-if="directory" class="space-y-2">
					<div class="flex items-center justify-between text-sm text-gray-600 dark:text-gray-400">
						<div class="flex items-center gap-3">
							<span>
								Showing {{ displayedTrails.length }} of {{ trails.length }} trails
							</span>
							<!-- Tenant Context Indicator -->
							<UBadge
								v-if="!isAllTenants"
								:color="getTenantColor(selectedTenant)"
								variant="subtle"
							>
								<template #leading>
									<UIcon name="i-heroicons-user" class="w-3 h-3" />
								</template>
								Viewing: {{ currentTenantDisplay }}
							</UBadge>
							<UBadge
								v-else-if="availableTenants.length > 0"
								color="gray"
								variant="subtle"
							>
								<template #leading>
									<UIcon name="i-heroicons-user-group" class="w-3 h-3" />
								</template>
								Viewing all tenants ({{ availableTenants.length }})
							</UBadge>
						</div>
						<span v-if="bookmarks.length > 0" class="flex items-center gap-1">
							<UIcon name="i-heroicons-star-solid" class="text-yellow-500" />
							{{ bookmarks.length }} bookmark{{ bookmarks.length !== 1 ? 's' : '' }}
						</span>
					</div>
					<div class="flex items-center gap-2 text-xs bg-gray-100 dark:bg-gray-800 p-2 rounded">
						<UIcon name="i-heroicons-folder-open" class="text-gray-500" />
						<span class="font-medium text-gray-700 dark:text-gray-300">Directory:</span>
						<span class="font-mono text-gray-600 dark:text-gray-400 flex-1 truncate" :title="directory">{{ directory }}</span>
					</div>
				</div>
			</div>

			<!-- Content Section -->
			<div class="flex-1 overflow-auto">
				<!-- Error State -->
				<UAlert
					v-if="error"
					color="red"
					variant="subtle"
					title="Error loading trails"
					:description="error"
					class="mb-4"
				/>

				<!-- Loading State -->
				<div v-if="loading" class="flex items-center justify-center p-12">
					<div class="text-center">
						<div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-primary-500 mb-4" />
						<p class="text-gray-600 dark:text-gray-400">
							Loading trails...
						</p>
					</div>
				</div>

				<!-- Empty State -->
				<div v-else-if="!directory" class="flex items-center justify-center h-full">
					<UCard class="max-w-md">
						<div class="p-8 text-center">
							<div class="mb-4 text-gray-400">
								<svg class="w-16 h-16 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
								</svg>
							</div>
							<h3 class="text-lg font-semibold mb-2">
								No directory selected
							</h3>
							<p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
								Select a directory containing trail JSON files to get started
							</p>
							<UButton variant="solid" icon="i-heroicons-folder-open" @click="selectDirectory">
								Load Directory
							</UButton>
						</div>
					</UCard>
				</div>

				<!-- No Results State -->
				<div v-else-if="displayedTrails.length === 0" class="flex items-center justify-center p-12">
					<UCard class="max-w-md">
						<div class="p-8 text-center">
							<div class="mb-4 text-gray-400">
								<svg class="w-16 h-16 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
								</svg>
							</div>
							<h3 class="text-lg font-semibold mb-2">
								No trails found
							</h3>
							<p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
								No trails match your current filters. Try adjusting your search criteria.
							</p>
							<UButton variant="outline" icon="i-heroicons-x-mark" @click="clearFilters">
								Clear Filters
							</UButton>
						</div>
					</UCard>
				</div>

				<!-- Trail Grid -->
				<div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
					<TrailCard
						v-for="trail in displayedTrails"
						:key="trail.id"
						:trail="trail"
						@delete="handleDeleteTrail"
					/>
				</div>
			</div>
		</div>
	</UContainer>
</template>

<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";
	import { getTenantColor } from "~/utils/tenantColors";
	import { removeRecentTrail } from "~/utils/trailStorage";

	definePageMeta({
		layout: "default"
	});

	const toast = useToast();

	const {
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
		clearFilters,
		loadSavedDirectory
	} = useTrails();

	// Bookmarks integration
	const { bookmarks, bookmarkedTrailIds, loadBookmarks, removeBookmark } = useBookmarks();

	// Tenant context integration
	const {
		selectedTenant,
		availableTenants,
		isAllTenants,
		currentTenantDisplay,
		tenantStatistics,
		updateTrailsData
	} = useTenantContext();

	// Watch trails and update tenant context
	watch(trails, (newTrails) => {
		updateTrailsData(newTrails);
	});

	// Computed for displayed trails with tenant filter
	const displayedTrails = computed(() => {
		let filteredList = filteredTrails.value;

		// Apply tenant filter
		if (!isAllTenants.value) {
			filteredList = filteredList.filter((t) => t.tenantId === selectedTenant.value);
		}

		return filteredList;
	});

	// Computed options for select menus
	const ageGroupOptions = computed(() =>
		uniqueAgeGroups.value.map((group) => ({ label: group, value: group }))
	);

	const languageOptions = computed(() =>
		uniqueLanguages.value.map((lang) => ({
			label: lang.toUpperCase(),
			value: lang
		}))
	);

	const statusOptions = computed(() =>
		uniqueStatuses.value.map((status) => ({
			label: status.charAt(0).toUpperCase() + status.slice(1),
			value: status
		}))
	);

	async function selectDirectory() {
		const selected = await useTauriDialogOpen({
			directory: true,
			multiple: false
		});

		if (selected) {
			await loadTrails(selected as string);
		}
	}

	async function reloadTrails() {
		if (directory.value) {
			await loadTrails(directory.value);
		}
	}

	async function handleDeleteTrail(trailId: string) {
		try {
			// Find the trail to get file path
			const trail = trails.value.find((t) => t.id === trailId);
			if (!trail) {
				throw new Error("Trail not found");
			}

			console.log("[index] Deleting trail:", {
				id: trailId,
				title: trail.title,
				file_path: trail.file_path
			});

			// Delete from filesystem
			await invoke("delete_trail", { filePath: trail.file_path });

			// Remove from recent trails localStorage
			removeRecentTrail(trailId);

			// Remove from bookmarks if bookmarked
			if (bookmarkedTrailIds.value.has(trailId)) {
				removeBookmark(trailId);
			}

			// Remove from displayed list
			trails.value = trails.value.filter((t) => t.id !== trailId);

			// Show success toast
			toast.add({
				title: "Trail Deleted",
				description: `"${trail.title}" has been deleted successfully`,
				color: "green"
			});

			console.log("[index] Trail deleted successfully:", trailId);
		} catch (error) {
			console.error("[index] Failed to delete trail:", error);
			toast.add({
				title: "Delete Failed",
				description: error instanceof Error ? error.message : "Unknown error occurred",
				color: "red"
			});
		}
	}

	// Load saved directory and bookmarks on mount
	onMounted(async () => {
		loadSavedDirectory();
		await loadBookmarks();
	});
</script>
