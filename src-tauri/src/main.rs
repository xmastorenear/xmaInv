#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// 1. Добавлена точка с запятой.
// Убедитесь, что файл называется именно models.rs
mod strategy;

use strategy::{AppData, Strategy};
use std::sync::Mutex;
use tauri::{Manager, State};

struct AppState(Mutex<AppData>);

#[tauri::command]
fn get_data(state: State<AppState>) -> AppData {
    let data = state.0.lock().unwrap();
    AppData { strategy: data.strategy.clone() }
}

#[tauri::command]
fn create_strategy(handle: tauri::AppHandle, state: State<AppState>, name: String) -> Strategy {
    let mut data = state.0.lock().unwrap();
    let new_strategy = Strategy { id: 1, name };

    data.strategy = Some(new_strategy.clone());
    data.save(&handle);
    new_strategy
}

fn main() {
    // 2. Изменено на стандартный tauri::Builder (или верните свое, если уверены в lib)
    tauri::Builder::default()
        .setup(|app| {
            let initial_data = AppData::load(&app.handle());
            app.manage(AppState(Mutex::new(initial_data)));
            Ok(())
        })
        // 3. Исправлено на invoke_handler (было handle)
        .invoke_handler(tauri::generate_handler![get_data, create_strategy])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}