#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod strategy;

use strategy::{AppData, Strategy};
use std::sync::Mutex;
use tauri::{Manager, State};

struct AppState(Mutex<AppData>);

#[tauri::command]
fn get_data(state: State<AppState>) -> AppData {
    let data = state.0.lock().unwrap();
    println!("[DEBUG] get_data called, strategy: {:?}", data.strategy);
    AppData {
        strategy: data.strategy.clone(),
        all_strategies: data.all_strategies.clone()
    }
}

#[tauri::command]
fn create_strategy(app_handle: tauri::AppHandle, name: String, color: String) -> Result<Strategy, String> {
    let mut data = AppData::load(&app_handle);

    let new_id = data.all_strategies.iter().map(|s| s.id).max().unwrap_or(0) + 1;
    let new_strategy = Strategy { id: new_id, name, color };

    data.all_strategies.push(new_strategy.clone());
    data.strategy = Some(new_strategy.clone());

    data.save(&app_handle)?;
    Ok(new_strategy)
}

#[tauri::command]
fn delete_strategy(app_handle: tauri::AppHandle, id: u32) -> Result<(), String> {
    let mut data = AppData::load(&app_handle);

    data.all_strategies.retain(|s| s.id != id);

    if let Some(ref s) = data.strategy {
        if s.id == id {
            data.strategy = None;
        }
    }

    data.save(&app_handle)
}



fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let initial_data = AppData::load(&app.handle());
            println!("[DEBUG] Initial data loaded: {:?}", initial_data.strategy);
            app.manage(AppState(Mutex::new(initial_data)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_data,
            create_strategy,
            delete_strategy
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}