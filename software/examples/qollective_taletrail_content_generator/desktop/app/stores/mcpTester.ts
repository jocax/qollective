import type {
	GroupedTemplates,
	JsonSchema,
	McpResponseEnvelope,
	ServerName,
	TemplateData,
	TemplateInfo
} from "@/types/mcp";
import { defineStore } from "pinia";
import { computed, ref } from "vue";

/**
 * MCP Tester Store
 *
 * Manages state for the MCP Testing UI including:
 * - Server selection
 * - Template management
 * - Current request/response
 * - UI visibility toggles
 */
export const useMcpTesterStore = defineStore("mcpTester", () => {
	// ============================================================================
	// State
	// ============================================================================

	const selectedServer = ref<ServerName>("orchestrator");

	// Template state
	const availableTemplates = ref<GroupedTemplates | null>(null);
	const selectedTemplate = ref<TemplateInfo | null>(null);
	const templateContent = ref<TemplateData | null>(null);
	const templateSchema = ref<JsonSchema | null>(null);

	// Request/Response state
	const requestParams = ref<Record<string, any>>({});
	const requestJson = ref<string>("");
	const currentRequest = ref<any>(null);
	const currentResponse = ref<McpResponseEnvelope | null>(null);

	// UI state
	const showHistory = ref(false);
	const showVerboseMode = ref(false);
	const isLoading = ref(false);
	const isLoadingResponse = ref(false);
	const error = ref<string | null>(null);

	// ============================================================================
	// Actions - Server
	// ============================================================================

	function setServer(server: ServerName) {
		selectedServer.value = server;
	}

	// ============================================================================
	// Actions - Template Management
	// ============================================================================

	function loadTemplates(templates: GroupedTemplates) {
		availableTemplates.value = templates;
	}

	function selectTemplate(template: TemplateInfo) {
		selectedTemplate.value = template;
	}

	function setTemplateContent(content: TemplateData) {
		templateContent.value = content;
		// Initialize request params with template arguments from envelope
		const args = content.envelope?.payload?.tool_call?.params?.arguments || {};
		requestParams.value = { ...args };
		requestJson.value = JSON.stringify(args, null, 2);
	}

	function setTemplateSchema(schema: JsonSchema) {
		templateSchema.value = schema;
	}

	function clearTemplate() {
		selectedTemplate.value = null;
		templateContent.value = null;
		templateSchema.value = null;
		requestParams.value = {};
		requestJson.value = "";
	}

	// ============================================================================
	// Actions - Request Management
	// ============================================================================

	function updateRequestParams(params: Record<string, any>) {
		requestParams.value = params;
		// Sync with JSON
		requestJson.value = JSON.stringify(params, null, 2);
	}

	function updateRequestJson(json: string) {
		requestJson.value = json;
		// Try to parse and sync with params
		try {
			const parsed = JSON.parse(json);
			requestParams.value = parsed;
		} catch {
			// Invalid JSON, don't update params
		}
	}

	function setCurrentRequest(request: any) {
		currentRequest.value = request;
	}

	// ============================================================================
	// Actions - Response Management
	// ============================================================================

	function setCurrentResponse(response: McpResponseEnvelope | null) {
		currentResponse.value = response;
	}

	function setResponse(response: McpResponseEnvelope) {
		currentResponse.value = response;
		error.value = null;
	}

	function clearResponse() {
		currentResponse.value = null;
		error.value = null;
	}

	// ============================================================================
	// Actions - UI State
	// ============================================================================

	function toggleHistory() {
		showHistory.value = !showHistory.value;
	}

	function toggleVerboseMode() {
		showVerboseMode.value = !showVerboseMode.value;
	}

	function setLoading(loading: boolean) {
		isLoading.value = loading;
	}

	function setLoadingResponse(loading: boolean) {
		isLoadingResponse.value = loading;
	}

	function setError(err: string | null) {
		error.value = err;
		if (err) {
			currentResponse.value = null;
		}
	}

	// ============================================================================
	// Actions - Clear State
	// ============================================================================

	function clearState() {
		clearTemplate();
		clearResponse();
		currentRequest.value = null;
		error.value = null;
		isLoading.value = false;
	}

	function reset() {
		clearState();
		// Don't reset server or editor mode - keep user preferences
	}

	// ============================================================================
	// Getters
	// ============================================================================

	const hasRequest = computed(() => currentRequest.value !== null);
	const hasResponse = computed(() => currentResponse.value !== null);
	const hasError = computed(() => error.value !== null);
	const hasTemplate = computed(() => templateContent.value !== null);
	const hasSchema = computed(() => templateSchema.value !== null);

	const canSend = computed(() => {
		if (!templateContent.value) return false;
		if (isLoading.value) return false;
		// Check if JSON is valid
		try {
			JSON.parse(requestJson.value);
			return true;
		} catch {
			return false;
		}
	});

	return {
		// State
		selectedServer,
		availableTemplates,
		selectedTemplate,
		templateContent,
		templateSchema,
		requestParams,
		requestJson,
		currentRequest,
		currentResponse,
		showHistory,
		showVerboseMode,
		isLoading,
		isLoadingResponse,
		error,

		// Actions
		setServer,
		loadTemplates,
		selectTemplate,
		setTemplateContent,
		setTemplateSchema,
		clearTemplate,
		updateRequestParams,
		updateRequestJson,
		setCurrentRequest,
		setCurrentResponse,
		setResponse,
		clearResponse,
		toggleHistory,
		toggleVerboseMode,
		setLoading,
		setLoadingResponse,
		setError,
		clearState,
		reset,

		// Getters
		hasRequest,
		hasResponse,
		hasError,
		hasTemplate,
		hasSchema,
		canSend
	};
});
