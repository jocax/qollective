import { invoke } from "@tauri-apps/api/core";
import { computed, ref } from "vue";

export interface Bookmark {
	trail_id: string
	trail_title: string
	file_path: string
	timestamp: string
	user_note: string
	tenantId?: string // Multi-tenant support
}

export interface TrailListItem {
	id: string
	title: string
	file_path: string
	description?: string
	genre?: string
	difficulty?: string
	estimated_duration?: number
	created_at?: string
	updated_at?: string
	node_count?: number
	choice_count?: number
	tenantId?: string // Multi-tenant support
}

export function useBookmarks() {
	const bookmarks = ref<Bookmark[]>([]);
	const loading = ref(false);
	const error = ref<string | null>(null);

	// Load bookmarks from store with optional tenant filtering
	async function loadBookmarks(tenantId: string | null = null) {
		try {
			loading.value = true;
			error.value = null;
			// Pass tenantId to the Tauri command (null means all tenants)
			bookmarks.value = await invoke<Bookmark[]>("get_bookmarks", {
				app: "taletrail",
				tenantId
			});
		} catch (e) {
			error.value = (e as Error).message;
			console.error("Failed to load bookmarks:", e);
		} finally {
			loading.value = false;
		}
	}

	// Add bookmark with tenant context
	async function addBookmark(trail: TrailListItem, note?: string, tenantId: string | null = null) {
		try {
			const bookmark: Bookmark = {
				trail_id: trail.id,
				trail_title: trail.title,
				file_path: trail.file_path,
				timestamp: new Date().toISOString(),
				user_note: note || "",
				tenantId: tenantId || trail.tenantId
			};

			bookmarks.value = await invoke<Bookmark[]>("add_bookmark", {
				app: "taletrail",
				bookmark,
				tenantId: tenantId || trail.tenantId || null
			});
		} catch (e) {
			error.value = (e as Error).message;
			console.error("Failed to add bookmark:", e);
			throw e;
		}
	}

	// Remove bookmark with tenant context
	async function removeBookmark(trailId: string, tenantId: string | null = null) {
		try {
			bookmarks.value = await invoke<Bookmark[]>("remove_bookmark", {
				app: "taletrail",
				trailId,
				tenantId
			});
		} catch (e) {
			error.value = (e as Error).message;
			console.error("Failed to remove bookmark:", e);
			throw e;
		}
	}

	// Check if trail is bookmarked
	function isBookmarked(trailId: string): boolean {
		return bookmarks.value.some((b) => b.trail_id === trailId);
	}

	// Get bookmark for trail
	function getBookmark(trailId: string): Bookmark | undefined {
		return bookmarks.value.find((b) => b.trail_id === trailId);
	}

	// Get bookmarked trail IDs
	const bookmarkedTrailIds = computed(() => {
		return new Set(bookmarks.value.map((b) => b.trail_id));
	});

	return {
		bookmarks,
		loading,
		error,
		loadBookmarks,
		addBookmark,
		removeBookmark,
		isBookmarked,
		getBookmark,
		bookmarkedTrailIds
	};
}
