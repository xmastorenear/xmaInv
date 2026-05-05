use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Source {
    pub id: u32,
    pub strategy_id: u32,
    pub name: String,
    pub icon_url: String,
    pub total_balance: f64,
    pub profit_loss: f64,
}