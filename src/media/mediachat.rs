use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Author {
    pub id: String,
    pub name: String,
    pub image: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Video,
    Image,
    Sound,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Media {
    pub id: String,
    pub url: String,
    #[serde(rename = "type")]
    pub media_type: MediaType,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct TextOptions {
    #[serde(rename = "positionX")]
    pub position_x: Option<String>,
    #[serde(rename = "positionY")]
    pub position_y: Option<String>,
    pub color: Option<String>,
    #[serde(rename = "fontSize")]
    pub font_size: Option<f32>,
    #[serde(rename = "fontFamily")]
    pub font_family: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct FileOptions {
    #[serde(rename = "positionX")]
    pub position_x: Option<String>,
    #[serde(rename = "positionY")]
    pub position_y: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct MediachatOptions {
    pub file: Option<FileOptions>,
    pub text: Option<TextOptions>,
    #[serde(rename = "hideAuthor")]
    pub hide_author: Option<bool>,
    pub target: Option<String>,
    pub target_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MediaChat {
    pub id: String,
    pub author: Author,
    pub duration: Option<f64>,
    pub message: Option<String>,
    pub media: Option<Media>,
    pub options: Option<MediachatOptions>,
}
