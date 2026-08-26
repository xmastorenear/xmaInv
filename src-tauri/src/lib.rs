pub mod models;
pub mod proto;
pub mod tbank;

use models::{AppData, Strategy, Source, Transaction, AssetGroup, Asset};
use std::sync::Mutex;
use tauri::{Manager, State};

struct AppState(Mutex<AppData>);

pub struct TBankState {
    client: Mutex<Option<tbank::TBankClient>>,
    cache: tbank::TickerCache,
    price_cache: tbank::PriceCache,
    token: Mutex<Option<String>>,
}

impl TBankState {
    pub fn new() -> Self {
        Self {
            client: Mutex::new(None),
            cache: tbank::TickerCache::new(),
            price_cache: tbank::PriceCache::new(),
            token: Mutex::new(None),
        }
    }

    pub async fn ensure_client(&self, token: &str) -> Result<tbank::TBankClient, String> {
        {
            let is_valid = {
                let client_guard = self.client.lock().unwrap();
                let current_token = self.token.lock().unwrap();
                client_guard.is_some() && current_token.as_deref() == Some(token)
            };
            if is_valid {
                let client_guard = self.client.lock().unwrap();
                return Ok(client_guard.as_ref().expect("client exists").clone());
            }
        }

        let client = tbank::TBankClient::connect(token)
            .await
            .map_err(|e| format!("T-Bank connect failed: {}", e))?;

        {
            let mut client_guard = self.client.lock().unwrap();
            *client_guard = Some(client.clone());
            *self.token.lock().unwrap() = Some(token.to_string());
        }

        Ok(client)
    }

    pub fn reset_client(&self) {
        let mut client_guard = self.client.lock().unwrap();
        *client_guard = None;
        *self.token.lock().unwrap() = None;
    }
}

#[derive(serde::Serialize)]
pub struct AssetPrice {
    pub asset_id: u32,
    pub price: Option<f64>,
    pub stale: bool,
}

#[tauri::command]
fn get_data(state: State<AppState>) -> AppData {
    let data = state.0.lock().unwrap();
    AppData {
        strategy: data.strategy.clone(),
        all_strategies: data.all_strategies.clone(),
        sources: data.sources.clone(),
        asset_groups: data.asset_groups.clone(),
        transactions: data.transactions.clone(),
        assets: data.assets.clone(),
        tbank_token: data.tbank_token.clone(),
    }
}

#[tauri::command(rename_all = "snake_case")]
fn set_tbank_token(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    token: String,
) -> Result<(), String> {
    let mut data = state.0.lock().unwrap();
    data.tbank_token = Some(token);
    data.save(&app_handle)
}

#[tauri::command(rename_all = "snake_case")]
async fn search_instruments(
    app_state: State<'_, AppState>,
    tbank_state: State<'_, TBankState>,
    query: String,
) -> Result<Vec<tbank::InstrumentCard>, String> {
    let token = {
        let data = app_state.0.lock().unwrap();
        data.tbank_token
            .clone()
            .ok_or_else(|| "T-Bank token not set".to_string())?
    };

    let mut retried = false;
    loop {
        let client = tbank_state.ensure_client(&token).await?;
        match client.search(&query, &token, &tbank_state.cache).await {
            Ok(results) => return Ok(results),
            Err(e) if tbank::is_transport_error(&e) && !retried => {
                eprintln!("[search_instruments] transport error, reconnecting: {e}");
                tbank_state.reset_client();
                retried = true;
            }
            Err(e) => return Err(e),
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
async fn resolve_asset_uids(
    app_handle: tauri::AppHandle,
    app_state: State<'_, AppState>,
    tbank_state: State<'_, TBankState>,
) -> Result<Vec<Asset>, String> {
    let token = {
        let data = app_state.0.lock().unwrap();
        data.tbank_token
            .clone()
            .ok_or_else(|| "T-Bank token not set".to_string())?
    };

    // Unique tickers without uid (excluding cash-currency cache assets)
    let mut tickers: Vec<String> = Vec::new();
    {
        let data = app_state.0.lock().unwrap();
        for asset in data.assets.iter() {
            let is_cash_currency =
                matches!(asset.ticker.as_str(), "RUB" | "USD" | "EUR") && asset.buy_price == 1.0;
            if asset.uid.is_none() && !is_cash_currency {
                let t = asset.ticker.trim().to_uppercase();
                if !t.is_empty() && !tickers.contains(&t) {
                    tickers.push(t);
                }
            }
        }
    }

    for ticker in &tickers {
        let mut retried = false;
        loop {
            let client = tbank_state.ensure_client(&token).await?;
            match client
                .find_uid_by_ticker(ticker, &token, &tbank_state.cache)
                .await
            {
                Ok(Some(uid)) => {
                    let mut data = app_state.0.lock().unwrap();
                    for asset in data.assets.iter_mut() {
                        if asset.uid.is_none() && asset.ticker.eq_ignore_ascii_case(ticker) {
                            asset.uid = Some(uid.clone());
                        }
                    }
                    break;
                }
                Ok(None) => break,
                Err(e) if tbank::is_transport_error(&e) && !retried => {
                    eprintln!("[resolve_asset_uids] transport error, reconnecting: {e}");
                    tbank_state.reset_client();
                    retried = true;
                }
                Err(_) => break,
            }
        }
    }

    {
        let data = app_state.0.lock().unwrap();
        data.save(&app_handle)?;
    }

    let assets = {
        let data = app_state.0.lock().unwrap();
        data.assets.clone()
    };
    Ok(assets)
}

#[tauri::command(rename_all = "snake_case")]
async fn get_asset_prices(
    app_state: State<'_, AppState>,
    tbank_state: State<'_, TBankState>,
) -> Result<Vec<AssetPrice>, String> {
    // uid by asset_id
    let mut uid_map: std::collections::HashMap<u32, String> = std::collections::HashMap::new();
    {
        let data = app_state.0.lock().unwrap();
        for asset in data.assets.iter() {
            if let Some(uid) = &asset.uid {
                uid_map.insert(asset.id, uid.clone());
            }
        }
    }

    let mut need_fetch: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut result: Vec<AssetPrice> = Vec::new();

    for (asset_id, uid) in &uid_map {
        if let Some(price) = tbank_state.price_cache.get_fresh(uid) {
            result.push(AssetPrice {
                asset_id: *asset_id,
                price: Some(price),
                stale: false,
            });
        } else if let Some(price) = tbank_state.price_cache.get_stale(uid) {
            result.push(AssetPrice {
                asset_id: *asset_id,
                price: Some(price),
                stale: true,
            });
            need_fetch.insert(uid.clone());
        } else {
            need_fetch.insert(uid.clone());
        }
    }

    if !need_fetch.is_empty() {
        let token = {
            let data = app_state.0.lock().unwrap();
            data.tbank_token
                .clone()
                .ok_or_else(|| "T-Bank token not set".to_string())?
        };
        let client = tbank_state.ensure_client(&token).await?;

        let fetch_vec: Vec<String> = need_fetch.into_iter().collect();
        match client.last_prices_batch(&fetch_vec, &token).await {
            Ok(prices) => {
                for (uid, price) in prices.iter() {
                    tbank_state.price_cache.set(uid.clone(), *price);
                }
                let mut known_ids: std::collections::HashSet<u32> =
                    result.iter().map(|r| r.asset_id).collect();
                for (asset_id, uid) in uid_map.iter() {
                    if let Some(&price) = prices.get(uid) {
                        if known_ids.contains(asset_id) {
                            if let Some(r) = result.iter_mut().find(|r| r.asset_id == *asset_id) {
                                r.price = Some(price);
                                r.stale = false;
                            }
                        } else {
                            result.push(AssetPrice {
                                asset_id: *asset_id,
                                price: Some(price),
                                stale: false,
                            });
                            known_ids.insert(*asset_id);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("[get_asset_prices] batch fetch failed: {e}");
                if tbank::is_transport_error(&e) {
                    tbank_state.reset_client();
                }
            }
        }
    }

    Ok(result)
}

#[tauri::command(rename_all = "snake_case")]
fn create_asset(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    group_id: u32,
    ticker: String,
    amount: f64,
    buy_price: f64,
    uid: Option<String>,
) -> Result<Asset, String> {
    let mut data = state.0.lock().unwrap();

    let ticker_upper = ticker.to_uppercase();

    let existing_index = data.assets.iter().position(|a| {
        a.group_id == group_id && a.ticker == ticker_upper && a.source_id.is_none()
    });

    if let Some(idx) = existing_index {
        let added_cost = amount * buy_price;
        {
            let existing = data.assets.get_mut(idx).expect("index is valid");
            let current_cost = existing.amount * existing.buy_price;
            existing.amount += amount;
            existing.buy_price = (current_cost + added_cost) / existing.amount;

            if existing.uid.is_none() && uid.is_some() {
                existing.uid = uid;
            }
        }

        if let Some(group) = data.asset_groups.iter_mut().find(|g| g.id == group_id) {
            group.total_value += added_cost;
        }

        data.save(&app_handle)?;
        let merged = data.assets[idx].clone();
        return Ok(merged);
    }

    let new_id = data.assets.iter().map(|a| a.id).max().unwrap_or(0) + 1;
    let new_asset = Asset {
        id: new_id,
        group_id,
        source_id: None,
        ticker: ticker_upper,
        amount,
        buy_price,
        uid,
    };

    if let Some(group) = data.asset_groups.iter_mut().find(|g| g.id == group_id) {
        group.total_value += amount * buy_price;
    }

    data.assets.push(new_asset.clone());
    data.save(&app_handle)?;

    Ok(new_asset)
}

#[tauri::command(rename_all = "snake_case")]
fn create_strategy(app_handle: tauri::AppHandle, state: State<AppState>, name: String, color: String) -> Result<Strategy, String> {
    let mut data = state.0.lock().unwrap();

    let new_id = data.all_strategies.iter().map(|s| s.id).max().unwrap_or(0) + 1;
    let new_strategy = Strategy {
        id: new_id,
        name,
        color,
        distribution: std::collections::HashMap::new(),
    };

    data.all_strategies.push(new_strategy.clone());
    data.strategy = Some(new_strategy.clone());

    data.save(&app_handle)?;
    Ok(new_strategy)
}

#[tauri::command(rename_all = "snake_case")]
fn rename_strategy(app_handle: tauri::AppHandle, state: State<AppState>, id: u32, new_name: String) -> Result<(), String> {
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
    data.sources.retain(|s| s.strategy_id != id);

    if let Some(ref s) = data.strategy {
        if s.id == id {
            data.strategy = None;
        }
    }
    data.save(&app_handle)
}

#[tauri::command(rename_all = "snake_case")]
fn delete_asset_group(app_handle: tauri::AppHandle, state: State<AppState>, id: u32) -> Result<(), String> {
    let mut data = state.0.lock().unwrap();

    data.asset_groups.retain(|g| g.id != id);


    data.save(&app_handle)
}

#[tauri::command(rename_all = "snake_case")]
fn delete_asset(app_handle: tauri::AppHandle, state: State<AppState>, id: u32) -> Result<(), String> {
    let mut data = state.0.lock().unwrap();

    data.assets.retain(|a| a.id != id);

    data.save(&app_handle)?;

    Ok(())
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

#[tauri::command(rename_all = "snake_case")]
fn create_asset_group(
    app_handle: tauri::AppHandle,
    state: tauri::State<AppState>,
    strategy_id: u32,
    name: String,
    distribution: std::collections::HashMap<u32, f64>,
    new_group_percent: f64
) -> Result<AssetGroup, String> {
    let mut data = state.0.lock().unwrap();

    let new_group_id = data.asset_groups.iter().map(|g| g.id).max().unwrap_or(0) + 1;
    let new_group = AssetGroup {
        id: new_group_id,
        strategy_id,
        name: name.clone(),
        total_value: 0.0,
    };
    data.asset_groups.push(new_group.clone());

    let mut target_distribution = std::collections::HashMap::new();

    if let Some(strat) = data.all_strategies.iter_mut().find(|s| s.id == strategy_id) {
        strat.distribution = distribution;
        strat.distribution.insert(new_group_id, new_group_percent);
        target_distribution = strat.distribution.clone();
    }

    if let Some(ref mut current_strat) = data.strategy {
        if current_strat.id == strategy_id {
            current_strat.distribution = target_distribution;
        }
    }

    data.save(&app_handle)?;
    Ok(new_group)
}

#[tauri::command(rename_all = "snake_case")]
fn buy_more_asset(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    id: u32,
    added_amount: f64,
    execution_price: f64
) -> Result<Asset, String> {
    let mut data = state.0.lock().unwrap();

    // Find the asset in the vector
    if let Some(asset) = data.assets.iter_mut().find(|a| a.id == id) {
        let current_cost = asset.amount * asset.buy_price;
        let added_cost = added_amount * execution_price;

        asset.amount += added_amount;
        asset.buy_price = (current_cost + added_cost) / asset.amount;

        let updated = asset.clone();
        data.save(&app_handle)?;
        return Ok(updated);
    }

    Err("Asset not found in backend list".into())
}

#[tauri::command(rename_all = "snake_case")]
fn sell_asset(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    id: u32,
    sell_amount: f64,
    _execution_price: f64
) -> Result<Option<Asset>, String> {
    let mut data = state.0.lock().unwrap();
    let mut should_remove = false;
    let mut found = false;
    let mut updated_asset: Option<Asset> = None;

    if let Some(asset) = data.assets.iter_mut().find(|a| a.id == id) {
        found = true;
        if asset.amount <= sell_amount {
            should_remove = true;
        } else {
            asset.amount -= sell_amount;
            updated_asset = Some(asset.clone());
        }
    }

    if found {
        if should_remove {
            data.assets.retain(|a| a.id != id);
        }
        data.save(&app_handle)?;
        return Ok(updated_asset);
    }

    Err("Asset not found in backend".into())
}

#[tauri::command(rename_all = "snake_case")]
fn execute_source_deposit(
    app_handle: tauri::AppHandle,
    state: tauri::State<AppState>,
    source_id: u32,
    total_amount: f64,
    currency: String,
) -> Result<Vec<Asset>, String> {
    let mut data = state.0.lock().unwrap();

    let (strategy_id, _current_total_balance) = {
        if let Some(src) = data.sources.iter_mut().find(|s| s.id == source_id) {
            src.total_balance += total_amount;
            (src.strategy_id, src.total_balance)
        } else {
            return Err("Source not found".into());
        }
    };

    let distribution_rules = {
        if let Some(strat) = data.all_strategies.iter().find(|s| s.id == strategy_id) {
            strat.distribution.clone()
        } else {
            return Err("Active strategy investment plan not found".into());
        }
    };

    for (group_id, percent) in distribution_rules {
        if percent <= 0.0 { continue; }

        let allocated_cash = total_amount * (percent / 100.0);
        let ticker_upper = currency.to_uppercase();

        let mut asset_found = false;
        if let Some(existing_cash_asset) = data.assets.iter_mut().find(|a| {
            a.group_id == group_id
            && a.ticker == ticker_upper
            && a.source_id == Some(source_id)
        }) {
            existing_cash_asset.amount += allocated_cash;
            asset_found = true;
        }

        if !asset_found {
            let new_asset_id = data.assets.iter().map(|a| a.id).max().unwrap_or(0) + 1;
            data.assets.push(Asset {
                id: new_asset_id,
                group_id,
                source_id: Some(source_id),
                ticker: ticker_upper,
                amount: allocated_cash,
                buy_price: 1.0,
                uid: None,
            });
        }
    }

    data.save(&app_handle)?;

    Ok(data.assets.clone())
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let initial_data = AppData::load(&app.handle());
            app.manage(AppState(Mutex::new(initial_data)));
            app.manage(TBankState::new());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_data,
            set_tbank_token,
            search_instruments,
            resolve_asset_uids,
            get_asset_prices,
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
            sell_asset,
            execute_source_deposit
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
