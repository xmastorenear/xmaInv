use crate::proto::tinkoff::{
    instruments_service_client::InstrumentsServiceClient,
    market_data_service_client::MarketDataServiceClient,
    FindInstrumentRequest, GetLastPricesRequest, GetLastPricesResponse, InstrumentShort,
    InstrumentStatus, LastPriceType, Quotation,
};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;
use tonic::Request;

const PROD_URL: &str = "https://invest-public-api.tbank.ru";

pub const PRICE_CACHE_TTL: Duration = Duration::from_secs(300);

const RU_CA_PEM: &str = include_str!("../certs/russian_ca_chain.pem");

#[derive(Clone)]
pub struct TBankClient {
    channel: Channel,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct InstrumentCard {
    pub ticker: String,
    pub name: String,
    pub class_code: String,
    pub instrument_type: String,
    pub currency: String,
    pub uid: String,
    pub price: Option<f64>,
}

pub struct TickerCache {
    map: Mutex<HashMap<String, InstrumentCard>>,
}

impl TickerCache {
    pub fn new() -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
        }
    }

    fn get(&self, query: &str) -> Option<InstrumentCard> {
        self.map.lock().unwrap().get(query).cloned()
    }

    fn set(&self, query: String, card: InstrumentCard) {
        self.map.lock().unwrap().insert(query, card);
    }
}

pub struct PriceCache {
    map: Mutex<HashMap<String, (f64, Instant)>>,
}

impl PriceCache {
    pub fn new() -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
        }
    }

    pub fn get_fresh(&self, uid: &str) -> Option<f64> {
        let map = self.map.lock().unwrap();
        let (price, ts) = map.get(uid)?;
        if ts.elapsed() <= PRICE_CACHE_TTL {
            Some(*price)
        } else {
            None
        }
    }

    pub fn get_stale(&self, uid: &str) -> Option<f64> {
        self.map.lock().unwrap().get(uid).map(|(price, _)| *price)
    }

    pub fn has_fresh(&self, uid: &str) -> bool {
        self.get_fresh(uid).is_some()
    }

    pub fn set(&self, uid: String, price: f64) {
        self.map
            .lock()
            .unwrap()
            .insert(uid, (price, Instant::now()));
    }
}

fn quotation_to_f64(q: &Quotation) -> f64 {
    q.units as f64 + (q.nano as f64) / 1_000_000_000.0
}

impl TBankClient {
    pub async fn connect(_token: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let tls = tonic::transport::ClientTlsConfig::new()
            .ca_certificate(tonic::transport::Certificate::from_pem(RU_CA_PEM));

        let endpoint = tonic::transport::Endpoint::new(PROD_URL.to_string())?
            .tls_config(tls)?
            .connect_timeout(std::time::Duration::from_secs(20));

        let channel = endpoint
            .connect()
            .await
            .map_err(|e| format!("T-Bank API unavailable (TLS/DNS/network): {e}"))?;
        Ok(Self { channel })
    }

    pub async fn search(
        &self,
        query: &str,
        token: &str,
        cache: &TickerCache,
    ) -> Result<Vec<InstrumentCard>, String> {
        if let Some(cached) = cache.get(query) {
            return Ok(vec![cached]);
        }

        let mut client = InstrumentsServiceClient::new(self.channel.clone());
        let req = FindInstrumentRequest {
            query: query.to_string(),
            instrument_kind: None,
            api_trade_available_flag: None,
        };

        let mut r = Request::new(req);
        add_auth(&mut r, token);

        let resp = client
            .find_instrument(r)
            .await
            .map_err(|e| format!("Search failed: {}", e))?;

        let instruments = resp.into_inner().instruments;

        let deduped = dedupe_by_ticker(instruments.iter().collect());

        let uids: Vec<String> = deduped.iter().map(|ins| ins.uid.clone()).collect();
        let prices = self.last_prices_batch(&uids, token).await.unwrap_or_default();

        let mut cards = Vec::with_capacity(deduped.len());
        for ins in deduped {
            let mut card = instrument_short_to_card(ins);
            card.price = prices.get(&ins.uid).copied();
            cache.set(ins.uid.clone(), card.clone());
            cache.set(ins.ticker.clone(), card.clone());
            cards.push(card);
        }

        Ok(cards)
    }

    pub async fn last_price(&self, uid: &str, token: &str) -> Result<f64, String> {
        let mut prices = self.last_prices_batch(&[uid.to_string()], token).await?;
        prices.remove(uid).ok_or_else(|| "No last price".to_string())
    }

    pub async fn last_prices_batch(
        &self,
        uids: &[String],
        token: &str,
    ) -> Result<HashMap<String, f64>, String> {
        let mut client = MarketDataServiceClient::new(self.channel.clone());
        let req = GetLastPricesRequest {
            instrument_id: uids.to_vec(),
            last_price_type: LastPriceType::LastPriceExchange.into(),
            instrument_status: Some(InstrumentStatus::Base.into()),
            ..Default::default()
        };

        let mut r = Request::new(req);
        add_auth(&mut r, token);

        let resp = client
            .get_last_prices(r)
            .await
            .map_err(|e| format!("Price failed: {}", e))?;

        let prices: GetLastPricesResponse = resp.into_inner();
        let mut result = HashMap::with_capacity(prices.last_prices.len());
        for lp in prices.last_prices {
            if let Some(price) = lp.price.as_ref() {
                result.insert(lp.instrument_uid, quotation_to_f64(price));
            }
        }
        Ok(result)
    }

    pub async fn find_uid_by_ticker(
        &self,
        ticker: &str,
        token: &str,
        cache: &TickerCache,
    ) -> Result<Option<String>, String> {
        let ticker_upper = ticker.trim().to_uppercase();
        if ticker_upper.is_empty() {
            return Ok(None);
        }

        if let Some(cached) = cache.get(&ticker_upper) {
            return Ok(Some(cached.uid));
        }

        let mut client = InstrumentsServiceClient::new(self.channel.clone());
        let req = FindInstrumentRequest {
            query: ticker_upper.clone(),
            instrument_kind: None,
            api_trade_available_flag: None,
        };
        let mut r = Request::new(req);
        add_auth(&mut r, token);

        let resp = client
            .find_instrument(r)
            .await
            .map_err(|e| format!("Find instrument failed: {}", e))?;

        let instruments = resp.into_inner().instruments;

        let exact: Vec<&InstrumentShort> = instruments
            .iter()
            .filter(|ins| ins.ticker.eq_ignore_ascii_case(&ticker_upper))
            .collect();

        let matches = dedupe_by_ticker(exact);
        if let Some(ins) = matches.into_iter().next() {
            let uid = ins.uid.clone();
            let card = instrument_short_to_card(ins);
            cache.set(ticker_upper.clone(), card.clone());
            cache.set(uid.clone(), card);
            return Ok(Some(uid));
        }

        Ok(None)
    }
}

fn code_rank(cc: &str) -> i32 {
    match cc.to_uppercase().as_str() {
        "TQBR" => 3,
        "SPBXM" => 2,
        _ => 1,
    }
}

fn dedupe_by_ticker(list: Vec<&InstrumentShort>) -> Vec<&InstrumentShort> {
    let mut sorted = list;
    sorted.sort_by(|a, b| {
        let api_a = a.api_trade_available_flag as u8;
        let api_b = b.api_trade_available_flag as u8;
        api_b.cmp(&api_a).then_with(|| {
            code_rank(&b.class_code).cmp(&code_rank(&a.class_code))
        })
    });

    let mut seen_keys: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut result: Vec<&InstrumentShort> = Vec::new();
    for ins in sorted {
        let key = ins.ticker.to_uppercase();
        if seen_keys.insert(key.clone()) {
            result.push(ins);
        }
    }
    result
}

pub fn is_transport_error(err: &str) -> bool {
    let lower = err.to_lowercase();
    lower.contains("transport error")
        || lower.contains("unavailable")
        || lower.contains("connection reset")
        || lower.contains("connection refused")
        || lower.contains("broken pipe")
}

fn instrument_short_to_card(ins: &InstrumentShort) -> InstrumentCard {
    InstrumentCard {
        ticker: ins.ticker.clone(),
        name: ins.name.clone(),
        class_code: ins.class_code.clone(),
        instrument_type: ins.instrument_type.clone(),
        currency: String::new(), // InstrumentShort has no currency — filled later via GetInstrumentBy
        uid: ins.uid.clone(),
        price: None,
    }
}

fn add_auth<T>(request: &mut tonic::Request<T>, token: &str) {
    if let Ok(metadata_value) = MetadataValue::try_from(format!("Bearer {}", token)) {
        request
            .metadata_mut()
            .insert("authorization", metadata_value);
    }
    // T-Bank Invest API requires the x-app-name header with the application name.
    if let Ok(app_name) = MetadataValue::try_from("xmainv") {
        request.metadata_mut().insert("x-app-name", app_name);
    }
}
