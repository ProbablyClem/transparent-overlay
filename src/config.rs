use std::io::Write;

use notify_rust::Notification;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    pub room: String,
    pub server: String,
}

impl Config {
    pub fn default() -> Self {
        Self {
            room: "".to_string(),
            server: "".to_string(),
        }
    }

    pub fn load() -> Self {
        let config_path = _config_file_path();
        if let Ok(config_str) = std::fs::read_to_string(config_path) {
            let conf = serde_json::from_str(&config_str).unwrap_or_else(|_| Self::default());
            Self {
                room: conf.room,
                server: conf.server,
            }
        } else {
            Self::default()
        }
    }
}
// TODO
// Check the only keys are "room" and "server"
// pub fn check_config_file() -> bool {
//     let config_path = _config_file_path();
//     let icon_path = get_icon_path();
//     let config_str = match std::fs::read_to_string(&config_path) {
//         Ok(s) => s,
//         Err(err) => {
//             Notification::new()
//                 .app_id("Transparent.Overlay.App")
//                 .summary("Error loading configuration file")
//                 .body(format!("Error reading {:?}", config_path).as_str())
//                 .icon(&icon_path)
//                 .show()
//                 .unwrap();
//             return false;
//         }
//     };
//     false
//     // let json: serde_json::Value = match serde_json::from_str(&config_str) {
//     //     Ok(v) => v,
//     //     Err(err) => {
//     //         log::warn!("Invalid JSON file {:?}: {err}", config_path);
//     //         return false;
//     //     }
//     // };

//     // let object = match json.as_object() {
//     //     Some(obj) => obj,
//     //     None => {
//     //         Self::show_warn("Le fichier de config n'est pas un objet JSON");
//     //         return false;
//     //     }
//     // };

//     // let allowed = ["room", "server"];
//     // let mut valid = true;

//     // for key in object.keys() {
//     //     if !allowed.contains(&key.as_str()) {
//     //         Self::show_warn(&format!("La clé '{}' n'est pas reconnue", key));
//     //         valid = false;
//     //     }
//     // }
//     // valid
// }

fn _config_file_path() -> std::path::PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        std::path::PathBuf::from(appdata)
            .join("Transparent-Overlay")
            .join("config.json")
    } else {
        std::env::current_dir().unwrap().join("config.json")
    }
}

pub fn get_icon_path() -> String {
    let temp_dir = std::env::temp_dir();
    let icon_path = temp_dir.join("transparent-overlay.png");

    if !icon_path.exists() {
        let ico_bytes = include_bytes!("../assets/icon.png");

        if let Ok(mut file) = std::fs::File::create(&icon_path) {
            let _ = file.write_all(ico_bytes);
        }
    }
    icon_path.to_string_lossy().into_owned()
}
