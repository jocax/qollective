<template>
	<div v-if="isVisible" class="fixed bottom-0 left-0 right-0 z-50 bg-gray-900/95 backdrop-blur">
		<UCard>
			<template #header>
				<div class="flex items-center justify-between">
					<h3 class="text-sm font-semibold">
						Debug Console
					</h3>
					<div class="flex items-center gap-2">
						<UBadge :label="`${logs.length} logs`" />
						<UButton size="xs" @click="copyLogs">
							Copy All
						</UButton>
						<UButton size="xs" variant="ghost" @click="clearLogs">
							Clear
						</UButton>
						<UButton size="xs" variant="ghost" icon="i-heroicons-x-mark" @click="isVisible = false" />
					</div>
				</div>
			</template>

			<div ref="logContainer" class="font-mono text-xs max-h-72 overflow-y-auto space-y-1">
				<div v-for="(log, idx) in logs" :key="idx" :class="getLogClass(log.level)">
					<span class="text-gray-500">[{{ log.timestamp }}]</span>
					<span class="ml-2">{{ log.message }}</span>
				</div>
			</div>
		</UCard>
	</div>

	<!-- Toggle button when collapsed -->
	<UButton
		v-else
		class="fixed bottom-4 right-4 z-50"
		icon="i-heroicons-bug-ant"
		@click="isVisible = true"
	>
		Debug Console
	</UButton>
</template>

<script setup lang="ts">
	import { nextTick, onMounted, onUnmounted, ref } from "vue";

	interface LogEntry {
		timestamp: string;
		level: "log" | "error" | "warn" | "info";
		message: string;
	}

	const isVisible = ref(false);
	const logs = ref<LogEntry[]>([]);
	const logContainer = ref<HTMLElement | null>(null);
	const MAX_LOGS = 500;

	// Store original console methods
	const originalConsole = {
		log: console.log,
		error: console.error,
		warn: console.warn,
		info: console.info
	};

	function addLog(level: LogEntry["level"], args: any[]) {
		const timestamp = new Date().toLocaleTimeString();
		const message = args.map((arg) =>
			typeof arg === "object" ? JSON.stringify(arg, null, 2) : String(arg)
		).join(" ");

		logs.value.push({ timestamp, level, message });

		// Keep only last MAX_LOGS entries
		if (logs.value.length > MAX_LOGS) {
			logs.value = logs.value.slice(-MAX_LOGS);
		}

		// Auto-scroll to bottom
		nextTick(() => {
			if (logContainer.value) {
				logContainer.value.scrollTop = logContainer.value.scrollHeight;
			}
		});
	}

	function interceptConsole() {
		console.log = (...args) => {
			originalConsole.log(...args);
			addLog("log", args);
		};

		console.error = (...args) => {
			originalConsole.error(...args);
			addLog("error", args);
		};

		console.warn = (...args) => {
			originalConsole.warn(...args);
			addLog("warn", args);
		};

		console.info = (...args) => {
			originalConsole.info(...args);
			addLog("info", args);
		};
	}

	function restoreConsole() {
		console.log = originalConsole.log;
		console.error = originalConsole.error;
		console.warn = originalConsole.warn;
		console.info = originalConsole.info;
	}

	function clearLogs() {
		logs.value = [];
	}

	async function copyLogs() {
		const text = logs.value.map((log) =>
			`[${log.timestamp}] [${log.level.toUpperCase()}] ${log.message}`
		).join("\n");

		await navigator.clipboard.writeText(text);
		console.info("Logs copied to clipboard");
	}

	function getLogClass(level: string) {
		switch (level) {
		case "error": return "text-red-400";
		case "warn": return "text-yellow-400";
		case "info": return "text-blue-400";
		default: return "text-gray-300";
		}
	}

	// Keyboard shortcut (Ctrl+Shift+D)
	function handleKeydown(e: KeyboardEvent) {
		if (e.ctrlKey && e.shiftKey && e.key === "D") {
			e.preventDefault();
			isVisible.value = !isVisible.value;
		}
	}

	onMounted(() => {
		interceptConsole();
		window.addEventListener("keydown", handleKeydown);
	});

	onUnmounted(() => {
		restoreConsole();
		window.removeEventListener("keydown", handleKeydown);
	});
</script>
