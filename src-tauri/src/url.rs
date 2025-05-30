use std::env;

use tauri::webview;

#[tauri::command]
pub fn get_url_from_arg() -> String {
    let args: Vec<String> = env::args().collect();
    let url = &args[1];
    url.into()
}

#[tauri::command]
pub fn url_is_parsable(url: String) -> bool {
    url.trim().parse::<webview::Url>().is_ok()
}
