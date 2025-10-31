<template>
	<UModal v-model="isOpen" title="Request Settings">
		<div class="space-y-4 p-4">
			<UFormField
				label="Timeout (seconds)"
				description="Request timeout duration"
				required
			>
				<UInput
					v-model.number="localTimeout"
					type="number"
					:min="1"
					:max="600"
				/>
			</UFormField>

			<div class="flex justify-end gap-2 pt-4">
				<UButton variant="ghost" @click="cancel">
					Cancel
				</UButton>
				<UButton @click="save">
					Save
				</UButton>
			</div>
		</div>
	</UModal>
</template>

<script lang="ts" setup>
	import { computed, ref, watch } from "vue";

	const props = defineProps<{
		open: boolean
		timeout: number
	}>();

	const emit = defineEmits<{
		"update:open": [value: boolean]
		"update:timeout": [value: number]
	}>();

	const isOpen = computed({
		get: () => props.open,
		set: (value) => emit("update:open", value)
	});

	const localTimeout = ref(props.timeout);

	watch(() => props.timeout, (timeout) => {
		localTimeout.value = timeout;
	});

	function save() {
		emit("update:timeout", localTimeout.value);
		emit("update:open", false);
	}

	function cancel() {
		localTimeout.value = props.timeout;
		emit("update:open", false);
	}
</script>
