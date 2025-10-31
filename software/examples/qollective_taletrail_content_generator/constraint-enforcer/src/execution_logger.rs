//! Execution logging for request-ID-bound server-side logs
//!
//! This module provides execution logging that binds server-side logs to request-IDs
//! by creating per-request subdirectories with log files and metadata.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    pub root_directory: String,
    pub use_request_subdirs: bool,
    pub log_level: String,
}

#[derive(Debug, Serialize)]
pub struct Metadata {
    pub request_id: String,
    pub server_name: String,
    pub tool_name: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration_ms: Option<u64>,
    pub success: bool,
}

pub struct ExecutionLogger {
    request_id: String,
    server_name: String,
    execution_dir: PathBuf,
    log_file: Option<File>,
    start_time: Instant,
    start_timestamp: String,
}

impl ExecutionLogger {
    /// Create new execution logger for a request
    pub fn new(
        request_id: String,
        server_name: String,
        config: &ExecutionConfig,
    ) -> Result<Self, std::io::Error> {
        // Create execution directory: {root}/execution/{request-id}/
        let execution_dir = if config.use_request_subdirs {
            Path::new(&config.root_directory).join(&request_id)
        } else {
            PathBuf::from(&config.root_directory)
        };

        fs::create_dir_all(&execution_dir)?;

        // Create log file: {server-name}-log.txt
        let log_path = execution_dir.join(format!("{}-log.txt", server_name));
        let mut log_file = File::create(&log_path)?;

        // Write header
        writeln!(log_file, "=== {} Execution Log ===", server_name)?;
        writeln!(log_file, "Request ID: {}", request_id)?;
        writeln!(log_file, "Started: {}\n", chrono::Utc::now().to_rfc3339())?;

        Ok(Self {
            request_id,
            server_name,
            execution_dir,
            log_file: Some(log_file),
            start_time: Instant::now(),
            start_timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Log a message to the log file
    pub fn log(&mut self, level: &str, message: &str) -> Result<(), std::io::Error> {
        if let Some(ref mut file) = self.log_file {
            let timestamp = chrono::Utc::now().format("%H:%M:%S%.3f");
            writeln!(file, "[{}] [{}] {}", timestamp, level, message)?;
            file.flush()?;
        }
        Ok(())
    }

    /// Log request parameters
    pub fn log_request(&mut self, tool_name: &str, arguments: &serde_json::Value) -> Result<(), std::io::Error> {
        self.log("INFO", &format!("Tool: {}", tool_name))?;
        self.log("DEBUG", &format!("Arguments: {}", serde_json::to_string_pretty(arguments).unwrap_or_default()))?;
        Ok(())
    }

    /// Log response
    pub fn log_response(&mut self, result: &serde_json::Value, duration_ms: u64) -> Result<(), std::io::Error> {
        self.log("INFO", &format!("Completed in {} ms", duration_ms))?;
        self.log("DEBUG", &format!("Result: {}", serde_json::to_string_pretty(result).unwrap_or_default()))?;
        Ok(())
    }

    /// Write metadata.json file
    pub fn write_metadata(&self, tool_name: &str, success: bool, duration_ms: u64) -> Result<(), std::io::Error> {
        let metadata = Metadata {
            request_id: self.request_id.clone(),
            server_name: self.server_name.clone(),
            tool_name: tool_name.to_string(),
            start_time: self.start_timestamp.clone(),
            end_time: Some(chrono::Utc::now().to_rfc3339()),
            duration_ms: Some(duration_ms),
            success,
        };

        let metadata_path = self.execution_dir.join(format!("{}-metadata.json", self.server_name));
        let json = serde_json::to_string_pretty(&metadata)?;
        fs::write(metadata_path, json)?;

        Ok(())
    }
}
