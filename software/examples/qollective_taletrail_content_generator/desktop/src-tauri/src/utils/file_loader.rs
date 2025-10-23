use crate::models::{ResponseEnvelope, GenerationResponse, TrailListItem};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// File loader utility for scanning directories and loading trail metadata
pub struct FileLoader;

impl FileLoader {
    /// Load all trail metadata from a directory
    ///
    /// Scans recursively for files matching patterns `response_*.json` or `work_result_*.json` and extracts metadata
    /// Invalid files are skipped with warnings logged
    pub fn load_trails_from_directory(directory: &str) -> Result<Vec<TrailListItem>, String> {
        let path = Path::new(directory);

        if !path.exists() {
            return Err(format!("Directory does not exist: {}", directory));
        }

        if !path.is_dir() {
            return Err(format!("Path is not a directory: {}", directory));
        }

        let mut trails = Vec::new();

        // Walk directory recursively
        for entry in WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Check if file matches pattern
            if !path.is_file() {
                continue;
            }

            let file_name = match path.file_name() {
                Some(name) => name.to_string_lossy(),
                None => continue,
            };

            // Skip non-JSON files
            if !file_name.ends_with(".json") {
                continue;
            }

            // Check if file matches known trail file patterns
            let is_trail_file = file_name.starts_with("response_")
                || file_name.starts_with("work_result_");

            if !is_trail_file {
                continue;
            }

            // Try to parse the file
            match Self::parse_trail_metadata(path.to_str().unwrap()) {
                Ok(trail_item) => {
                    trails.push(trail_item);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                    continue;
                }
            }
        }

        // Sort by generated_at descending (most recent first)
        trails.sort_by(|a, b| b.generated_at.cmp(&a.generated_at));

        Ok(trails)
    }

    /// Parse trail metadata from a single file
    fn parse_trail_metadata(file_path: &str) -> Result<TrailListItem, String> {
        // Read file contents
        let contents = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        // Parse envelope structure
        let envelope: ResponseEnvelope = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse envelope JSON: {}", e))?;

        // Extract the inner JSON string from tool_response.content[0].text
        if envelope.payload.tool_response.content.is_empty() {
            return Err("No content in tool_response".to_string());
        }

        let inner_json = &envelope.payload.tool_response.content[0].text;

        // Parse as GenerationResponse (shared type)
        // Try direct format first, then wrapped format
        let generation_response: GenerationResponse = serde_json::from_str(inner_json)
            .or_else(|_| {
                // Try wrapped format: {"generation_response": {...}}
                #[derive(serde::Deserialize)]
                struct Wrapper {
                    generation_response: GenerationResponse,
                }
                serde_json::from_str::<Wrapper>(inner_json)
                    .map(|w| w.generation_response)
            })
            .map_err(|e| format!("Failed to parse GenerationResponse: {}", e))?;

        // Extract trail - it's optional in the shared type
        let trail = generation_response.trail
            .as_ref()
            .ok_or("Missing trail in GenerationResponse")?;

        // Extract metadata from trail.metadata HashMap
        let metadata = &trail.metadata;

        // Extract generation_params from metadata
        let generation_params = metadata.get("generation_params")
            .ok_or("Missing generation_params in trail metadata")?;

        let theme = generation_params.get("theme")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let age_group = generation_params.get("age_group")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let language = generation_params.get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("en")
            .to_string();

        let node_count = generation_params.get("node_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        // Extract tenant_id from envelope metadata
        let tenant_id = if envelope.meta.tenant.is_empty() {
            None
        } else {
            Some(envelope.meta.tenant.clone())
        };

        // Build TrailListItem
        Ok(TrailListItem {
            id: envelope.meta.request_id.clone(),
            file_path: file_path.to_string(),
            title: trail.title.clone(),
            description: trail.description.clone().unwrap_or_default(),
            theme,
            age_group,
            language,
            tags: trail.tags.clone().unwrap_or_default(),
            status: format!("{:?}", generation_response.status),
            generated_at: envelope.meta.timestamp.clone(),
            node_count,
            tenant_id,
        })
    }

    /// Load full trail data from a file
    pub fn load_trail_full(file_path: &str) -> Result<GenerationResponse, String> {
        // Read file contents
        let contents = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        // Parse envelope structure
        let envelope: ResponseEnvelope = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse envelope JSON: {}", e))?;

        // Extract the inner JSON string
        if envelope.payload.tool_response.content.is_empty() {
            return Err("No content in tool_response".to_string());
        }

        let inner_json = &envelope.payload.tool_response.content[0].text;

        // Parse as GenerationResponse (shared type)
        // Try direct format first, then wrapped format
        let generation_response: GenerationResponse = serde_json::from_str(inner_json)
            .or_else(|_| {
                // Try wrapped format: {"generation_response": {...}}
                #[derive(serde::Deserialize)]
                struct Wrapper {
                    generation_response: GenerationResponse,
                }
                serde_json::from_str::<Wrapper>(inner_json)
                    .map(|w| w.generation_response)
            })
            .map_err(|e| format!("Failed to parse GenerationResponse: {}", e))?;

        Ok(generation_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn create_test_file(dir: &Path, filename: &str, content: &str) -> PathBuf {
        let file_path = dir.join(filename);
        fs::write(&file_path, content).unwrap();
        file_path
    }

    #[test]
    fn test_load_trails_from_empty_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = FileLoader::load_trails_from_directory(temp_dir.path().to_str().unwrap());

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_load_trails_from_nonexistent_directory() {
        let result = FileLoader::load_trails_from_directory("/nonexistent/path");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }

    #[test]
    fn test_parse_valid_trail_file() {
        let temp_dir = tempfile::tempdir().unwrap();

        let valid_content = r#"{
            "meta": {
                "request_id": "test-uuid-123",
                "timestamp": "2025-10-22T12:00:00Z",
                "tenant": "1",
                "version": "1.0"
            },
            "payload": {
                "tool_response": {
                    "content": [{
                        "type": "text",
                        "text": "{\"generation_response\":{\"request_id\":\"test-uuid-123\",\"progress_percentage\":100,\"status\":\"completed\",\"trail\":{\"title\":\"Test Trail\",\"description\":\"A test trail\",\"is_public\":false,\"status\":\"DRAFT\",\"tags\":[],\"metadata\":{\"generation_params\":{\"age_group\":\"8-10\",\"theme\":\"Adventure\",\"language\":\"en\",\"node_count\":5},\"start_node_id\":\"node1\"}}}}"
                    }],
                    "isError": false
                }
            }
        }"#;

        let file_path = create_test_file(temp_dir.path(), "response_test.json", valid_content);

        let result = FileLoader::parse_trail_metadata(file_path.to_str().unwrap());
        assert!(result.is_ok());

        let trail = result.unwrap();
        assert_eq!(trail.id, "test-uuid-123");
        assert_eq!(trail.title, "Test Trail");
        assert_eq!(trail.theme, "Adventure");
        assert_eq!(trail.age_group, "8-10");
        assert_eq!(trail.node_count, 5);
    }

    #[test]
    fn test_load_trails_skips_invalid_files() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create an invalid JSON file
        create_test_file(temp_dir.path(), "response_invalid.json", "invalid json");

        let result = FileLoader::load_trails_from_directory(temp_dir.path().to_str().unwrap());

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_load_trails_filters_by_pattern() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create files with different patterns
        create_test_file(temp_dir.path(), "not_matching.json", "{}");
        create_test_file(temp_dir.path(), "response_matching.json", "invalid");
        create_test_file(temp_dir.path(), "work_result_matching.json", "invalid");

        let result = FileLoader::load_trails_from_directory(temp_dir.path().to_str().unwrap());

        // Should find both response_matching.json and work_result_matching.json but fail to parse them (count = 0)
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
}
