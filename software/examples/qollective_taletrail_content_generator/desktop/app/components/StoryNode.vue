<template>
	<div class="story-node space-y-4">
		<!-- Node metadata badge (top) -->
		<div v-if="showMetadata" class="flex flex-wrap gap-2">
			<UBadge color="gray" variant="subtle">
				Node: {{ node.id }}
			</UBadge>
			<UBadge v-if="node.generation_metadata?.llm_model" color="blue" variant="subtle">
				Model: {{ node.generation_metadata.llm_model }}
			</UBadge>
			<UBadge v-if="node.generation_metadata?.timestamp" color="green" variant="subtle">
				Generated: {{ formatTimestamp(node.generation_metadata.timestamp) }}
			</UBadge>
		</div>

		<!-- Convergence indicator -->
		<UBadge v-if="isConvergencePoint" color="purple" variant="solid" size="lg">
			<div class="flex items-center gap-1">
				<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
				</svg>
				Convergence Point
			</div>
		</UBadge>

		<!-- Story text (main content) -->
		<UCard>
			<div class="story-text prose prose-lg dark:prose-invert max-w-none p-4">
				<p class="text-lg leading-relaxed whitespace-pre-wrap">
					{{ cleanedText }}
				</p>
			</div>
		</UCard>

		<!-- Educational insights (collapsible) -->
		<div v-if="showInsights && hasEducationalContent" class="mt-4">
			<UAccordion :items="accordionItems" variant="soft">
				<template #default="{ item, open }">
					<UButton
						color="primary"
						variant="ghost"
						class="w-full justify-between"
						:ui="{ rounded: 'rounded-lg' }"
					>
						<div class="flex items-center gap-2">
							<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
							</svg>
							<span class="font-medium">{{ item.label }}</span>
						</div>
						<svg
							class="w-5 h-5 transition-transform duration-200"
							:class="{ 'rotate-180': open }"
							fill="none"
							stroke="currentColor"
							viewBox="0 0 24 24"
						>
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
						</svg>
					</UButton>
				</template>
				<template #item="{ item }">
					<div class="p-4 text-gray-700 dark:text-gray-300 prose prose-sm dark:prose-invert max-w-none">
						<p class="whitespace-pre-wrap">
							{{ item.content }}
						</p>
					</div>
				</template>
			</UAccordion>
		</div>
	</div>
</template>

<script lang="ts" setup>
	import type { ContentNode } from "~/types/trails";

	interface Props {
		node: ContentNode
		showMetadata?: boolean
		showInsights?: boolean
		isConvergencePoint?: boolean
	}

	const props = withDefaults(defineProps<Props>(), {
		showMetadata: false,
		showInsights: true,
		isConvergencePoint: false
	});

	/**
	 * Clean story text by removing AI-generated placeholder patterns
	 * Filters out choice markers, formatting headers, and empty lines
	 */
	const cleanedText = computed(() => {
		const text = props.node.content.text;

		// Split into lines for processing
		const lines = text.split("\n");

		// Patterns to remove (case-insensitive)
		const removePatterns = [
			/^\*\*Choice/i, // **Choice Options:**, **Choice 1**, etc.
			/^\*\*Optional/i, // **Optional Educational Content:**
			/^\*\*Narrative/i, // **Narrative Text:**
			/^Choice\s+\d+/i, // Choice 1, Choice 2, Choice 3
			/^[A-C]\)\s+/, // A) , B) , C) choice markers
			/^\*\*Educational/i, // **Educational Note:**
			/^\s*\*\*$/ // Lines with just **
		];

		// Filter lines
		const filteredLines = lines.filter((line) => {
			const trimmedLine = line.trim();

			// Remove empty lines
			if (trimmedLine.length === 0) return false;

			// Check against all patterns
			for (const pattern of removePatterns) {
				if (pattern.test(trimmedLine)) {
					return false;
				}
			}

			return true;
		});

		// Join back with line breaks, preserving intentional paragraph breaks
		return filteredLines.join("\n").trim();
	});

	/**
	 * Extract educational content from node text
	 * Looks for patterns like "Educational Note:" or similar markers
	 */
	const educationalContent = computed(() => {
		const text = props.node.content.text;
		const educationalMarkers = [
			"Educational Note:",
			"Learning Point:",
			"Did you know:",
			"Fun Fact:",
			"Historical Context:"
		];

		for (const marker of educationalMarkers) {
			const index = text.indexOf(marker);
			if (index !== -1) {
				return text.substring(index + marker.length).trim();
			}
		}

		// If no explicit marker, check if node has separate educational metadata
		// (This would need to be added to the ContentNode type if the backend provides it)
		return null;
	});

	/**
	 * Check if node has educational content to display
	 */
	const hasEducationalContent = computed(() => {
		return educationalContent.value !== null && educationalContent.value.length > 0;
	});

	/**
	 * Accordion items for educational insights
	 */
	const accordionItems = computed(() => {
		if (!hasEducationalContent.value) return [];

		return [{
			label: "Educational Insights",
			content: educationalContent.value,
			defaultOpen: false
		}];
	});

	/**
	 * Format timestamp to human-readable format
	 */
	function formatTimestamp(timestamp: string): string {
		try {
			const date = new Date(timestamp);
			return date.toLocaleString("en-US", {
				month: "short",
				day: "numeric",
				year: "numeric",
				hour: "2-digit",
				minute: "2-digit"
			});
		} catch {
			return timestamp;
		}
	}
</script>

<style scoped>
.story-text {
  font-family: 'Georgia', 'Times New Roman', serif;
  line-height: 1.8;
}
</style>
