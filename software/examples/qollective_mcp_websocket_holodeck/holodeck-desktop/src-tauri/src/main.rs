// ABOUTME: Binary entry point for holodeck-desktop Tauri application  
// ABOUTME: Starts the desktop app using Tauri infrastructure

// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod websocket_client;
mod mcp_commands;

use commands::AppState;
use tracing::info;

fn main() {
    // Install default rustls crypto provider for TLS support
    let _ = rustls::crypto::CryptoProvider::install_default(
        rustls::crypto::ring::default_provider()
    );
    
    // Initialize logging (will add file logging later)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug,holodeck_desktop=debug".into())
        )
        .with_target(false)
        .with_thread_ids(true)
        .with_line_number(true)
        .init();
    
    info!("🚀 Starting Holodeck Desktop application with Tauri v2");
    info!("🎭 Enterprise Star Trek theme ready");
    info!("🔗 Coordinator URL: {}", shared_types::constants::network::coordinator_mcp_url());

    let app_state = AppState::default();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::connect_coordinator,
            commands::get_coordinator_status,
            commands::create_holodeck_session,
            commands::get_system_health,
            commands::discover_servers,
            commands::orchestrate_validation,
            commands::get_app_info,
            mcp_commands::mcp_create_holodeck_session,
            mcp_commands::mcp_character_interaction,
            mcp_commands::mcp_generate_environment,
            mcp_commands::mcp_check_content_safety,
            mcp_commands::mcp_system_status,
            mcp_commands::mcp_orchestrate_validation,
            mcp_commands::initialize_mcp_client,
            mcp_commands::get_performance_metrics,
            mcp_commands::get_system_alerts,
            mcp_commands::force_health_check
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}