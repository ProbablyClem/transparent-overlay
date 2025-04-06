use tauri::{webview, AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use crate::{config::Config, warning::create_warning_window};

pub fn create_window_livechat(
    app: &tauri::AppHandle,
    config: &Config,
) -> Result<WebviewWindow, String> {
    let url: webview::Url = match config.url.trim().parse() {
        Ok(parsed_url) => parsed_url,
        Err(_) => {
            create_warning_window("Invalid URL to parse".into(), "Error parsing URL".into());
            return Err("Error parsing URL".to_string());
        }
    };

    let window: Result<WebviewWindow, tauri::Error> =
        WebviewWindowBuilder::new(app, "livechat", WebviewUrl::External(url.clone()))
            .title("Transparent Overlay")
            .transparent(true)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .position(config.monitor.pos_x.into(), 0.0)
            .build();

    match window {
        Ok(w) => {
            w.maximize().unwrap();
            let hwnd = w.hwnd().unwrap().0;
            let _pre_val;
            let hwnd = windows::Win32::Foundation::HWND(hwnd);

            unsafe {
                use windows::Win32::UI::WindowsAndMessaging::*;
                let nindex = GWL_EXSTYLE;
                let style = WS_EX_APPWINDOW
                    | WS_EX_COMPOSITED
                    | WS_EX_LAYERED
                    | WS_EX_TRANSPARENT
                    | WS_EX_TOPMOST;
                _pre_val = SetWindowLongA(hwnd, nindex, style.0 as i32);
            };
            Ok(w)
        }
        Err(e) => Err(format!(
            "Failed to create window with the url {} : {}",
            url, e
        )),
    }
}

pub fn close_livechat(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("livechat") {
        window.close().expect("Error closing the livechat window");
    }
}

#[tauri::command]
pub async fn open_livechat_window(app: tauri::AppHandle, config: Config) -> Result<(), String> {
    create_window_livechat(&app, &config)
        .map(|_| ())
        .map_err(|_| format!("Cannot create livechat window with config {}", config))
}
