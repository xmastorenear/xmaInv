#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;

use models::{AppData, Strategy, Source, Transaction, AssetGroup, Asset};
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
        asset_groups: data.asset_groups.clone(),
        assets: data.assets.clone(),
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
fn create_asset_group(app_handle: tauri::AppHandle, state: State<AppState>, strategy_id: u32, name: String) -> Result<AssetGroup, String> {
    let mut data = state.0.lock().unwrap();
    let new_id = data.asset_groups.iter().map(|g| g.id).max().unwrap_or(0) + 1;
    let new_group = AssetGroup {
        id: new_id,
        strategy_id,
        name,
        total_value: 0.0,
    };
    data.asset_groups.push(new_group.clone());
    data.save(&app_handle)?;
    Ok(new_group)
}

// И проверьте наличие команды для создания круглых бабблов-активов:
#[tauri::command(rename_all = "snake_case")]
fn create_asset(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    group_id: u32,
    ticker: String,
    amount: f64,
    buy_price: f64,
) -> Result<Asset, String> {
    let mut data = state.0.lock().unwrap();
    let new_id = data.assets.iter().map(|a| a.id).max().unwrap_or(0) + 1;
    let new_asset = Asset {
        id: new_id,
        group_id,
        ticker: ticker.to_uppercase(),
        amount,
        buy_price,
    };

    if let Some(group) = data.asset_groups.iter_mut().find(|g| g.id == group_id) {
        group.total_value += amount * buy_price;
    }

    data.assets.push(new_asset.clone());
    data.save(&app_handle)?;
    Ok(new_asset)
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
fn delete_asset_group(app_handle: tauri::AppHandle, state: State<AppState>, id: u32) -> Result<(), String> {
    let mut data = state.0.lock().unwrap();

    // Удаляем саму группу
    data.asset_groups.retain(|g| g.id != id);

    // (Опционально) Удаляем связанные активы, если они уже есть в моделях
    // data.assets.retain(|a| a.group_id != id);

    data.save(&app_handle)
}

#[tauri::command(rename_all = "snake_case")]
fn delete_asset(app_handle: tauri::AppHandle, state: State<AppState>, id: u32) -> Result<(), String> {
    let mut data = state.0.lock().unwrap();

    // Удаляем конкретный актив из списка по его ID
    // Примечание: Убедитесь, что массив в структуре AppData называется именно `assets`
    data.assets.retain(|a| a.id != id);

    // Сохраняем изменения в файл storage3.json
    data.save(&app_handle)?;

    println!("[DEBUG] Успешно удален актив с ID: {}", id);
    Ok(())
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

#[tauri::command(rename_all = "snake_case")]
fn buy_more_asset(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    id: u32,
    added_amount: f64,
    execution_price: f64
) -> Result<(), String> {
    let mut data = state.0.lock().unwrap();

    // Создаем переменные для лога заранее
    let mut log_amount = 0.0;
    let mut log_price = 0.0;
    let mut found = false;

    // Внутри этого блока вносим изменения и сразу отпускаем заимствование asset
    if let Some(asset) = data.assets.iter_mut().find(|a| a.id == id) {
        let current_cost = asset.amount * asset.buy_price;
        let added_cost = added_amount * execution_price;

        asset.amount += added_amount;
        asset.buy_price = (current_cost + added_cost) / asset.amount;

        // Копируем новые значения для лога (f64 копируются безопасно)
        log_amount = asset.amount;
        log_price = asset.buy_price;
        found = true;
    }

    if found {
        // Теперь data абсолютно свободна от мутабельных заимствований элементов массива
        data.save(&app_handle)?;
        println!("[DEBUG] Докуплен актив ID {}. Новое кол-во: {}, Цена: {}", id, log_amount, log_price);
        Ok(())
    } else {
        Err("Asset not found".into())
    }
}

#[tauri::command(rename_all = "snake_case")]
fn sell_asset(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    id: u32,
    sell_amount: f64,
    _execution_price: f64 // Цена продажи (можно использовать позже для фиксации финреза в транзакциях)
) -> Result<(), String> {
    let mut data = state.0.lock().unwrap();
    let mut should_remove = false;
    let mut found = false;

    if let Some(asset) = data.assets.iter_mut().find(|a| a.id == id) {
        found = true;
        if asset.amount <= sell_amount {
            should_remove = true;
        } else {
            asset.amount -= sell_amount;
        }
    }

    if found {
        if should_remove {
            // Если позиция закрыта в ноль — удаляем из вектора базы данных
            data.assets.retain(|a| a.id != id);
        }
        data.save(&app_handle)?;
        println!("[DEBUG] Rust успешно обработал продажу актива ID {}", id);
        Ok(())
    } else {
        Err("Asset not found in backend".into())
    }
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
            add_transaction,
            create_asset_group,
            delete_asset_group,
            create_asset,
            delete_asset,
            buy_more_asset,
            sell_asset
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}