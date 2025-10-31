<template>
	<Html class="overflow-x-hidden">
		<Body class="font-sans antialiased">
			<UApp>
				<NuxtLayout>
					<NuxtPage />
				</NuxtLayout>
			</UApp>
		</Body>
	</Html>
</template>

<script setup lang="ts">
	const { loadTheme } = useTheme();

	onMounted(() => {
		loadTheme();
	});
</script>
