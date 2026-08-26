use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

use super::strategy::Strategy;
use super::source::Source;
use super::transaction::Transaction;
use super::asset_group::AssetGroup;
use super::asset::Asset;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct AppData {
    pub strategy: Option<Strategy>,
    pub all_strategies: Vec<Strategy>,
    pub sources: Vec<Source>,
    pub asset_groups: Vec<AssetGroup>,
    pub assets: Vec<Asset>,
    pub transactions: Vec<Transaction>,
    pub tbank_token: Option<String>,
}

impl AppData {
    fn get_path(app_handle: &tauri::AppHandle) -> PathBuf {
        let app_data_dir = app_handle.path().app_data_dir().expect("Failed dir");
        if !app_data_dir.exists() {
            fs::create_dir_all(&app_data_dir).expect("Failed create dir");
        }
        app_data_dir.join("storage4.json")
    }

    pub fn load(app_handle: &tauri::AppHandle) -> Self {
        let path = Self::get_path(app_handle);
        if !path.exists() { return Self::default(); }
        fs::read_to_string(&path)
            .and_then(|content| Ok(serde_json::from_str(&content).unwrap_or_default()))
            .unwrap_or_else(|_| Self::default())
    }

    pub fn save(&self, app_handle: &tauri::AppHandle) -> Result<(), String> {
        let path = Self::get_path(app_handle);
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, content).map_err(|e| e.to_string())?;
        Ok(())
    }
}