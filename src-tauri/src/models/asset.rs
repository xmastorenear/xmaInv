use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Asset {
    pub id: u32,
    pub group_id: u32,  // К какой группе относится (Крипта, Акции...)
    pub ticker: String, // Короткое имя: BTC, AAPL
    pub amount: f64,    // Количество на балансе
    pub buy_price: f64, // Средняя цена покупки
}