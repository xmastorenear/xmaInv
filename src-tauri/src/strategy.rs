use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Strategy {
    pub id: u32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct AppData {
    pub strategy: Option<Strategy>,
}

impl AppData {
    fn get_path(app_handle: &tauri::AppHandle) -> PathBuf {
        // Для Tauri v2 используется .path().app_data_dir()
        // Если у вас v1, замените на app_handle.path_resolver().app_data_dir()
        let mut path = app_handle.path().app_data_dir().expect("No app data dir");

        if !path.exists() {
            fs::create_dir_all(&path).unwrap(); // Исправлено на ::
        }
        path.push("storage.json");
        path // Без точки с запятой
    }

    pub fn load(app_handle: &tauri::AppHandle) -> Self {
        let path = Self::get_path(app_handle);
        fs::read_to_string(path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }


    pub fn save(&self, app_handle: &tauri::AppHandle) {
        let path = Self::get_path(app_handle);
        let content = serde_json::to_string_pretty(self).unwrap();
        fs::write(path, content).expect("Failed to save data"); // Исправлено на ::
    }
}