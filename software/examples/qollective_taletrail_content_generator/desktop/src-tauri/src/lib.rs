#[cfg_attr(mobile, tauri::mobile_entry_point)]

use tauri::{
	menu::{Menu, MenuItem},
	tray::TrayIconBuilder
};

pub mod error;
pub mod models;
pub mod utils;
pub mod commands;
pub mod nats;
pub mod constants;

pub fn run() {
    // Initialize rustls crypto provider (required for TLS)
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    tauri::Builder::default()
		.setup(|app| {
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

			Ok(())
		})
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
			commands::subscribe_to_generations,
			commands::unsubscribe_from_generations,
			commands::nats_connection_status,
			commands::disconnect_nats,
			commands::submit_generation_request,
			commands::replay_generation_request
		])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
