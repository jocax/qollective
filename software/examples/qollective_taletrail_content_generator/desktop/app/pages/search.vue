<template>
	<UContainer class="relative overflow-hidden h-screen">
		<div class="flex flex-col h-full p-6">
			<!-- Header Section -->
			<div class="mb-6">
				<h1 class="text-3xl font-bold font-heading mb-2">
					Search - Execution History
				</h1>
				<p class="text-gray-600 dark:text-gray-400">
					Browse and inspect MCP request/response execution history
				</p>
			</div>

			<!-- Search and Directory Info Section -->
			<div class="mb-6 space-y-3">
				<!-- Full Text Search Input -->
				<UInput
					v-model="searchQuery"
					placeholder="Search execution history..."
					icon="i-heroicons-magnifying-glass"
					size="lg"
					class="w-full"
				/>

				<!-- Root Directory Display -->
				<div class="flex items-center gap-2 text-xs bg-gray-100 dark:bg-gray-800 p-2 rounded">
					<UIcon name="i-heroicons-folder-open" class="text-gray-500" />
					<span class="font-medium text-gray-700 dark:text-gray-300">Root Directory:</span>
					<span class="font-mono text-gray-600 dark:text-gray-400 flex-1 truncate" :title="rootDirectory">
						{{ rootDirectory }}
					</span>
				</div>
			</div>

			<!-- Split Layout: Left Panel (Execution Tree) + Right Panel (Request/Response Viewer) -->
			<div class="flex-1 overflow-hidden flex gap-4">
				<!-- Left Panel: Execution Directories Tree (30% width) -->
				<div class="w-[30%] flex flex-col overflow-hidden">
					<h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-3">
						Execution Directories
						<UBadge color="blue" variant="subtle" class="ml-2">
							{{ executionDirs.length }}
						</UBadge>
					</h3>

					<!-- Loading State -->
					<div v-if="loading" class="flex items-center justify-center p-8">
						<div class="text-center">
							<div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-primary-500 mb-3" />
							<p class="text-sm text-gray-600 dark:text-gray-400">
								Loading execution directories...
							</p>
						</div>
					</div>

					<!-- Execution Tree Component -->
					<SearchExecutionTree
						v-else
						:directories="executionDirs"
						:selected-request-id="selectedRequestId"
						:selected-server="selectedServer"
						@select="handleFileSelect"
					/>
				</div>

				<!-- Right Panel: Request/Response Viewer (70% width) -->
				<div class="w-[70%] flex flex-col overflow-hidden">
					<h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-3">
						Request / Response Viewer
					</h3>

					<SearchRequestResponseViewer
						:request-content="requestContent"
						:response-content="responseContent"
						:loading="fileLoading"
						@copy-request="handleCopyRequest"
						@copy-response="handleCopyResponse"
					/>
				</div>
			</div>
		</div>
	</UContainer>
</template>

<script lang="ts" setup>
	import type { ExecutionDirectory } from "~/types/search";
	// import { invoke } from "@tauri-apps/api/core";

	definePageMeta({
		layout: "default",
		name: "Search",
		description: "Browse execution history",
		icon: "i-heroicons-magnifying-glass",
		category: "monitoring",
		showInNav: true
	});

	const toast = useToast();

	// State
	const searchQuery = ref<string>("");
	const executionDirs = ref<ExecutionDirectory[]>([]);
	const selectedRequestId = ref<string | null>(null);
	const selectedServer = ref<string | null>(null);
	const requestContent = ref<string>("");
	const responseContent = ref<string>("");
	const loading = ref<boolean>(false);
	const fileLoading = ref<boolean>(false);
	const rootDirectory = ref<string>("/path/to/execution");

	// TODO: Placeholder data for UI development
	// This will be replaced with real backend integration
	const mockExecutionDirs: ExecutionDirectory[] = [
		{
			requestId: "req-abc123-2025-01-15",
			servers: ["orchestrator", "story-generator", "quality-control"],
			timestamp: "2025-01-15T10:30:00Z"
		},
		{
			requestId: "req-def456-2025-01-15",
			servers: ["prompt-helper", "orchestrator"],
			timestamp: "2025-01-15T11:45:00Z"
		},
		{
			requestId: "req-ghi789-2025-01-14",
			servers: ["orchestrator", "story-generator", "constraint-enforcer"],
			timestamp: "2025-01-14T16:20:00Z"
		}
	];

	const mockRequestJson = {
		tool_name: "generate_story",
		arguments: {
			theme: "sci-fi",
			length: "short",
			age_group: "9-11"
		},
		timeout: 180
	};

	const mockResponseJson = {
		status: "success",
		request_id: "abc123",
		data: {
			story: "Once upon a time in a galaxy far away...",
			nodes: [
				{ id: "node-1", content: "Beginning" },
				{ id: "node-2", content: "Middle" },
				{ id: "node-3", content: "End" }
			]
		}
	};

	// Methods

	/**
	 * Load execution directories from backend
	 * TODO: Integrate backend - call list_execution_directories()
	 */
	async function loadExecutionDirectories() {
		loading.value = true;
		try {
			// TODO: Integrate backend
			// const dirs = await invoke<string[]>("list_execution_directories");
			// Process dirs and populate executionDirs with server info

			// For now, use mock data
			await new Promise((resolve) => setTimeout(resolve, 500)); // Simulate loading
			executionDirs.value = mockExecutionDirs;
		} catch (error) {
			console.error("[Search] Failed to load execution directories:", error);
			toast.add({
				title: "Load Failed",
				description: error instanceof Error ? error.message : "Failed to load execution directories",
				color: "red"
			});
		} finally {
			loading.value = false;
		}
	}

	/**
	 * Load execution file (request or response)
	 * TODO: Integrate backend - call load_execution_file(requestId, server, fileType)
	 */
	async function loadExecutionFile(_requestId: string, _server: string) {
		fileLoading.value = true;
		try {
			// TODO: Integrate backend
			// const requestData = await invoke<string>("load_execution_file", {
			//   requestId,
			//   server,
			//   fileType: "request"
			// });
			// const responseData = await invoke<string>("load_execution_file", {
			//   requestId,
			//   server,
			//   fileType: "response"
			// });

			// For now, use mock data
			await new Promise((resolve) => setTimeout(resolve, 300)); // Simulate loading
			requestContent.value = formatJson(JSON.stringify(mockRequestJson));
			responseContent.value = formatJson(JSON.stringify(mockResponseJson));
		} catch (error) {
			console.error("[Search] Failed to load execution file:", error);
			toast.add({
				title: "Load Failed",
				description: error instanceof Error ? error.message : "Failed to load execution file",
				color: "red"
			});
			requestContent.value = "";
			responseContent.value = "";
		} finally {
			fileLoading.value = false;
		}
	}

	/**
	 * Handle file selection from tree
	 */
	function handleFileSelect(requestId: string, server: string) {
		selectedRequestId.value = requestId;
		selectedServer.value = server;
		loadExecutionFile(requestId, server);
	}

	/**
	 * Copy request JSON to clipboard
	 */
	async function handleCopyRequest() {
		try {
			await navigator.clipboard.writeText(requestContent.value);
			toast.add({
				title: "Copied",
				description: "Request JSON copied to clipboard",
				color: "green"
			});
		} catch (error) {
			console.error("[Search] Failed to copy request:", error);
			toast.add({
				title: "Copy Failed",
				description: "Failed to copy request to clipboard",
				color: "red"
			});
		}
	}

	/**
	 * Copy response JSON to clipboard
	 */
	async function handleCopyResponse() {
		try {
			await navigator.clipboard.writeText(responseContent.value);
			toast.add({
				title: "Copied",
				description: "Response JSON copied to clipboard",
				color: "green"
			});
		} catch (error) {
			console.error("[Search] Failed to copy response:", error);
			toast.add({
				title: "Copy Failed",
				description: "Failed to copy response to clipboard",
				color: "red"
			});
		}
	}

	/**
	 * Pretty-print JSON with indentation
	 */
	function formatJson(jsonString: string): string {
		try {
			const parsed = JSON.parse(jsonString);
			return JSON.stringify(parsed, null, 2);
		} catch {
			return jsonString;
		}
	}

	/**
	 * Load root directory from settings
	 * TODO: Integrate with settings/preferences system
	 */
	function loadRootDirectory() {
		// TODO: Integrate backend - get from settings
		// For now, use mock value
		rootDirectory.value = "/Users/data/taletrail/execution";
	}

	// Lifecycle
	onMounted(async () => {
		loadRootDirectory();
		await loadExecutionDirectories();
	});
</script>

<style scoped>
/* Custom scrollbar styling */
.overflow-y-auto {
	scrollbar-width: thin;
	scrollbar-color: rgba(156, 163, 175, 0.5) transparent;
}

.overflow-y-auto::-webkit-scrollbar {
	width: 8px;
}

.overflow-y-auto::-webkit-scrollbar-track {
	background: transparent;
}

.overflow-y-auto::-webkit-scrollbar-thumb {
	background-color: rgba(156, 163, 175, 0.5);
	border-radius: 4px;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover {
	background-color: rgba(156, 163, 175, 0.7);
}
</style>
