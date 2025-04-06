// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::env;

use config::Config;
use config_window::{
    close_config_window, create_config_window, get_available_monitors, get_config, save_config,
};
use livechat::{create_window_livechat, open_livechat_window};
use tray::setup_tray;
use url::{get_url_from_arg, url_is_parsable};

mod config;
mod config_window;
mod livechat;
mod monitors_utils;
mod tray;
mod url;
mod warning;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|_, _, _| {}))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .setup(move |app| {
            let handle = app.handle();

            match Config::load(handle) {
                Some(config) => {
                    create_window_livechat(handle, &config)?.maximize().unwrap();
                }
                None => {
                    create_config_window(handle);
                }
            };

            setup_tray(app);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_url_from_arg,
            url_is_parsable,
            open_livechat_window,
            close_config_window,
            get_config,
            save_config,
            get_available_monitors
        ])
        .run(tauri::generate_context!())
        .expect("Error launching window");
}
