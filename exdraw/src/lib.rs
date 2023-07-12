use serde::Serialize;
use serde_json::{Map, Value};

// This is copied almost exactly from
// https://github.com/etolbakov/excalidocker-rs/blob/main/src/exporters/excalidraw.rs

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExcalidrawFile {
    pub r#type: String,
    pub version: i32,
    pub source: Option<String>,
    pub elements: Vec<Element>,
    pub app_state: AppState,
    pub files: Map<String, Value>,
}

impl Default for ExcalidrawFile {
    fn default() -> Self {
        Self {
            r#type: "excalidraw".into(),
            version: 2,
            source: None,
            elements: Vec::with_capacity(0),
            app_state: Default::default(),
            files: Map::with_capacity(0),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub grid_size: i32,
    pub view_background_color: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            grid_size: 20,
            view_background_color: "#ffffff".into(),
        }
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
