<template>
	<div class="space-y-1 p-2">
		<div
			v-for="template in templates"
			:key="template.file_path"
			class="template-item"
		>
			<UButton
				variant="ghost"
				size="sm"
				class="w-full justify-start text-left"
				@click="$emit('select', template)"
			>
				<div class="flex-1 min-w-0">
					<div class="font-medium text-sm truncate">
						{{ template.tool_name }}
					</div>
					<div class="text-xs text-gray-500 truncate">
						{{ template.description }}
					</div>
				</div>
			</UButton>
		</div>

		<!-- Empty state for group -->
		<div v-if="templates.length === 0" class="text-xs text-gray-400 text-center py-2">
			No templates
		</div>
	</div>
</template>

<script lang="ts" setup>
	import type { TemplateInfo } from "@/types/mcp";

	defineProps<{
		templates: TemplateInfo[]
	}>();

	defineEmits<{
		select: [template: TemplateInfo]
	}>();
</script>

<style scoped>
.template-item:hover {
  background-color: rgba(0, 0, 0, 0.02);
}

.dark .template-item:hover {
  background-color: rgba(255, 255, 255, 0.05);
}
</style>
