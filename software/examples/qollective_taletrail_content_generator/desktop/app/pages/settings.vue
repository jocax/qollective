<template>
	<UContainer class="relative overflow-hidden h-screen">
		<div class="flex flex-col h-full p-6">
			<div class="mb-6">
				<h1 class="text-3xl font-bold font-heading mb-2">
					Settings
				</h1>
				<p class="text-gray-600 dark:text-gray-400">
					Configure your TaleTrail viewer preferences
				</p>
			</div>

			<!-- Loading state -->
			<LoadingState
				v-if="loading"
				message="Loading settings..."
			/>

			<!-- Settings form with tabs -->
			<div v-else class="flex-1 overflow-auto">
				<UTabs v-model="activeTab" :items="settingsTabs" class="max-w-4xl">
					<!-- Overview Tab -->
					<template #overview>
						<div class="mt-4 space-y-3">
							<!-- Directories - Inline layout -->
							<div class="space-y-2 p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
								<h3 class="text-xs font-semibold mb-2 text-gray-700 dark:text-gray-300">
									Data Directories
								</h3>
								<div class="flex items-center gap-2">
									<label class="text-xs font-medium w-28 flex-shrink-0">Root Directory:</label>
									<UInput v-model="preferences.root_directory" size="xs" class="flex-1" placeholder="taletrail-data" />
									<UButton icon="i-heroicons-folder-open" size="xs" @click="selectRootDirectory">
										Browse
									</UButton>
								</div>
								<div class="flex items-center gap-2">
									<label class="text-xs font-medium w-28 flex-shrink-0">Templates:</label>
									<UInput :model-value="preferences.root_directory ? `${preferences.root_directory}/templates/` : ''" readonly disabled size="xs" class="flex-1" />
								</div>
								<div class="flex items-center gap-2">
									<label class="text-xs font-medium w-28 flex-shrink-0">Execution:</label>
									<UInput :model-value="preferences.root_directory ? `${preferences.root_directory}/execution/` : ''" readonly disabled size="xs" class="flex-1" />
								</div>
							</div>

							<!-- Legacy Trails Directory - Inline layout -->
							<div class="space-y-2 p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
								<h3 class="text-xs font-semibold mb-2 text-gray-700 dark:text-gray-300">
									Legacy Trails Directory
								</h3>
								<div class="flex items-center gap-2">
									<label class="text-xs font-medium w-28 flex-shrink-0">Trails Directory:</label>
									<UInput v-model="preferences.directory_path" size="xs" class="flex-1" placeholder="/path/to/trails" />
									<UButton icon="i-heroicons-folder-open" size="xs" @click="selectDirectory">
										Browse
									</UButton>
								</div>
							</div>

							<!-- NATS Connection - Inline layout -->
							<div class="space-y-2 p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
								<div class="flex items-center gap-2">
									<label class="text-xs font-medium w-28 flex-shrink-0">NATS URL:</label>
									<UInput :model-value="NETWORK.NATS_URL" readonly disabled size="xs" class="flex-1" />
								</div>
								<div class="flex items-center gap-2">
									<label class="text-xs font-medium w-28 flex-shrink-0">Status:</label>
									<div class="flex items-center gap-2">
										<div class="w-2 h-2 rounded-full bg-green-500" />
										<span class="text-xs text-gray-700 dark:text-gray-300">Configured</span>
									</div>
								</div>
							</div>

							<!-- MCP Components - Simple list -->
							<div class="p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
								<h3 class="text-xs font-semibold mb-2 text-gray-700 dark:text-gray-300">
									MCP Components
								</h3>
								<div class="space-y-1">
									<div
										v-for="server in MCP_SERVERS"
										:key="server"
										class="flex items-center justify-between text-xs py-1"
									>
										<span class="font-medium text-gray-700 dark:text-gray-300">{{ server }}</span>
										<UBadge color="gray" variant="subtle" size="xs">
											Configured
										</UBadge>
									</div>
								</div>
							</div>

							<!-- Actions -->
							<div class="flex gap-2">
								<UButton :loading="saving" icon="i-heroicons-check" size="sm" @click="saveSettings">
									Save Settings
								</UButton>
								<UButton variant="outline" icon="i-heroicons-arrow-path" size="sm" @click="resetSettings">
									Reset
								</UButton>
								<UButton variant="ghost" to="/" icon="i-heroicons-arrow-left" size="sm">
									Back
								</UButton>
							</div>
						</div>
					</template>

					<!-- Config Tab -->
					<template #config>
						<div class="mt-4">
							<UCard>
								<template #header>
									<div class="flex items-center justify-between">
										<div>
											<h3 class="text-sm font-semibold">
												Application Configuration
											</h3>
											<p class="text-xs text-gray-500 mt-1">
												File: src-tauri/config.toml
											</p>
										</div>
										<div class="flex items-center gap-2">
											<UButton
												icon="i-heroicons-arrow-path"
												size="xs"
												variant="ghost"
												:loading="loadingConfig"
												@click="loadConfigFile"
											>
												Refresh
											</UButton>
											<UButton
												icon="i-heroicons-clipboard-document"
												size="xs"
												variant="outline"
												@click="copyConfig"
											>
												Copy
											</UButton>
										</div>
									</div>
								</template>
								<div class="p-3">
									<!-- Loading state -->
									<div v-if="loadingConfig" class="flex items-center justify-center py-8">
										<UIcon name="i-heroicons-arrow-path" class="w-6 h-6 animate-spin text-gray-400" />
									</div>

									<!-- TOML content -->
									<pre v-else class="text-xs overflow-auto max-h-96 p-3 bg-gray-50 dark:bg-gray-900 rounded font-mono border border-gray-200 dark:border-gray-700 whitespace-pre-wrap">{{ configToml }}</pre>

									<p class="text-xs text-gray-500 mt-3 italic">
										Note: Configuration is read-only. Edit src-tauri/config.toml directly or use environment variables to override values.
									</p>
								</div>
							</UCard>

							<!-- Back button for Config tab -->
							<div class="mt-4">
								<UButton variant="ghost" to="/" icon="i-heroicons-arrow-left" size="sm">
									Back to Trails
								</UButton>
							</div>
						</div>
					</template>
				</UTabs>
			</div>
		</div>
	</UContainer>
</template>

<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";
	import { onMounted, ref } from "vue";
	import { MCP_SERVERS, NETWORK, PATHS } from "~/config/constants";

	definePageMeta({
		layout: "default",
		name: "Settings",
		description: "Configure viewer preferences",
		icon: "i-heroicons-cog-6-tooth",
		category: "other",
		showInNav: true
	});

	interface UserPreferences {
		directory_path: string
		auto_validate: boolean
		root_directory: string
	}

	const preferences = ref<UserPreferences>({
		directory_path: "",
		auto_validate: true,
		root_directory: PATHS.DEFAULT_ROOT_DIRECTORY
	});

	const loading = ref(true);
	const saving = ref(false);
	const configToml = ref("");
	const loadingConfig = ref(false);
	const { showSuccess, handleError } = useErrorHandling();

	// Tab state
	const activeTab = ref(0);
	const settingsTabs = [
		{ label: "Overview", icon: "i-heroicons-eye", slot: "overview" },
		{ label: "Config", icon: "i-heroicons-code-bracket", slot: "config" }
	];

	async function loadConfigFile() {
		loadingConfig.value = true;
		try {
			configToml.value = await invoke<string>("load_config_toml");
		} catch (e) {
			handleError(e, "Failed to load configuration file");
			configToml.value = `# Error loading config.toml\n# ${String(e)}`;
		} finally {
			loadingConfig.value = false;
		}
	}

	onMounted(async () => {
		await loadSettings();
		await loadConfigFile();
	});

	async function loadSettings() {
		try {
			loading.value = true;
			// Load preferences without tenant context for global settings
			preferences.value = await invoke<UserPreferences>("load_preferences", {
				tenantId: null
			});
			console.log("[settings] Preferences loaded:", preferences.value);
		} catch (e) {
			console.error("[settings] Failed to load preferences:", e);
			handleError(e, "Failed to load preferences");
		} finally {
			loading.value = false;
		}
	}

	async function saveSettings() {
		try {
			saving.value = true;

			// Initialize root directory structure if root_directory is set
			if (preferences.value.root_directory) {
				try {
					await invoke("initialize_root_directory", {
						path: preferences.value.root_directory
					});
					console.log("[settings] Root directory initialized:", preferences.value.root_directory);
				} catch (e) {
					console.error("[settings] Failed to initialize root directory:", e);
					handleError(e, "Failed to initialize root directory");
					return;
				}
			}

			// Save preferences without tenant context for global settings
			await invoke("save_preferences", {
				preferences: preferences.value,
				tenantId: null
			});

			console.log("[settings] Preferences saved:", preferences.value);
			showSuccess("Settings saved", "Your preferences have been updated");
		} catch (e) {
			console.error("[settings] Failed to save settings:", e);
			handleError(e, "Failed to save settings");
		} finally {
			saving.value = false;
		}
	}

	async function selectRootDirectory() {
		try {
			const selected = await useTauriDialogOpen({
				directory: true,
				multiple: false
			});

			if (selected) {
				preferences.value.root_directory = selected as string;
				console.log("[settings] Root directory selected:", selected);
			}
		} catch (e) {
			console.error("[settings] Failed to select root directory:", e);
			handleError(e, "Failed to select root directory");
		}
	}

	async function selectDirectory() {
		try {
			const selected = await useTauriDialogOpen({
				directory: true,
				multiple: false
			});

			if (selected) {
				preferences.value.directory_path = selected as string;
				console.log("[settings] Directory selected:", selected);
			}
		} catch (e) {
			console.error("[settings] Failed to select directory:", e);
			handleError(e, "Failed to select directory");
		}
	}

	function resetSettings() {
		preferences.value = {
			directory_path: "",
			auto_validate: true,
			root_directory: PATHS.DEFAULT_ROOT_DIRECTORY
		};
		showSuccess("Settings reset", "Preferences have been reset to defaults");
		console.log("[settings] Preferences reset to defaults");
	}

	async function copyConfig() {
		try {
			await navigator.clipboard.writeText(configToml.value);
			showSuccess("Copied", "Configuration copied to clipboard");
		} catch (e) {
			handleError(e, "Failed to copy configuration");
		}
	}
</script>
