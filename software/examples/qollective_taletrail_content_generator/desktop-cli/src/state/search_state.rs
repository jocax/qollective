use std::sync::{Arc, RwLock};
use std::path::PathBuf;

/// Context for managing search/execution history state
#[derive(Clone)]
pub struct SearchContext {
    state: Arc<RwLock<SearchState>>,
}

/// Internal state for search view
struct SearchState {
    root_directory: PathBuf,
    execution_dirs: Vec<ExecutionDirectory>,
    selected_dir_index: usize,
    selected_server_index: usize,
    search_query: String,
    expanded_request: Option<String>,
    expanded_response: Option<String>,
}

/// Represents an execution directory containing server logs
#[derive(Clone, Debug)]
pub struct ExecutionDirectory {
    pub name: String,
    pub path: PathBuf,
    pub servers: Vec<ServerEntry>,
    pub expanded: bool,
}

/// Represents a server entry within an execution directory
#[derive(Clone, Debug)]
pub struct ServerEntry {
    pub name: String,
    pub request_path: PathBuf,
    pub response_path: PathBuf,
}

impl SearchContext {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(SearchState {
                root_directory: PathBuf::from("./execution"),
                execution_dirs: vec![],
                selected_dir_index: 0,
                selected_server_index: 0,
                search_query: String::new(),
                expanded_request: None,
                expanded_response: None,
            })),
        }
    }

    /// Load execution directories from filesystem
    pub fn load_execution_directories(&self) -> Result<(), String> {
        let mut state = self.state.write().unwrap();

        // Mock data for now (real implementation would scan filesystem)
        state.execution_dirs = vec![
            ExecutionDirectory {
                name: "req-abc123-2025-01-15".to_string(),
                path: PathBuf::from("./execution/req-abc123"),
                servers: vec![
                    ServerEntry {
                        name: "orchestrator".to_string(),
                        request_path: PathBuf::from("./execution/req-abc123/orchestrator_request.json"),
                        response_path: PathBuf::from("./execution/req-abc123/orchestrator_response.json"),
                    },
                    ServerEntry {
                        name: "story-generator".to_string(),
                        request_path: PathBuf::from("./execution/req-abc123/story_generator_request.json"),
                        response_path: PathBuf::from("./execution/req-abc123/story_generator_response.json"),
                    },
                ],
                expanded: true,
            },
            ExecutionDirectory {
                name: "req-def456-2025-01-15".to_string(),
                path: PathBuf::from("./execution/req-def456"),
                servers: vec![
                    ServerEntry {
                        name: "orchestrator".to_string(),
                        request_path: PathBuf::from("./execution/req-def456/orchestrator_request.json"),
                        response_path: PathBuf::from("./execution/req-def456/orchestrator_response.json"),
                    },
                ],
                expanded: false,
            },
            ExecutionDirectory {
                name: "req-ghi789-2025-01-14".to_string(),
                path: PathBuf::from("./execution/req-ghi789"),
                servers: vec![
                    ServerEntry {
                        name: "orchestrator".to_string(),
                        request_path: PathBuf::from("./execution/req-ghi789/orchestrator_request.json"),
                        response_path: PathBuf::from("./execution/req-ghi789/orchestrator_response.json"),
                    },
                ],
                expanded: false,
            },
        ];

        Ok(())
    }

    /// Get all execution directories
    pub fn get_execution_dirs(&self) -> Vec<ExecutionDirectory> {
        let state = self.state.read().unwrap();
        state.execution_dirs.clone()
    }

    /// Toggle directory expansion
    pub fn toggle_directory(&self, index: usize) {
        let mut state = self.state.write().unwrap();
        if let Some(dir) = state.execution_dirs.get_mut(index) {
            dir.expanded = !dir.expanded;
        }
    }

    /// Get currently selected directory index
    pub fn selected_dir_index(&self) -> usize {
        let state = self.state.read().unwrap();
        state.selected_dir_index
    }

    /// Set selected directory index
    pub fn set_selected_dir_index(&self, index: usize) {
        let mut state = self.state.write().unwrap();
        if index < state.execution_dirs.len() {
            state.selected_dir_index = index;
        }
    }

    /// Get currently selected server index
    pub fn selected_server_index(&self) -> usize {
        let state = self.state.read().unwrap();
        state.selected_server_index
    }

    /// Set selected server index
    pub fn set_selected_server_index(&self, index: usize) {
        let mut state = self.state.write().unwrap();
        state.selected_server_index = index;
    }

    /// Get root directory
    pub fn get_root_directory(&self) -> PathBuf {
        let state = self.state.read().unwrap();
        state.root_directory.clone()
    }

    /// Set search query
    pub fn set_search_query(&self, query: String) {
        let mut state = self.state.write().unwrap();
        state.search_query = query;
    }

    /// Get search query
    pub fn get_search_query(&self) -> String {
        let state = self.state.read().unwrap();
        state.search_query.clone()
    }

    /// Load and display request/response for selected server
    pub fn load_selected_content(&self) -> Result<(String, String), String> {
        let state = self.state.read().unwrap();

        if let Some(dir) = state.execution_dirs.get(state.selected_dir_index) {
            if dir.expanded && !dir.servers.is_empty() {
                let server_idx = state.selected_server_index.min(dir.servers.len() - 1);
                if let Some(_server) = dir.servers.get(server_idx) {
                    // Mock data - real implementation would read from files
                    let request = format!(
                        "{{\n  \"method\": \"generate_trail\",\n  \"params\": {{\n    \"theme\": \"Adventure\",\n    \"age_group\": \"6-8\"\n  }}\n}}",
                    );
                    let response = format!(
                        "{{\n  \"status\": \"success\",\n  \"trail_id\": \"trail-123\",\n  \"message\": \"Trail generated successfully\"\n}}",
                    );
                    return Ok((request, response));
                }
            }
        }

        Ok((
            "No request data available".to_string(),
            "No response data available".to_string(),
        ))
    }
}

impl Default for SearchContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_context_creation() {
        let ctx = SearchContext::new();
        assert_eq!(ctx.get_root_directory(), PathBuf::from("./execution"));
        assert_eq!(ctx.selected_dir_index(), 0);
        assert_eq!(ctx.selected_server_index(), 0);
    }

    #[test]
    fn test_load_execution_directories() {
        let ctx = SearchContext::new();
        let result = ctx.load_execution_directories();
        assert!(result.is_ok());

        let dirs = ctx.get_execution_dirs();
        assert_eq!(dirs.len(), 3);
        assert_eq!(dirs[0].name, "req-abc123-2025-01-15");
        assert!(dirs[0].expanded);
        assert_eq!(dirs[0].servers.len(), 2);
    }

    #[test]
    fn test_toggle_directory() {
        let ctx = SearchContext::new();
        ctx.load_execution_directories().unwrap();

        let dirs_before = ctx.get_execution_dirs();
        assert!(dirs_before[0].expanded);
        assert!(!dirs_before[1].expanded);

        ctx.toggle_directory(0);
        let dirs_after = ctx.get_execution_dirs();
        assert!(!dirs_after[0].expanded);

        ctx.toggle_directory(1);
        let dirs_after2 = ctx.get_execution_dirs();
        assert!(dirs_after2[1].expanded);
    }

    #[test]
    fn test_search_query() {
        let ctx = SearchContext::new();
        assert_eq!(ctx.get_search_query(), "");

        ctx.set_search_query("test query".to_string());
        assert_eq!(ctx.get_search_query(), "test query");
    }

    #[test]
    fn test_selected_indices() {
        let ctx = SearchContext::new();
        ctx.load_execution_directories().unwrap();

        assert_eq!(ctx.selected_dir_index(), 0);
        ctx.set_selected_dir_index(1);
        assert_eq!(ctx.selected_dir_index(), 1);

        assert_eq!(ctx.selected_server_index(), 0);
        ctx.set_selected_server_index(1);
        assert_eq!(ctx.selected_server_index(), 1);
    }

    #[test]
    fn test_load_selected_content() {
        let ctx = SearchContext::new();
        ctx.load_execution_directories().unwrap();

        let result = ctx.load_selected_content();
        assert!(result.is_ok());

        let (request, response) = result.unwrap();
        assert!(request.contains("method"));
        assert!(response.contains("status"));
    }
}
