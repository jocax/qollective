use crate::models::{ResponseEnvelope, GenerationResponse, TrailListItem};
use crate::error::AppError;
use smol::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Async file loader utility for scanning directories and loading trail metadata
///
/// Uses smol-compatible async primitives for file I/O operations

/// Load all trail metadata from a directory
///
/// Scans recursively for files matching patterns `response_*.json` or `work_result_*.json` and extracts metadata
/// Invalid files are skipped with warnings logged
///
/// # Arguments
/// * `directory` - Directory path to scan for trail files
///
/// # Returns
/// * `Ok(Vec<TrailListItem>)` - List of trail metadata items
/// * `Err(AppError)` - If directory doesn't exist or cannot be read
pub async fn load_trails_from_directory(directory: &str) -> Result<Vec<TrailListItem>, AppError> {
    let path = Path::new(directory);

    if !path.exists() {
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Directory does not exist: {}", directory),
        )));
    }

    if !path.is_dir() {
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Path is not a directory: {}", directory),
        )));
    }

    let mut trails = Vec::new();

    // Walk directory recursively (synchronous, but fast for directory traversal)
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

        // Try to parse the file (async)
        match parse_trail_metadata(path.to_str().unwrap()).await {
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
///
/// # Arguments
/// * `file_path` - Path to the trail JSON file
///
/// # Returns
/// * `Ok(TrailListItem)` - Parsed trail metadata
/// * `Err(AppError)` - If file cannot be read or parsed
async fn parse_trail_metadata(file_path: &str) -> Result<TrailListItem, AppError> {
    // Read file contents (async with smol)
    let contents = fs::read_to_string(file_path).await?;

    // Parse envelope structure
    let envelope: ResponseEnvelope = serde_json::from_str(&contents)
        .map_err(|e| AppError::Serialization(e))?;

    // Extract the inner JSON string from tool_response.content[0].text
    if envelope.payload.tool_response.content.is_empty() {
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "No content in tool_response",
        )));
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
        .map_err(|e| AppError::Serialization(e))?;

    // Extract trail - it's optional in the shared type
    let trail = generation_response.trail
        .as_ref()
        .ok_or_else(|| AppError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Missing trail in GenerationResponse",
        )))?;

    // Extract metadata from trail.metadata HashMap
    let metadata = &trail.metadata;

    // Extract generation_params from metadata
    let generation_params = metadata.get("generation_params")
        .ok_or_else(|| AppError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Missing generation_params in trail metadata",
        )))?;

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

    // Calculate actual node count from trail data
    let node_count = generation_response.trail_steps
        .as_ref()
        .map(|steps| steps.len())
        .unwrap_or_else(|| {
            // Fallback to generation_params requested node_count if trail_steps not available
            generation_params.get("node_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize
        });

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
///
/// # Arguments
/// * `file_path` - Path to the trail JSON file
///
/// # Returns
/// * `Ok(GenerationResponse)` - Complete trail data
/// * `Err(AppError)` - If file cannot be read or parsed
pub async fn load_trail_full(file_path: &str) -> Result<GenerationResponse, AppError> {
    // Read file contents (async with smol)
    let contents = fs::read_to_string(file_path).await?;

    // Parse envelope structure
    let envelope: ResponseEnvelope = serde_json::from_str(&contents)
        .map_err(|e| AppError::Serialization(e))?;

    // Extract the inner JSON string
    if envelope.payload.tool_response.content.is_empty() {
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "No content in tool_response",
        )));
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
        .map_err(|e| AppError::Serialization(e))?;

    Ok(generation_response)
}
