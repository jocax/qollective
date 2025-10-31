/// Configuration module for TaleTrail Desktop Application
///
/// Implements CONSTANTS FIRST principle with hierarchical configuration:
/// 1. Constants (default values, relative paths only)
/// 2. config.toml (grouped configuration sections)
/// 3. .env file (optional, environment-specific overrides)
/// 4. System Environment Variables (highest priority)

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::env;

/// Application configuration root structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub application: ApplicationConfig,
    pub nats: NatsConfig,
    pub tls: TlsConfig,
    pub paths: PathsConfig,
    pub mcp: McpConfig,
    pub monitoring: MonitoringConfig,
    pub validation: ValidationConfig,
    pub timeouts: TimeoutsConfig,
    pub history: HistoryConfig,
    pub development: DevelopmentConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsConfig {
    pub url: String,
    pub connection_timeout_ms: u64,
    pub request_timeout_ms: u64,
    pub tls_enabled: bool,
    pub subjects: NatsSubjectsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsSubjectsConfig {
    pub generation_events_base: String,
    pub generation_events_tenant_pattern: String,
    pub orchestrator_request_subject: String,
    pub mcp_servers: McpServerSubjectsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerSubjectsConfig {
    pub orchestrator: String,
    pub story_generator: String,
    pub quality_control: String,
    pub constraint_enforcer: String,
    pub prompt_helper: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub certs_dir: String,
    pub ca_cert_file: String,
    pub nkeys_dir: String,
    pub desktop_nkey_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsConfig {
    pub root_directory: String,
    pub templates_dir: String,
    pub trails_dir: String,
    pub mcp_templates: McpTemplatesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTemplatesConfig {
    pub orchestrator: String,
    pub story_generator: String,
    pub quality_control: String,
    pub constraint_enforcer: String,
    pub prompt_helper: String,
    pub template_file_extension: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    pub default_timeout_ms: u64,
    pub servers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub max_event_buffer_size: usize,
    pub request_cleanup_timeout_secs: u64,
    pub event_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    pub valid_story_structures: Vec<String>,
    pub min_node_count: u32,
    pub max_node_count: u32,
    pub default_max_choices_per_node: u32,
    pub min_choices_per_node: u32,
    pub max_choices_per_node: u32,
    pub min_story_length: u32,
    pub max_story_length: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutsConfig {
    pub default_request_timeout_secs: u64,
    pub min_request_timeout_secs: u64,
    pub max_request_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryConfig {
    pub store_key: String,
    pub store_file: String,
    pub default_page_size: usize,
    pub max_page_size: usize,
    pub min_page_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentConfig {
    pub debug: bool,
    pub dev_server_port: u16,
    pub hmr_port: u16,
}

impl AppConfig {
    /// Load configuration with hierarchy: constants → config.toml → .env → env vars
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // Start with config.toml
        let config_path = Self::get_config_path()?;
        let config_content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file at {:?}: {}", config_path, e))?;

        let mut config: AppConfig = toml::from_str(&config_content)
            .map_err(|e| format!("Failed to parse config.toml: {}", e))?;

        // Load .env file if exists (optional)
        dotenv::dotenv().ok();

        // Override with environment variables (highest priority)
        Self::apply_env_overrides(&mut config);

        Ok(config)
    }

    fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        // Try multiple locations for config.toml

        // 1. Look relative to current executable (production)
        if let Ok(mut path) = env::current_exe() {
            path.pop(); // Remove executable name
            path.push("config.toml");
            if path.exists() {
                return Ok(path);
            }
        }

        // 2. Look in src-tauri directory (development)
        let dev_path = PathBuf::from("src-tauri/config.toml");
        if dev_path.exists() {
            return Ok(dev_path);
        }

        // 3. Look in current directory
        let current_path = PathBuf::from("config.toml");
        if current_path.exists() {
            return Ok(current_path);
        }

        // 4. Look in desktop/src-tauri (if running from desktop directory)
        let desktop_path = PathBuf::from("desktop/src-tauri/config.toml");
        if desktop_path.exists() {
            return Ok(desktop_path);
        }

        Err("config.toml not found in any expected location".into())
    }

    fn apply_env_overrides(config: &mut AppConfig) {
        // NATS overrides
        if let Ok(url) = env::var("TALETRAIL_NATS_URL") {
            config.nats.url = url;
        }
        if let Ok(timeout) = env::var("TALETRAIL_NATS_TIMEOUT_MS") {
            if let Ok(value) = timeout.parse() {
                config.nats.connection_timeout_ms = value;
            }
        }
        if let Ok(timeout) = env::var("TALETRAIL_NATS_REQUEST_TIMEOUT_MS") {
            if let Ok(value) = timeout.parse() {
                config.nats.request_timeout_ms = value;
            }
        }
        if let Ok(enabled) = env::var("TALETRAIL_NATS_TLS_ENABLED") {
            if let Ok(value) = enabled.parse() {
                config.nats.tls_enabled = value;
            }
        }

        // TLS overrides
        if let Ok(certs_dir) = env::var("TALETRAIL_CERTS_DIR") {
            config.tls.certs_dir = certs_dir;
        }
        if let Ok(ca_cert) = env::var("TALETRAIL_CA_CERT_FILE") {
            config.tls.ca_cert_file = ca_cert;
        }
        if let Ok(nkeys_dir) = env::var("TALETRAIL_NKEYS_DIR") {
            config.tls.nkeys_dir = nkeys_dir;
        }
        if let Ok(nkey_file) = env::var("TALETRAIL_DESKTOP_NKEY_FILE") {
            config.tls.desktop_nkey_file = nkey_file;
        }

        // Path overrides
        if let Ok(root_directory) = env::var("TALETRAIL_ROOT_DIRECTORY") {
            config.paths.root_directory = root_directory;
        }
        if let Ok(templates_dir) = env::var("TALETRAIL_TEMPLATES_DIR") {
            config.paths.templates_dir = templates_dir;
        }
        if let Ok(trails_dir) = env::var("TALETRAIL_TRAILS_DIR") {
            config.paths.trails_dir = trails_dir;
        }

        // MCP overrides
        if let Ok(timeout) = env::var("TALETRAIL_MCP_TIMEOUT_MS") {
            if let Ok(value) = timeout.parse() {
                config.mcp.default_timeout_ms = value;
            }
        }
        if let Ok(servers) = env::var("TALETRAIL_MCP_SERVERS") {
            config.mcp.servers = servers
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        // Monitoring overrides
        if let Ok(buffer_size) = env::var("TALETRAIL_MAX_EVENT_BUFFER") {
            if let Ok(value) = buffer_size.parse() {
                config.monitoring.max_event_buffer_size = value;
            }
        }
        if let Ok(timeout) = env::var("TALETRAIL_REQUEST_CLEANUP_TIMEOUT") {
            if let Ok(value) = timeout.parse() {
                config.monitoring.request_cleanup_timeout_secs = value;
            }
        }

        // NATS subjects overrides
        if let Ok(base) = env::var("TALETRAIL_GENERATION_EVENTS_BASE") {
            config.nats.subjects.generation_events_base = base;
        }
        if let Ok(pattern) = env::var("TALETRAIL_GENERATION_EVENTS_TENANT_PATTERN") {
            config.nats.subjects.generation_events_tenant_pattern = pattern;
        }
        if let Ok(subject) = env::var("TALETRAIL_ORCHESTRATOR_REQUEST_SUBJECT") {
            config.nats.subjects.orchestrator_request_subject = subject;
        }

        // MCP server subjects overrides
        if let Ok(subject) = env::var("TALETRAIL_MCP_SUBJECT_ORCHESTRATOR") {
            config.nats.subjects.mcp_servers.orchestrator = subject;
        }
        if let Ok(subject) = env::var("TALETRAIL_MCP_SUBJECT_STORY_GENERATOR") {
            config.nats.subjects.mcp_servers.story_generator = subject;
        }
        if let Ok(subject) = env::var("TALETRAIL_MCP_SUBJECT_QUALITY_CONTROL") {
            config.nats.subjects.mcp_servers.quality_control = subject;
        }
        if let Ok(subject) = env::var("TALETRAIL_MCP_SUBJECT_CONSTRAINT_ENFORCER") {
            config.nats.subjects.mcp_servers.constraint_enforcer = subject;
        }
        if let Ok(subject) = env::var("TALETRAIL_MCP_SUBJECT_PROMPT_HELPER") {
            config.nats.subjects.mcp_servers.prompt_helper = subject;
        }

        // MCP template paths overrides
        if let Ok(path) = env::var("TALETRAIL_MCP_TEMPLATES_ORCHESTRATOR") {
            config.paths.mcp_templates.orchestrator = path;
        }
        if let Ok(path) = env::var("TALETRAIL_MCP_TEMPLATES_STORY_GENERATOR") {
            config.paths.mcp_templates.story_generator = path;
        }
        if let Ok(path) = env::var("TALETRAIL_MCP_TEMPLATES_QUALITY_CONTROL") {
            config.paths.mcp_templates.quality_control = path;
        }
        if let Ok(path) = env::var("TALETRAIL_MCP_TEMPLATES_CONSTRAINT_ENFORCER") {
            config.paths.mcp_templates.constraint_enforcer = path;
        }
        if let Ok(path) = env::var("TALETRAIL_MCP_TEMPLATES_PROMPT_HELPER") {
            config.paths.mcp_templates.prompt_helper = path;
        }
        if let Ok(ext) = env::var("TALETRAIL_TEMPLATE_FILE_EXTENSION") {
            config.paths.mcp_templates.template_file_extension = ext;
        }

        // Validation overrides
        if let Ok(structures) = env::var("TALETRAIL_VALID_STORY_STRUCTURES") {
            config.validation.valid_story_structures = structures
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        if let Ok(min) = env::var("TALETRAIL_MIN_NODE_COUNT") {
            if let Ok(value) = min.parse() {
                config.validation.min_node_count = value;
            }
        }
        if let Ok(max) = env::var("TALETRAIL_MAX_NODE_COUNT") {
            if let Ok(value) = max.parse() {
                config.validation.max_node_count = value;
            }
        }
        if let Ok(default_max) = env::var("TALETRAIL_DEFAULT_MAX_CHOICES_PER_NODE") {
            if let Ok(value) = default_max.parse() {
                config.validation.default_max_choices_per_node = value;
            }
        }
        if let Ok(min) = env::var("TALETRAIL_MIN_CHOICES_PER_NODE") {
            if let Ok(value) = min.parse() {
                config.validation.min_choices_per_node = value;
            }
        }
        if let Ok(max) = env::var("TALETRAIL_MAX_CHOICES_PER_NODE") {
            if let Ok(value) = max.parse() {
                config.validation.max_choices_per_node = value;
            }
        }
        if let Ok(min) = env::var("TALETRAIL_MIN_STORY_LENGTH") {
            if let Ok(value) = min.parse() {
                config.validation.min_story_length = value;
            }
        }
        if let Ok(max) = env::var("TALETRAIL_MAX_STORY_LENGTH") {
            if let Ok(value) = max.parse() {
                config.validation.max_story_length = value;
            }
        }

        // Timeout overrides
        if let Ok(timeout) = env::var("TALETRAIL_DEFAULT_REQUEST_TIMEOUT_SECS") {
            if let Ok(value) = timeout.parse() {
                config.timeouts.default_request_timeout_secs = value;
            }
        }
        if let Ok(min) = env::var("TALETRAIL_MIN_REQUEST_TIMEOUT_SECS") {
            if let Ok(value) = min.parse() {
                config.timeouts.min_request_timeout_secs = value;
            }
        }
        if let Ok(max) = env::var("TALETRAIL_MAX_REQUEST_TIMEOUT_SECS") {
            if let Ok(value) = max.parse() {
                config.timeouts.max_request_timeout_secs = value;
            }
        }

        // History overrides
        if let Ok(key) = env::var("TALETRAIL_HISTORY_STORE_KEY") {
            config.history.store_key = key;
        }
        if let Ok(file) = env::var("TALETRAIL_HISTORY_STORE_FILE") {
            config.history.store_file = file;
        }
        if let Ok(size) = env::var("TALETRAIL_HISTORY_DEFAULT_PAGE_SIZE") {
            if let Ok(value) = size.parse() {
                config.history.default_page_size = value;
            }
        }
        if let Ok(max) = env::var("TALETRAIL_HISTORY_MAX_PAGE_SIZE") {
            if let Ok(value) = max.parse() {
                config.history.max_page_size = value;
            }
        }
        if let Ok(min) = env::var("TALETRAIL_HISTORY_MIN_PAGE_SIZE") {
            if let Ok(value) = min.parse() {
                config.history.min_page_size = value;
            }
        }

        // Development overrides
        if let Ok(debug) = env::var("TALETRAIL_DEBUG") {
            if let Ok(value) = debug.parse() {
                config.development.debug = value;
            }
        }
        if let Ok(port) = env::var("TALETRAIL_DEV_PORT") {
            if let Ok(value) = port.parse() {
                config.development.dev_server_port = value;
            }
        }
        if let Ok(port) = env::var("TALETRAIL_HMR_PORT") {
            if let Ok(value) = port.parse() {
                config.development.hmr_port = value;
            }
        }
    }

    /// Get absolute path by resolving relative path from project root
    ///
    /// # Path Resolution Strategy
    ///
    /// 1. **Development Mode**: When running `cargo tauri dev` from desktop directory
    ///    - Current directory: `.../desktop/`
    ///    - Project root detected: `.../qollective_taletrail_content_generator/`
    ///    - Relative paths in config.toml resolved from project root
    ///    - Example: `certs` → `.../qollective_taletrail_content_generator/certs`
    ///
    /// 2. **Production Mode**: When running bundled Tauri application
    ///    - Executable location: varies by platform (app bundle, install dir, etc.)
    ///    - Looks for sibling `config.toml` next to executable
    ///    - Relative paths resolved from executable directory
    ///    - Example: `certs` → `<exe_dir>/certs`
    ///
    /// 3. **Absolute Paths**: If path starts with `/` (Unix) or drive letter (Windows)
    ///    - Used as-is without modification
    ///    - Allows production deployments to reference system-wide locations
    ///    - Example: `/etc/taletrail/certs` → `/etc/taletrail/certs`
    pub fn resolve_path(&self, relative_path: &str) -> PathBuf {
        let path = PathBuf::from(relative_path);

        // If already absolute, use as-is (production deployment scenario)
        if path.is_absolute() {
            return path;
        }

        // Otherwise resolve from project root
        let project_root = Self::get_project_root();
        project_root.join(relative_path)
    }

    /// Get the project root directory
    ///
    /// # Detection Strategy
    ///
    /// 1. **Try characteristic file pattern**: Look for directory containing:
    ///    - `Cargo.toml` (workspace root marker)
    ///    - `nats-cli/` directory (TaleTrail-specific structure)
    ///    - `certs/` directory (TaleTrail-specific structure)
    ///
    ///    This identifies the `qollective_taletrail_content_generator` project root.
    ///
    /// 2. **Production fallback**: If characteristic files not found:
    ///    - Use directory containing `config.toml` as project root
    ///    - This handles bundled applications where project structure differs
    ///
    /// 3. **Final fallback**: Current working directory
    ///    - Used when all detection methods fail
    fn get_project_root() -> PathBuf {
        // Try to find project root by looking for characteristic files
        let mut current = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        // Walk up directory tree looking for project root markers
        loop {
            // Check for characteristic TaleTrail project structure
            if current.join("Cargo.toml").exists() &&
               current.join("nats-cli").exists() &&
               current.join("certs").exists() {
                return current;
            }

            // Stop at filesystem root
            if !current.pop() {
                break;
            }
        }

        // Production fallback: look for config.toml location
        // This handles bundled apps where project structure is different
        if let Ok(mut exe_path) = env::current_exe() {
            exe_path.pop(); // Remove executable name
            if exe_path.join("config.toml").exists() {
                return exe_path;
            }
        }

        // Final fallback: current directory
        env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    }

    /// Get CA certificate absolute path
    pub fn ca_cert_path(&self) -> PathBuf {
        self.resolve_path(&self.tls.certs_dir).join(&self.tls.ca_cert_file)
    }

    /// Get NKey file absolute path
    pub fn nkey_path(&self) -> PathBuf {
        self.resolve_path(&self.tls.nkeys_dir).join(&self.tls.desktop_nkey_file)
    }

    /// Get templates directory absolute path
    pub fn templates_dir(&self) -> PathBuf {
        self.resolve_path(&self.paths.templates_dir)
    }

    /// Get trails directory absolute path
    pub fn trails_dir(&self) -> PathBuf {
        self.resolve_path(&self.paths.trails_dir)
    }

    /// Get root directory absolute path
    pub fn root_directory(&self) -> PathBuf {
        self.resolve_path(&self.paths.root_directory)
    }

    /// Create test AppConfig with values from constants
    /// Used for consistent test configuration across all test modules
    #[cfg(test)]
    pub fn create_test_app_config() -> Self {
        use crate::constants::{defaults, history, monitoring, network, nats, paths, templates, timeouts, validation};

        Self {
            application: ApplicationConfig {
                name: defaults::NATS_CLIENT_NAME.to_string(),
                version: "0.1.0".to_string(),
            },
            nats: NatsConfig {
                url: network::DEFAULT_NATS_URL.to_string(),
                connection_timeout_ms: network::DEFAULT_NATS_TIMEOUT_MS,
                request_timeout_ms: network::DEFAULT_REQUEST_TIMEOUT_MS,
                tls_enabled: true,
                subjects: NatsSubjectsConfig {
                    generation_events_base: nats::GENERATION_EVENTS_BASE.to_string(),
                    generation_events_tenant_pattern: nats::GENERATION_EVENTS_TENANT_PATTERN.to_string(),
                    orchestrator_request_subject: nats::ORCHESTRATOR_REQUEST_SUBJECT.to_string(),
                    mcp_servers: McpServerSubjectsConfig {
                        orchestrator: nats::mcp_subjects::ORCHESTRATOR.to_string(),
                        story_generator: nats::mcp_subjects::STORY_GENERATOR.to_string(),
                        quality_control: nats::mcp_subjects::QUALITY_CONTROL.to_string(),
                        constraint_enforcer: nats::mcp_subjects::CONSTRAINT_ENFORCER.to_string(),
                        prompt_helper: nats::mcp_subjects::PROMPT_HELPER.to_string(),
                    },
                },
            },
            tls: TlsConfig {
                certs_dir: paths::CERTS_DIR.to_string(),
                ca_cert_file: paths::CA_CERT_FILE.to_string(),
                nkeys_dir: paths::NKEYS_DIR.to_string(),
                desktop_nkey_file: paths::DESKTOP_NKEY_FILE.to_string(),
            },
            paths: PathsConfig {
                root_directory: "taletrail-data".to_string(),
                templates_dir: paths::TEMPLATES_DIR.to_string(),
                trails_dir: paths::DEFAULT_TRAILS_DIR_NAME.to_string(),
                mcp_templates: McpTemplatesConfig {
                    orchestrator: paths::ORCHESTRATOR_TEMPLATES.to_string(),
                    story_generator: paths::STORY_GENERATOR_TEMPLATES.to_string(),
                    quality_control: paths::QUALITY_CONTROL_TEMPLATES.to_string(),
                    constraint_enforcer: paths::CONSTRAINT_ENFORCER_TEMPLATES.to_string(),
                    prompt_helper: paths::PROMPT_HELPER_TEMPLATES.to_string(),
                    template_file_extension: templates::TEMPLATE_FILE_EXTENSION.to_string(),
                },
            },
            mcp: McpConfig {
                default_timeout_ms: network::DEFAULT_REQUEST_TIMEOUT_MS,
                servers: vec![
                    "orchestrator".to_string(),
                    "story-generator".to_string(),
                ],
            },
            monitoring: MonitoringConfig {
                max_event_buffer_size: monitoring::MAX_EVENT_BUFFER_SIZE,
                request_cleanup_timeout_secs: monitoring::REQUEST_CLEANUP_TIMEOUT_SECS,
                event_types: monitoring::EVENT_TYPES.iter().map(|s| s.to_string()).collect(),
            },
            validation: ValidationConfig {
                valid_story_structures: validation::VALID_STORY_STRUCTURES.iter().map(|s| s.to_string()).collect(),
                min_node_count: validation::MIN_NODE_COUNT,
                max_node_count: validation::MAX_NODE_COUNT,
                default_max_choices_per_node: validation::DEFAULT_MAX_CHOICES_PER_NODE,
                min_choices_per_node: validation::MIN_CHOICES_PER_NODE,
                max_choices_per_node: validation::MAX_CHOICES_PER_NODE,
                min_story_length: validation::MIN_STORY_LENGTH,
                max_story_length: validation::MAX_STORY_LENGTH,
            },
            timeouts: TimeoutsConfig {
                default_request_timeout_secs: timeouts::DEFAULT_REQUEST_TIMEOUT_SECS,
                min_request_timeout_secs: timeouts::MIN_REQUEST_TIMEOUT_SECS,
                max_request_timeout_secs: timeouts::MAX_REQUEST_TIMEOUT_SECS,
            },
            history: HistoryConfig {
                store_key: history::STORE_KEY.to_string(),
                store_file: history::STORE_FILE.to_string(),
                default_page_size: history::DEFAULT_PAGE_SIZE,
                max_page_size: history::MAX_PAGE_SIZE,
                min_page_size: history::MIN_PAGE_SIZE,
            },
            development: DevelopmentConfig {
                debug: false,
                dev_server_port: 3030,
                hmr_port: 3031,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_structure() {
        let config = AppConfig::create_test_app_config();

        assert_eq!(config.nats.url, crate::constants::network::DEFAULT_NATS_URL);
        assert_eq!(config.tls.certs_dir, crate::constants::paths::CERTS_DIR);
        assert!(config.nats.tls_enabled);
    }

    #[test]
    fn test_path_resolution() {
        let config = AppConfig::create_test_app_config();

        // Test that path resolution returns non-empty paths
        let ca_cert = config.ca_cert_path();
        assert!(ca_cert.to_string_lossy().contains(crate::constants::paths::CERTS_DIR));
        assert!(ca_cert.to_string_lossy().contains(crate::constants::paths::CA_CERT_FILE));

        let nkey = config.nkey_path();
        assert!(nkey.to_string_lossy().contains(crate::constants::paths::NKEYS_DIR));
        assert!(nkey.to_string_lossy().contains(crate::constants::paths::DESKTOP_NKEY_FILE));
    }
}
