use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use crate::config::{Config, MonitorPos};
use crate::monitors_utils::get_monitor_infos;

#[tauri::command]
pub async fn close_config_window(app: AppHandle) {
    app.get_webview_window("config")
        .map(|window| window.close().expect("Error Closing the config window"))
        .expect("Can't find config window");
}

#[tauri::command]
pub fn get_config(app: AppHandle) -> Config {
    Config::load(&app).unwrap_or_else(Config::empty)
}

#[tauri::command]
pub fn save_config(app: AppHandle, config: Config) {
    config.save(&app);
}

#[tauri::command]
pub fn get_available_monitors(_app: AppHandle) -> Vec<MonitorPos> {
    get_monitor_infos().unwrap()
}

pub fn create_config_window(app: &tauri::AppHandle) -> WebviewWindow {
    if let Some(window) = app.get_webview_window("config") {
        window.set_focus().unwrap();
        if window.is_minimized().unwrap() {
            window.unminimize().unwrap();
        }
        return window;
    }
    WebviewWindowBuilder::new(app, "config", WebviewUrl::App("config.html".into()))
        .title("Transparent Overlay - Config")
        .resizable(false)
        .center()
        .inner_size(350.0, 275.0)
        .maximizable(false)
        .theme(Some(tauri::Theme::Dark))
        .build()
        .unwrap()
}
