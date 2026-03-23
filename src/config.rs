use notify_rust::Notification;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct IniRoot {
    #[serde(rename = "MediaChat")]
    config: Config,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "MEDIACHAT_SERVER")]
    pub server: String,
    #[serde(rename = "MEDIACHAT_ROOM")]
    pub room: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: "".into(),
            room: "".into(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = config_file_path();
        let parsed = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_ini::from_str::<IniRoot>(&s).ok())
            .map(|r| r.config);

        match parsed {
            Some(cfg) => cfg,
            None => {
                let default = Config::default();
                write_ini(&default, &path);
                default
            }
        }
    }
}

pub fn check_config_file(server: Option<&str>, room: Option<&str>) -> bool {
    let config_path = config_file_path();
    // let icon_path = get_icon_path();

    let mut cfg = if config_path.exists() {
        match std::fs::read_to_string(&config_path)
            .map_err(|e| e.to_string())
            .and_then(|s| serde_ini::from_str::<IniRoot>(&s).map_err(|e| e.to_string()))
        {
            Ok(r) => r.config,
            Err(e) => {
                let default: Config = Config::default();
                write_ini(&default, &config_path);

                let err_msg = e
                    .strip_prefix("Custom(\"")
                    .and_then(|s| s.strip_suffix("\")"))
                    .unwrap_or(&e)
                    .to_string();

                Notification::new()
                    .app_id("MediaChat") // Requires AUMID registered in registry (done by installer)
                    .summary("Configuration error - Config.ini")
                    .body(&format!("Can't parse config.ini : {err_msg}"))
                    // .icon(&icon_path)
                    .show()
                    .ok();
                let _ = std::process::Command::new("notepad")
                    .arg(&config_path)
                    .spawn();
                return false;
            }
        }
    } else {
        if let Some(parent) = config_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let default = Config::default();
        write_ini(&default, &config_path);
        default
    };

    // Override conf var with cli args
    if let Some(s) = server {
        cfg.server = s.to_string();
    }
    if let Some(r) = room {
        cfg.room = r.to_string();
    }

    // Check empty fields
    let mut empty_fields: Vec<&str> = Vec::new();
    if cfg.server.is_empty() {
        empty_fields.push("MEDIACHAT_SERVER");
    }
    if cfg.room.is_empty() {
        empty_fields.push("MEDIACHAT_ROOM");
    }

    if !empty_fields.is_empty() {
        write_ini(&cfg, &config_path);
        let keys = empty_fields
            .iter()
            .map(|k| format!("\"{k}\""))
            .collect::<Vec<_>>()
            .join(" and ");
        Notification::new()
            .app_id("MediaChat") // Requires AUMID registered in registry (done by installer)
            .summary("Configuration error - Empty key(s)")
            .body(&format!(
                "{keys} key{} {} empty.",
                if empty_fields.len() > 1 { "s" } else { "" },
                if empty_fields.len() > 1 { "are" } else { "is" },
            ))
            // .icon(&icon_path)
            .show()
            .ok();
        let _ = std::process::Command::new("notepad")
            .arg(&config_path)
            .spawn();
        return false;
    }

    // Save CLI args into conf
    if server.is_some() || room.is_some() {
        write_ini(&cfg, &config_path);
    }

    true
}

fn write_ini(cfg: &Config, path: &std::path::Path) {
    let root = IniRoot {
        config: cfg.clone(),
    };
    if let Ok(s) = serde_ini::to_string(&root) {
        let _ = std::fs::write(path, s);
    }
}

fn config_file_path() -> std::path::PathBuf {
    if let Ok(appdata) = std::env::var("LOCALAPPDATA") {
        std::path::PathBuf::from(appdata)
            .join("Mediachat")
            .join("config.ini")
    } else {
        std::env::current_dir().unwrap().join("config.ini")
    }
}

// TEST IF NOT NEEDED WITH WINDOWS INSTALLER
// pub fn get_icon_path() -> String {
//     let temp_dir = std::env::temp_dir();
//     let icon_path = temp_dir.join("transparent-overlay.png");

//     if !icon_path.exists() {
//         let ico_bytes = include_bytes!("../assets/icon.png");

//         if let Ok(mut file) = std::fs::File::create(&icon_path) {
//             let _ = file.write_all(ico_bytes);
//         }
//     }
//     icon_path.to_string_lossy().into_owned()
// }
