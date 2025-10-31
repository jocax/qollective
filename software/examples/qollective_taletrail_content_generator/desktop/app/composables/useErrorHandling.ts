export function useErrorHandling() {
	const toast = useToast();

	function handleError(error: unknown, context?: string) {
		const message = error instanceof Error ? error.message : String(error);

		console.error(`Error${context ? ` in ${context}` : ""}:`, error);

		toast.add({
			title: context || "Error",
			description: message,
			color: "red",
			timeout: 5000
		});
	}

	function showSuccess(message: string, description?: string) {
		toast.add({
			title: message,
			description,
			color: "green",
			timeout: 3000
		});
	}

	function showWarning(message: string, description?: string) {
		toast.add({
			title: message,
			description,
			color: "yellow",
			timeout: 4000
		});
	}

	function showInfo(message: string, description?: string) {
		toast.add({
			title: message,
			description,
			color: "blue",
			timeout: 3000
		});
	}

	return {
		handleError,
		showSuccess,
		showWarning,
		showInfo
	};
}
