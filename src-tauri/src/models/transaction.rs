use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    pub id: u32,
    pub source_id: u32,
    pub amount: f64,
    pub timestamp: String,
    pub description: String,
}