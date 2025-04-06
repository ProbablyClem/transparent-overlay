use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    App, Manager,
};
use tauri_plugin_autostart::ManagerExt;

use crate::{config_window::create_config_window, livechat::close_livechat};

pub fn setup_tray(app: &App) {
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).unwrap();
    let separator_i = PredefinedMenuItem::separator(app).unwrap();
    let reload_i = MenuItem::with_id(app, "reload", "Reload", true, None::<&str>).unwrap();
    let config_i = MenuItem::with_id(app, "config", "Open Config", true, None::<&str>).unwrap();
    let autostart_i = CheckMenuItem::with_id(
        app,
        "autostart",
        "Launch on Start Up",
        true,
        app.autolaunch().is_enabled().unwrap(),
        None::<&str>,
    )
    .unwrap();
    let menu = Menu::with_items(
        app,
        &[&autostart_i, &config_i, &reload_i, &separator_i, &quit_i],
    )
    .unwrap();

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Transparent Overlay")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0x0);
            }
            "reload" => {
                if let Some(livechat_window) = app.get_webview_window("livechat") {
                    livechat_window.reload().unwrap()
                }
            }
            "config" => {
                create_config_window(app.app_handle());
                close_livechat(app);
            }
            "autostart" => {
                if app.autolaunch().is_enabled().unwrap() {
                    let _ = app.autolaunch().disable();
                } else {
                    let _ = app.autolaunch().enable();
                }
            }
            _ => {
                println!("menu item {:?} not handled", event.id);
            }
        })
        .build(app)
        .expect("Failed to build tray");
}
