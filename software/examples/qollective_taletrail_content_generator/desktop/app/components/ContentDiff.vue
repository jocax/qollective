<template>
	<div class="space-y-4">
		<!-- Legend -->
		<UCard v-if="showLegend">
			<div class="flex items-center justify-between">
				<h3 class="text-lg font-semibold">
					Content Differences
				</h3>
				<div class="flex items-center gap-4 text-sm">
					<div class="flex items-center gap-2">
						<div class="w-3 h-3 bg-green-200 dark:bg-green-900 rounded" />
						<span>Added ({{ stats.added }})</span>
					</div>
					<div class="flex items-center gap-2">
						<div class="w-3 h-3 bg-red-200 dark:bg-red-900 rounded" />
						<span>Removed ({{ stats.removed }})</span>
					</div>
					<div class="flex items-center gap-2">
						<div class="w-3 h-3 bg-yellow-200 dark:bg-yellow-900 rounded" />
						<span>Modified ({{ stats.modified }})</span>
					</div>
					<div class="flex items-center gap-2">
						<div class="w-3 h-3 bg-gray-200 dark:bg-gray-700 rounded" />
						<span>Unchanged ({{ stats.unchanged }})</span>
					</div>
				</div>
			</div>
		</UCard>

		<!-- Diff Results -->
		<div class="space-y-3">
			<UCard
				v-for="result in diffResults"
				:key="result.nodeId"
				:class="{
					'border-l-4 border-green-500': result.status === 'added',
					'border-l-4 border-red-500': result.status === 'removed',
					'border-l-4 border-yellow-500': result.status === 'modified',
					'opacity-50': result.status === 'unchanged'
				}"
			>
				<!-- Node Header -->
				<div class="flex items-center justify-between mb-3">
					<div class="flex items-center gap-2">
						<code class="text-xs px-2 py-1 bg-gray-100 dark:bg-gray-800 rounded">
							{{ result.nodeId }}
						</code>
						<UBadge
							:color="result.status === 'added' ? 'green' : result.status === 'removed' ? 'red' : result.status === 'modified' ? 'yellow' : 'gray'"
							variant="subtle"
						>
							{{ result.status }}
						</UBadge>
					</div>
				</div>

				<!-- Added Node -->
				<div v-if="result.status === 'added' && result.newNode" class="space-y-2">
					<div class="p-3 bg-green-50 dark:bg-green-950 rounded">
						<p class="text-sm">
							{{ result.newNode.content.text }}
						</p>
					</div>
					<div v-if="result.newNode.content.choices && result.newNode.content.choices.length > 0" class="text-xs">
						<p class="font-semibold mb-1">
							Choices:
						</p>
						<ul class="list-disc list-inside space-y-1 text-green-700 dark:text-green-300">
							<li v-for="choice in result.newNode.content.choices" :key="choice.id">
								{{ choice.text }}
							</li>
						</ul>
					</div>
				</div>

				<!-- Removed Node -->
				<div v-else-if="result.status === 'removed' && result.originalNode" class="space-y-2">
					<div class="p-3 bg-red-50 dark:bg-red-950 rounded">
						<p class="text-sm line-through">
							{{ result.originalNode.content.text }}
						</p>
					</div>
					<div v-if="result.originalNode.content.choices && result.originalNode.content.choices.length > 0" class="text-xs">
						<p class="font-semibold mb-1">
							Choices:
						</p>
						<ul class="list-disc list-inside space-y-1 text-red-700 dark:text-red-300 line-through">
							<li v-for="choice in result.originalNode.content.choices" :key="choice.id">
								{{ choice.text }}
							</li>
						</ul>
					</div>
				</div>

				<!-- Modified Node -->
				<div v-else-if="result.status === 'modified'" class="space-y-3">
					<!-- Text Diff -->
					<div v-if="result.textDiff" class="p-3 bg-yellow-50 dark:bg-yellow-950 rounded">
						<p class="text-sm leading-relaxed">
							<span
								v-for="(chunk, idx) in result.textDiff"
								:key="idx"
								:class="{
									'bg-green-200 dark:bg-green-800': chunk.type === 'added',
									'bg-red-200 dark:bg-red-800 line-through': chunk.type === 'removed'
								}"
							>{{ chunk.text }}{{ idx < result.textDiff!.length - 1 ? ' ' : '' }}</span>
						</p>
					</div>

					<!-- Choices Diff -->
					<div v-if="result.choicesDiff" class="text-xs space-y-2">
						<p class="font-semibold">
							Choice Changes:
						</p>

						<div v-if="result.choicesDiff.added.length > 0">
							<p class="text-green-700 dark:text-green-300 font-medium">
								Added:
							</p>
							<ul class="list-disc list-inside space-y-1 text-green-600 dark:text-green-400">
								<li v-for="(choice, idx) in result.choicesDiff.added" :key="idx">
									{{ choice }}
								</li>
							</ul>
						</div>

						<div v-if="result.choicesDiff.removed.length > 0">
							<p class="text-red-700 dark:text-red-300 font-medium">
								Removed:
							</p>
							<ul class="list-disc list-inside space-y-1 text-red-600 dark:text-red-400">
								<li v-for="(choice, idx) in result.choicesDiff.removed" :key="idx">
									{{ choice }}
								</li>
							</ul>
						</div>

						<div v-if="result.choicesDiff.modified.length > 0">
							<p class="text-yellow-700 dark:text-yellow-300 font-medium">
								Modified:
							</p>
							<ul class="list-disc list-inside space-y-1 text-yellow-600 dark:text-yellow-400">
								<li v-for="(choice, idx) in result.choicesDiff.modified" :key="idx">
									{{ choice }}
								</li>
							</ul>
						</div>
					</div>
				</div>

				<!-- Unchanged Node (collapsed) -->
				<div v-else-if="result.status === 'unchanged' && result.newNode" class="text-sm text-gray-500">
					{{ result.newNode.content.text.substring(0, 100) }}{{ result.newNode.content.text.length > 100 ? '...' : '' }}
				</div>
			</UCard>
		</div>
	</div>
</template>

<script setup lang="ts">
	import type { ContentNode } from "~/types/trails";

	interface Props {
		originalNodes: Record<string, ContentNode>
		newNodes: Record<string, ContentNode>
		showLegend?: boolean
	}

	const props = withDefaults(defineProps<Props>(), {
		showLegend: true
	});

	interface DiffResult {
		nodeId: string
		status: "added" | "removed" | "modified" | "unchanged"
		originalNode?: ContentNode
		newNode?: ContentNode
		textDiff?: Array<{ type: "added" | "removed" | "unchanged", text: string }>
		choicesDiff?: {
			added: string[]
			removed: string[]
			modified: string[]
		}
	}

	/**
	 * Simple text diff algorithm
	 * Returns array of chunks with type (added/removed/unchanged)
	 */
	function diffText(text1: string, text2: string): Array<{ type: "added" | "removed" | "unchanged", text: string }> {
		if (text1 === text2) {
			return [{ type: "unchanged", text: text1 }];
		}

		// Simple word-based diff
		const words1 = text1.split(/\s+/);
		const words2 = text2.split(/\s+/);
		const result: Array<{ type: "added" | "removed" | "unchanged", text: string }> = [];

		// Very basic LCS-style comparison
		let i = 0;
		let j = 0;

		while (i < words1.length || j < words2.length) {
			if (i >= words1.length) {
				// Only words in text2 remain (added)
				result.push({ type: "added", text: words2.slice(j).join(" ") });
				break;
			}
			if (j >= words2.length) {
				// Only words in text1 remain (removed)
				result.push({ type: "removed", text: words1.slice(i).join(" ") });
				break;
			}

			if (words1[i] === words2[j]) {
				// Words match
				result.push({ type: "unchanged", text: words1[i] });
				i++;
				j++;
			} else {
				// Words differ - try to find next match
				const nextMatchInText2 = words2.indexOf(words1[i], j);
				const nextMatchInText1 = words1.indexOf(words2[j], i);

				if (nextMatchInText2 !== -1 && (nextMatchInText1 === -1 || nextMatchInText2 - j < nextMatchInText1 - i)) {
					// Found match in text2 sooner - words in text2 are added
					result.push({ type: "added", text: words2.slice(j, nextMatchInText2).join(" ") });
					j = nextMatchInText2;
				} else if (nextMatchInText1 !== -1) {
					// Found match in text1 - words in text1 are removed
					result.push({ type: "removed", text: words1.slice(i, nextMatchInText1).join(" ") });
					i = nextMatchInText1;
				} else {
					// No match found - mark as removed and added
					result.push({ type: "removed", text: words1[i] });
					result.push({ type: "added", text: words2[j] });
					i++;
					j++;
				}
			}
		}

		return result;
	}

	/**
	 * Compare nodes and generate diff results
	 */
	const diffResults = computed<DiffResult[]>(() => {
		const results: DiffResult[] = [];
		const originalIds = new Set(Object.keys(props.originalNodes));
		const newIds = new Set(Object.keys(props.newNodes));

		// Check for added, removed, and modified nodes
		const allIds = new Set([...originalIds, ...newIds]);

		for (const nodeId of allIds) {
			const originalNode = props.originalNodes[nodeId];
			const newNode = props.newNodes[nodeId];

			if (!originalNode && newNode) {
				// Added node
				results.push({
					nodeId,
					status: "added",
					newNode
				});
			} else if (originalNode && !newNode) {
				// Removed node
				results.push({
					nodeId,
					status: "removed",
					originalNode
				});
			} else if (originalNode && newNode) {
				// Check if modified
				const textChanged = originalNode.content.text !== newNode.content.text;
				const choicesChanged = JSON.stringify(originalNode.content.choices) !== JSON.stringify(newNode.content.choices);

				if (textChanged || choicesChanged) {
					// Modified node
					const textDiff = textChanged ? diffText(originalNode.content.text || "", newNode.content.text || "") : undefined;

					const choicesDiff = choicesChanged
						? {
							added: (newNode.content.choices || [])
								.filter((c) => !(originalNode.content.choices || []).find((oc) => oc.id === c.id))
								.map((c) => c.text || ""),
							removed: (originalNode.content.choices || [])
								.filter((c) => !(newNode.content.choices || []).find((nc) => nc.id === c.id))
								.map((c) => c.text || ""),
							modified: (newNode.content.choices || [])
								.filter((c) => {
									const orig = (originalNode.content.choices || []).find((oc) => oc.id === c.id);
									return orig && orig.text !== c.text;
								})
								.map((c) => c.text || "")
						}
						: undefined;

					results.push({
						nodeId,
						status: "modified",
						originalNode,
						newNode,
						textDiff,
						choicesDiff
					});
				} else {
					// Unchanged
					results.push({
						nodeId,
						status: "unchanged",
						originalNode,
						newNode
					});
				}
			}
		}

		return results.sort((a, b) => {
			// Sort by status priority: removed, modified, added, unchanged
			const priority = { removed: 0, modified: 1, added: 2, unchanged: 3 };
			return priority[a.status] - priority[b.status];
		});
	});

	/**
	 * Statistics
	 */
	const stats = computed(() => ({
		added: diffResults.value.filter((r) => r.status === "added").length,
		removed: diffResults.value.filter((r) => r.status === "removed").length,
		modified: diffResults.value.filter((r) => r.status === "modified").length,
		unchanged: diffResults.value.filter((r) => r.status === "unchanged").length,
		total: diffResults.value.length
	}));
</script>
