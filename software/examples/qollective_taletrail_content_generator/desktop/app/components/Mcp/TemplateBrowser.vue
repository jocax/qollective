<template>
	<UCard class="h-full flex flex-col">
		<template #header>
			<h3 class="font-semibold">
				Select Template
			</h3>
		</template>

		<!-- Server Context Info -->
		<div class="mb-3 text-xs text-gray-600 dark:text-gray-400 bg-gray-100 dark:bg-gray-800 p-2 rounded">
			<div><strong>Server:</strong> {{ server }}</div>
			<div class="text-gray-500 dark:text-gray-500 mt-0.5">
				<strong>Directory:</strong> taletrail-data/templates/{{ server }}/
			</div>
		</div>

		<!-- File Picker Button -->
		<div class="mb-4">
			<UButton
				icon="i-heroicons-folder-open"
				size="lg"
				@click="openFilePicker"
			>
				Choose Template File
			</UButton>
		</div>

		<!-- Template Initialization Section (only shown when no templates exist) -->
		<div v-if="!hasTemplates" class="mb-4 p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg border border-blue-200 dark:border-blue-800">
			<div class="flex items-start gap-3">
				<UIcon name="i-heroicons-information-circle" class="text-blue-500 text-xl flex-shrink-0 mt-0.5" />
				<div class="flex-1">
					<h4 class="text-sm font-semibold text-blue-900 dark:text-blue-100 mb-1">
						First Time Setup
					</h4>
					<p class="text-xs text-blue-700 dark:text-blue-300 mb-2">
						If you don't see any templates, initialize example templates from the application source.
					</p>
					<UButton
						size="sm"
						color="blue"
						variant="soft"
						icon="i-heroicons-arrow-down-tray"
						:loading="initializingTemplates"
						@click="initializeTemplates"
					>
						Initialize Example Templates
					</UButton>
				</div>
			</div>

			<!-- Success Message -->
			<div v-if="initMessage" class="mt-3 p-2 bg-green-50 dark:bg-green-900/20 rounded border border-green-200 dark:border-green-800">
				<p class="text-xs text-green-700 dark:text-green-300">
					✓ {{ initMessage }}
				</p>
			</div>
		</div>

		<!-- Loading State -->
		<div v-if="loading" class="flex items-center justify-center py-8">
			<UIcon name="i-heroicons-arrow-path" class="animate-spin text-gray-400 text-2xl" />
		</div>

		<!-- Error State -->
		<UAlert
			v-else-if="error"
			color="red"
			variant="soft"
			icon="i-heroicons-exclamation-triangle"
			:title="error"
			class="mb-4"
		>
			<template #description>
				<div class="text-xs mt-1">
					<p>Try initializing example templates using the button above, or select a template file manually.</p>
				</div>
			</template>
		</UAlert>

		<!-- Show selected file details if available -->
		<div v-else-if="selectedTemplate" class="flex-1 p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
			<div class="space-y-3">
				<div>
					<div class="text-xs text-gray-500 dark:text-gray-400 mb-1">
						Selected Template:
					</div>
					<div class="font-semibold text-sm">
						{{ selectedTemplate.tool_name }}
					</div>
				</div>

				<div>
					<div class="text-xs text-gray-500 dark:text-gray-400 mb-1">
						File Path:
					</div>
					<div class="font-mono text-xs break-all text-gray-700 dark:text-gray-300">
						{{ selectedTemplate.file_path }}
					</div>
				</div>

				<div v-if="selectedTemplate.description">
					<div class="text-xs text-gray-500 dark:text-gray-400 mb-1">
						Description:
					</div>
					<div class="text-sm text-gray-600 dark:text-gray-400">
						{{ selectedTemplate.description }}
					</div>
				</div>
			</div>
		</div>

		<!-- Empty State -->
		<div v-else class="flex-1 flex items-center justify-center text-center p-8">
			<div>
				<UIcon name="i-heroicons-document-text" class="text-gray-300 text-4xl mb-2" />
				<p class="text-sm text-gray-500">
					No template selected
				</p>
			</div>
		</div>
	</UCard>
</template>

<script lang="ts" setup>
	import type { ServerName, TemplateData, TemplateInfo } from "@/types/mcp";
	import { invoke } from "@tauri-apps/api/core";
	import { open } from "@tauri-apps/plugin-dialog";
	import { ref, watch } from "vue";
	import { useMcpTesterStore } from "@/stores/mcpTester";

	const props = defineProps<{
		server: ServerName
	}>();

	const emit = defineEmits<{
		select: [template: TemplateData]
	}>();

	const mcpStore = useMcpTesterStore();

	// State
	const selectedTemplate = ref<(TemplateInfo & { file_path: string }) | null>(null);
	const loading = ref(false);
	const error = ref<string | null>(null);
	const initializingTemplates = ref(false);
	const initMessage = ref<string | null>(null);
	const hasTemplates = ref(false);

	// Function to check if templates exist for current server
	async function checkTemplatesExist() {
		try {
			// Call existing backend command
			const grouped = await invoke<Record<string, any[]>>("list_mcp_templates");

			// Check if current server has any templates
			const serverTemplates = grouped[props.server] || [];
			hasTemplates.value = serverTemplates.length > 0;

			console.log(`[TemplateBrowser] Templates for ${props.server}:`, serverTemplates.length, "found");
		} catch (e) {
			console.error("[TemplateBrowser] Failed to check templates:", e);
			hasTemplates.value = false;  // Show init section if we can't check
		}
	}

	// Watch for server changes and reset component state
	watch(() => props.server, async (newServer, oldServer) => {
		if (newServer !== oldServer) {
			// Clear selected template when switching servers
			selectedTemplate.value = null;
			error.value = null;
			initMessage.value = null;

			console.log(`[TemplateBrowser] Server changed: ${oldServer} → ${newServer}`);

			// Check if new server has templates
			await checkTemplatesExist();
		}
	}, { immediate: true });  // Run on mount

	async function openFilePicker() {
		try {
			error.value = null;

			// Get the templates directory for the selected server
			let defaultPath: string | undefined;
			try {
				console.log("[TemplateBrowser] Opening file picker for server:", props.server);
				defaultPath = await invoke<string>("get_templates_directory", {
					server: props.server
				});
				console.log("[TemplateBrowser] Templates directory:", defaultPath);
			} catch (e) {
				console.warn("[TemplateBrowser] Failed to get templates directory:", e);
				// Show a hint to the user
				error.value = `Template directory may not exist yet. Try initializing templates first.`;
				// Continue without default path
			}

			// Open file dialog - filter for JSON files
			const selected = await open({
				multiple: false,
				defaultPath,
				filters: [
					{
						name: "JSON Templates",
						extensions: ["json"]
					}
				]
			});

			if (!selected) {
				return; // User cancelled
			}

			loading.value = true;

			// Load template data from selected file
			const templateData = await invoke<TemplateData>("load_mcp_template", {
				templatePath: selected
			});

			// Create template info object
			const templateInfo: TemplateInfo & { file_path: string } = {
				file_name: selected.split("/").pop() || "",
				file_path: selected,
				tool_name: templateData.tool_name,
				description: `Tool: ${templateData.tool_name}`
			};

			selectedTemplate.value = templateInfo;

			// Update store
			mcpStore.setTemplateContent(templateData);

			// Emit to parent
			emit("select", templateData);

			console.log("[TemplateBrowser] Template selected and loaded:", templateData.tool_name);
		} catch (e: any) {
			error.value = `Failed to load template: ${e}`;
			console.error("Failed to load template:", e);
		} finally {
			loading.value = false;
		}
	}

	async function initializeTemplates() {
		try {
			initializingTemplates.value = true;
			initMessage.value = null;
			error.value = null;

			console.log("[TemplateBrowser] Initializing templates...");

			// Call new Tauri command to initialize templates
			const result = await invoke<string>("initialize_templates");

			initMessage.value = result;
			console.log("[TemplateBrowser] Templates initialized:", result);

			// Recheck if templates now exist
			await checkTemplatesExist();

			// Show success message briefly
			setTimeout(() => {
				initMessage.value = null;
			}, 5000);
		} catch (e: any) {
			error.value = `Failed to initialize templates: ${e}`;
			console.error("[TemplateBrowser] Template initialization failed:", e);
		} finally {
			initializingTemplates.value = false;
		}
	}
</script>
