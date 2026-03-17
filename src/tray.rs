use tray_icon::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    Icon, TrayIconBuilder,
};

pub fn create_tray_icon() -> anyhow::Result<tray_icon::TrayIcon> {
    let tray_menu = _create_tray_menu();
    let tray_icon = TrayIconBuilder::new()
        .with_icon(Icon::from_resource(1, None).unwrap_or_else(|e| {
            log::error!("error loading resource : {:?}", e);
            let fallback_pixels = vec![255; 32 * 32 * 4];
            Icon::from_rgba(fallback_pixels, 32, 32).expect("Error creating fallback icon")
        }))
        .with_menu(Box::new(tray_menu))
        .with_tooltip("MediaChat - Overlay")
        .build()?;
    Ok(tray_icon)
}

fn _create_tray_menu() -> Menu {
    let url_i = MenuItem::with_id("change_url", "Changer server URL", true, None);
    let logs_i = MenuItem::with_id("check_logs", "Check logs", true, None);
    let sep_i = PredefinedMenuItem::separator();
    let quit_i = MenuItem::with_id("quit", "Quit", true, None);

    Menu::with_items(&[&url_i, &logs_i, &sep_i, &quit_i]).expect("Error creating tray menu")
}
