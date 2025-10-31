<template>
	<UCard class="h-full flex flex-col">
		<template #header>
			<h3 class="font-semibold">
				Select Template
			</h3>
		</template>

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

		<!-- Loading State -->
		<div v-if="loading" class="flex items-center justify-center py-8">
			<UIcon name="i-heroicons-arrow-path" class="animate-spin text-gray-400 text-2xl" />
		</div>

		<!-- Error State -->
		<div v-else-if="error" class="text-sm text-red-500">
			{{ error }}
		</div>

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
	import { ref } from "vue";
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

	async function openFilePicker() {
		try {
			error.value = null;

			// Get the templates directory for the selected server
			let defaultPath: string | undefined;
			try {
				defaultPath = await invoke<string>("get_templates_directory", {
					server: props.server
				});
				console.log("[TemplateBrowser] Templates directory:", defaultPath);
			} catch (e) {
				console.warn("[TemplateBrowser] Failed to get templates directory, using default:", e);
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
		} catch (e: any) {
			error.value = `Failed to load template: ${e}`;
			console.error("Failed to load template:", e);
		} finally {
			loading.value = false;
		}
	}
</script>
