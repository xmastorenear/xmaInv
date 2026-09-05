use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Asset {
    pub id: u32,
    pub group_id: u32,
    pub source_id: Option<u32>,
    pub ticker: String,
    pub amount: f64,
    pub buy_price: f64,
    pub uid: Option<String>,
}