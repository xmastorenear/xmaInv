#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod strategy;

use strategy::{AppData, Strategy};
use std::sync::Mutex;
use tauri::{Manager, State};
use crate::strategy::Source;
use crate::strategy::Transaction;

struct AppState(Mutex<AppData>);

#[tauri::command]
fn get_data(state: State<AppState>) -> AppData {
    let data = state.0.lock().unwrap();
    println!("[DEBUG] get_data called, strategy: {:?}", data.strategy);
    AppData {
        strategy: data.strategy.clone(),
        all_strategies: data.all_strategies.clone(),
        sources: data.sources.clone(),
        transactions: data.transactions.clone()
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

#[tauri::command(rename_all = "snake_case")]
fn rename_source(app_handle: tauri::AppHandle, id: u32, new_name: String) -> Result<(), String> {
    let mut data = AppData::load(&app_handle);

    if let Some(source) = data.sources.iter_mut().find(|s| s.id == id) {
        source.name = new_name;
        data.save(&app_handle)?;
        Ok(())
    } else {
        Err("Источник не найден".into())
    }
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

#[tauri::command(rename_all = "snake_case")] // Добавьте эту строку
fn create_source(
    app_handle: tauri::AppHandle,
    strategy_id: u32,
    name: String,
    icon_url: String
) -> Result<Source, String> {
    // ... ваш код без изменений ...
    let mut data = AppData::load(&app_handle);
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

#[tauri::command]
fn delete_source(app_handle: tauri::AppHandle, id: u32) -> Result<(), String> {
    let mut data = AppData::load(&app_handle);
    data.sources.retain(|s| s.id != id);
    data.save(&app_handle)
}

#[tauri::command]
fn rename_strategy(app_handle: tauri::AppHandle, id: u32, new_name: String) -> Result<(), String> {
    let mut data = AppData::load(&app_handle);

    // Обновляем в общем списке
    if let Some(s) = data.all_strategies.iter_mut().find(|s| s.id == id) {
        s.name = new_name.clone();
    }

    // Обновляем текущую активную, если это она
    if let Some(ref mut s) = data.strategy {
        if s.id == id {
            s.name = new_name;
        }
    }

    data.save(&app_handle)
}

#[tauri::command(rename_all = "snake_case")]
fn add_transaction(
    app_handle: tauri::AppHandle,
    source_id: u32,
    amount: f64,
    description: String,
) -> Result<(), String> {
    // 1. Загружаем текущие данные из файла
    let mut data = AppData::load(&app_handle);

    // 2. Ищем нужный источник и обновляем его баланс
    let source_exists = if let Some(source) = data.sources.iter_mut().find(|s| s.id == source_id) {
        source.total_balance += amount;
        true
    } else {
        false
    };

    if !source_exists {
        return Err("Источник не найден".into());
    }

    // 3. Создаем новую транзакцию
    let new_tx_id = data.transactions.iter().map(|t| t.id).max().unwrap_or(0) + 1;
    let new_transaction = Transaction {
        id: new_tx_id,
        source_id,
        amount,
        // Форматируем текущее время в строку
        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        description,
    };

    // 4. Добавляем транзакцию в общий список (журнал)
    data.transactions.push(new_transaction);

    // 5. Сохраняем обновленные данные в JSON
    data.save(&app_handle)?;

    println!("[DEBUG] Transaction added: {} for source {}", amount, source_id);
    Ok(())
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
            delete_strategy,
            create_source,
            delete_source,
            add_transaction,
            rename_strategy,
            rename_source
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}