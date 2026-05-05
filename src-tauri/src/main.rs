#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;

use models::{AppData, Strategy, Source, Transaction};
use std::sync::Mutex;
use tauri::{Manager, State};

struct AppState(Mutex<AppData>);

#[tauri::command]
fn get_data(state: State<AppState>) -> AppData {
    let data = state.0.lock().unwrap();
    println!("[DEBUG] get_data called, active strategy: {:?}", data.strategy);
    // Возвращаем копию данных для фронтенда
    AppData {
        strategy: data.strategy.clone(),
        all_strategies: data.all_strategies.clone(),
        sources: data.sources.clone(),
        transactions: data.transactions.clone(),
    }
}

#[tauri::command(rename_all = "snake_case")]
fn create_strategy(app_handle: tauri::AppHandle, state: State<AppState>, name: String, color: String) -> Result<Strategy, String> {
    let mut data = state.0.lock().unwrap();
    
    let new_id = data.all_strategies.iter().map(|s| s.id).max().unwrap_or(0) + 1;
    let new_strategy = Strategy { id: new_id, name, color };

    data.all_strategies.push(new_strategy.clone());
    data.strategy = Some(new_strategy.clone());

    data.save(&app_handle)?;
    Ok(new_strategy)
}

#[tauri::command(rename_all = "snake_case")]
fn rename_strategy(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    id: u32,
    new_name: String // Именно это имя ищет Tauri
) -> Result<(), String> {
    let mut data = state.0.lock().unwrap();
    
    if let Some(s) = data.all_strategies.iter_mut().find(|s| s.id == id) {
        s.name = new_name.clone();
    }
    
    if let Some(ref mut s) = data.strategy {
        if s.id == id {
            s.name = new_name;
        }
    }
    data.save(&app_handle)
}

#[tauri::command(rename_all = "snake_case")]
fn delete_strategy(app_handle: tauri::AppHandle, state: State<AppState>, id: u32) -> Result<(), String> {
    let mut data = state.0.lock().unwrap();

    data.all_strategies.retain(|s| s.id != id);
    data.sources.retain(|s| s.strategy_id != id); // Удаляем связанные источники

    if let Some(ref s) = data.strategy {
        if s.id == id {
            data.strategy = None;
        }
    }

    data.save(&app_handle)
}

#[tauri::command(rename_all = "snake_case")]
fn create_source(app_handle: tauri::AppHandle, state: State<AppState>, strategy_id: u32, name: String, icon_url: String) -> Result<Source, String> {
    let mut data = state.0.lock().unwrap();

    let new_id = data.sources.iter().map(|s| s.id).max().unwrap_or(0) + 1;
    let new_source = Source {
        id: new_id,
        strategy_id,
        name,
        icon_url,
        total_balance: 0.0,
        profit_loss: 0.0,
    };

    data.sources.push(new_source.clone());
    data.save(&app_handle)?;

    Ok(new_source)
}

#[tauri::command(rename_all = "snake_case")]
fn rename_source(app_handle: tauri::AppHandle, state: State<AppState>, id: u32, new_name: String) -> Result<(), String> {
    let mut data = state.0.lock().unwrap();
    
    if let Some(source) = data.sources.iter_mut().find(|s| s.id == id) {
        source.name = new_name;
        data.save(&app_handle)
    } else {
        Err("Source not found".into())
    }
}

#[tauri::command(rename_all = "snake_case")]
fn delete_source(app_handle: tauri::AppHandle, state: State<AppState>, id: u32) -> Result<(), String> {
    let mut data = state.0.lock().unwrap();
    data.sources.retain(|s| s.id != id);
    data.save(&app_handle)
}

#[tauri::command(rename_all = "snake_case")]
fn add_transaction(app_handle: tauri::AppHandle, state: State<AppState>, source_id: u32, amount: f64, timestamp: String, description: String) -> Result<(), String> {
    let mut data = state.0.lock().unwrap();

    if let Some(source) = data.sources.iter_mut().find(|s| s.id == source_id) {
        source.total_balance += amount;
    } else {
        return Err("Source not found".into());
    }

    let new_id = data.transactions.iter().map(|t| t.id).max().unwrap_or(0) + 1;
    let new_tx = Transaction {
        id: new_id,
        source_id,
        amount,
        timestamp,
        description,
    };

    data.transactions.push(new_tx);
    data.save(&app_handle)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let initial_data = AppData::load(&app.handle());
            app.manage(AppState(Mutex::new(initial_data)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_data,
            create_strategy,
            rename_strategy,
            delete_strategy,
            create_source,
            rename_source,
            delete_source,
            add_transaction
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}