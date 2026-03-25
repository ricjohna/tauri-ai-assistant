#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod calculator;
mod commands;
mod define;
mod memory;
mod state;
mod time;
mod weather;

use commands::AppState;

fn main() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::process_message,
            commands::get_personality,
            commands::get_debug_logs,
            commands::get_idle_message,
            commands::is_configured,
            commands::load_settings,
            commands::save_settings,
            calculator::open_calculator,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
