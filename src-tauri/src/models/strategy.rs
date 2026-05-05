use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Strategy {
    pub id: u32,
    pub name: String,
    pub color: String,
}