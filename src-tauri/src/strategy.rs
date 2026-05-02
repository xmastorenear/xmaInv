use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Strategy {
    pub id: u32,
    pub name: String,
    pub color: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Source {
    pub id: u32,
    pub strategy_id: u32,
    pub name: String,
    pub icon_url: String,
    pub total_balance: f64,
    pub profit_loss: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    pub id: u32,
    pub source_id: u32,
    pub amount: f64,
    pub timestamp: String,
    pub description: String,
}

// ОСТАВЛЯЕМ ТОЛЬКО ОДНУ СТРУКТУРУ AppData
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct AppData {
    pub strategy: Option<Strategy>,
    pub all_strategies: Vec<Strategy>,
    pub sources: Vec<Source>,
    pub transactions: Vec<Transaction>,
}

impl AppData {
    fn get_path(app_handle: &tauri::AppHandle) -> PathBuf {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .expect("Failed to get app data dir");

        if !app_data_dir.exists() {
            fs::create_dir_all(&app_data_dir).expect("Failed to create app data dir");
        }

        let mut file_path = app_data_dir;
        file_path.push("storage3.json"); // Вернул стандартное имя, если нужно
        file_path
    }

    pub fn load(app_handle: &tauri::AppHandle) -> Self {
        let path = Self::get_path(app_handle);

        if !path.exists() {
            return Self::default();
        }

        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self, app_handle: &tauri::AppHandle) -> Result<(), String> {
        let path = Self::get_path(app_handle);
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| e.to_string())?;
        fs::write(&path, content).map_err(|e| e.to_string())?;
        Ok(())
    }
}
