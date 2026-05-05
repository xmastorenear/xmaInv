use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AssetGroup {
    pub id: u32,
    pub strategy_id: u32,
    pub name: String,
    pub total_value: f64,
}