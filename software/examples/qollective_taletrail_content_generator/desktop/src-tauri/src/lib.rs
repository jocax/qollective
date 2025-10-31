#[cfg_attr(mobile, tauri::mobile_entry_point)]

use tauri::{
	menu::{Menu, MenuItem},
	tray::TrayIconBuilder,
	Manager
};

pub mod error;
pub mod models;
pub mod utils;
pub mod commands;
pub mod nats;
pub mod constants;
pub mod config;
pub mod services;

pub fn run() {
    // Initialize rustls crypto provider (required for TLS)
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    // Load application configuration
    let app_config = config::AppConfig::load()
        .expect("Failed to load application configuration. Ensure config.toml exists and is valid.");

    // Create RequestTracker with cleanup timeout from config
    let request_tracker = nats::RequestTracker::new(app_config.monitoring.request_cleanup_timeout_secs);

    // Clone tracker for cleanup task
    let tracker_for_cleanup = request_tracker.clone();

    // Spawn periodic cleanup task
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5 minutes
        loop {
            interval.tick().await;
            let removed = tracker_for_cleanup.cleanup_old_requests().await;
            if removed > 0 {
                eprintln!("[RequestTracker] Periodic cleanup removed {} old requests", removed);
            }
        }
    });

    // Clone app_config for setup closure
    let app_config_for_setup = app_config.clone();

    tauri::Builder::default()
		.setup(move |app| {
			let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
			let menu = Menu::with_items(app, &[&quit_i])?;

			let _tray = TrayIconBuilder::new()
				.menu(&menu)
				.show_menu_on_left_click(true)
				.icon(app.default_window_icon().unwrap().clone())
				.on_menu_event(|app, event| match event.id.as_ref() {
					"quit" => {
						app.exit(0);
					}
					other => {
						println!("menu item {} not handled", other);
					}
				})
				.build(app)?;

			// Initialize root directory structure on startup
			let root_path = app_config_for_setup.root_directory();
			if let Err(e) = utils::directory_manager::ensure_directory_structure(&root_path) {
				eprintln!("[TaleTrail] Warning: Failed to initialize root directory structure: {}", e);
			} else {
				println!("[TaleTrail] Root directory initialized at: {:?}", root_path);
			}

			// Copy example templates from source on first run
			let source_templates_path = app_config_for_setup.source_templates_dir();
			if let Err(e) = utils::directory_manager::initialize_templates_from_source(&root_path, &source_templates_path) {
				eprintln!("[TaleTrail] Warning: Failed to initialize templates from source: {}", e);
			} else {
				println!("[TaleTrail] Example templates initialized from: {:?}", source_templates_path);
			}

			// Auto-start NATS monitoring
			let app_handle = app.handle().clone();
			let config_for_monitoring = app_config_for_setup.clone();
			tauri::async_runtime::spawn(async move {
				let nats_url = config_for_monitoring.nats.url.clone();
				let ca_cert_path = config_for_monitoring.ca_cert_path();
				let nkey_path = config_for_monitoring.nkey_path();

				match nats::monitoring::start_monitoring(nats_url, app_handle, ca_cert_path, nkey_path).await {
					Ok(_) => eprintln!("[TaleTrail] NATS monitoring started successfully"),
					Err(e) => eprintln!("[TaleTrail] Warning: Failed to start NATS monitoring: {}", e),
				}
			});

			// Auto-start NATS connection for MCP requests
			let app_handle_mcp = app.handle().clone();
			let config_for_mcp = app_config_for_setup.clone();
			tauri::async_runtime::spawn(async move {
				// Use the NatsState to initialize connection
				let state = app_handle_mcp.state::<commands::nats_commands::NatsState>();
				let mut client_guard = state.client().write().await;

				// If already connected, skip
				if client_guard.is_some() {
					return;
				}

				let nats_config = nats::NatsConfig::from_app_config(&config_for_mcp);
				let client = nats::NatsClient::new(nats_config);

				match client.connect().await {
					Ok(_) => {
						*client_guard = Some(client);
						eprintln!("[TaleTrail] NATS connected for MCP requests");
					}
					Err(e) => eprintln!("[TaleTrail] Warning: Failed to connect NATS for MCP: {}", e),
				}
			});

			Ok(())
		})
		.manage(app_config)
		.manage(request_tracker)
		.manage(commands::nats_commands::NatsState::new())
		.plugin(tauri_plugin_shell::init())
		.plugin(tauri_plugin_notification::init())
		.plugin(tauri_plugin_os::init())
		.plugin(tauri_plugin_fs::init())
		.plugin(tauri_plugin_store::Builder::new().build())
		.plugin(tauri_plugin_dialog::init())
		.invoke_handler(tauri::generate_handler![
			commands::load_trails_from_directory,
			commands::load_trail_full,
			commands::delete_trail,
			commands::add_bookmark,
			commands::remove_bookmark,
			commands::get_bookmarks,
			commands::save_preferences,
			commands::load_preferences,
			commands::load_config_toml,
			commands::subscribe_to_generations,
			commands::unsubscribe_from_generations,
			commands::nats_connection_status,
			commands::connect_nats_for_mcp,
			commands::disconnect_nats,
			commands::get_active_requests,
			commands::submit_generation_request,
			commands::replay_generation_request,
			commands::list_mcp_templates,
			commands::load_mcp_template,
			commands::get_template_schema,
			commands::send_mcp_request,
			commands::send_mcp_template_request,
			commands::send_envelope_direct,
			commands::save_request_to_history,
			commands::load_request_history,
			commands::delete_history_entry,
			commands::clear_request_history,
			commands::initialize_root_directory,
			commands::get_templates_directory,
			commands::initialize_templates,
			commands::prepare_execution_directory,
			commands::list_execution_directories,
			commands::save_request_file,
			commands::save_response_file,
			commands::load_execution_file,
			commands::start_nats_monitoring,
			commands::stop_nats_monitoring,
			commands::get_monitoring_status
		])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
