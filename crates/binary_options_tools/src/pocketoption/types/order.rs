use core::fmt;
use std::{collections::HashMap, hash::Hash};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use uuid::Uuid;

use crate::pocketoption::{
    error::PocketResult, parser::message::WebSocketMessage, utils::basic::get_index,
};

use super::update::{float_time, string_time};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Call, // Buy
    Put,  // Sell
}

#[derive(Clone, Debug)]
pub enum PocketMessageFail {
    Order(FailOpenOrder),
    Pending(FailOpenPendingOrder),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailOpenOrder {
    error: String,
    amount: f64,
    asset: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OpenOrder {
    asset: String,
    action: Action,
    amount: f64,
    is_demo: u32,
    option_type: u32,
    pub request_id: u64,
    time: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OpenPendingOrder {
    amount: f64,
    asset: String,
    #[serde(serialize_with = "serialize_action")]
    command: Action,
    min_payout: i64,
    open_price: f64,
    #[serde(with = "string_time")]
    open_time: DateTime<Utc>,
    open_type: i32,
    time_frame: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SuccessOpenPendingOrder {
    data: SuccessOpenPendingOrderData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SuccessOpenPendingOrderData {
    ticket: Uuid,
    open_type: i32,
    amount: f64,
    symbol: String,
    #[serde(with = "string_time")]
    open_time: DateTime<Utc>,
    open_price: f64,
    time_frame: i64,
    min_payout: i64,
    command: i64,
    #[serde(with = "string_time")]
    date_created: DateTime<Utc>,
    id: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FailOpenPendingOrder {
    data: String,
    error: String,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateClosedDeals(pub Vec<Deal>);

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct SuccessCloseOrder {
    pub profit: f64,
    pub deals: Vec<Deal>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct UpdateOpenedDeals(pub Vec<Deal>);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Deal {
    pub id: Uuid,
    pub open_time: String,
    pub close_time: String,
    #[serde(with = "float_time")]
    pub open_timestamp: DateTime<Utc>,
    #[serde(with = "float_time")]
    pub close_timestamp: DateTime<Utc>,
    pub refund_time: Option<Value>,
    pub refund_timestamp: Option<Value>,
    pub uid: u64,
    pub request_id: Option<u64>,
    pub amount: f64,
    pub profit: f64,
    pub percent_profit: i32,
    pub percent_loss: i32,
    pub open_price: f64,
    pub close_price: f64,
    pub command: i32,
    pub asset: String,
    pub is_demo: u32,
    pub copy_ticket: String,
    pub open_ms: i32,
    pub close_ms: Option<i32>,
    pub option_type: i32,
    pub is_rollover: Option<bool>,
    pub is_copy_signal: Option<bool>,
    #[serde(rename = "isAI")]
    pub is_ai: Option<bool>,
    pub currency: String,
    pub amount_usd: Option<f64>,
    #[serde(rename = "amountUSD")]
    pub amount_usd2: Option<f64>,
}

impl Hash for Deal {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.uid.hash(state);
    }
}

impl Eq for Deal {}

impl OpenOrder {
    pub fn new(
        amount: f64,
        asset: String,
        action: Action,
        duration: u32,
        demo: u32,
    ) -> PocketResult<Self> {
        Ok(Self {
            amount,
            asset,
            action,
            is_demo: demo,
            option_type: 100, // FIXME: Check why it always is 100
            request_id: get_index()?,
            time: duration,
        })
    }

    pub fn put(amount: f64, asset: String, duration: u32, demo: u32) -> PocketResult<Self> {
        Self::new(amount, asset, Action::Put, duration, demo)
    }

    pub fn call(amount: f64, asset: String, duration: u32, demo: u32) -> PocketResult<Self> {
        Self::new(amount, asset, Action::Call, duration, demo)
    }
}

impl OpenPendingOrder {
    // pub fn new(amount: f64, asset: String, command: i64, min_payout: i64, open_price: f64, ) -> Self {
    //     Self { amount: (), asset: (), command: (), min_payout: (), open_price: (), open_time: (), open_type: (), time_frame: () }
    // }
}

impl fmt::Display for FailOpenOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Error: {}", self.error)?;
        writeln!(f, "Max Allowed requests: {}", self.amount)?;
        writeln!(f, "Error for asset: {}", self.asset)
    }
}

impl fmt::Display for FailOpenPendingOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Error: {}", self.error)?;
        writeln!(f, "Extra data: {:?}", self.extra)
    }
}

impl fmt::Display for PocketMessageFail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Order(order) => order.fmt(f),
            Self::Pending(order) => order.fmt(f),
        }
    }
}

impl From<PocketMessageFail> for WebSocketMessage {
    fn from(value: PocketMessageFail) -> Self {
        match value {
            PocketMessageFail::Order(order) => Self::FailOpenOrder(order),
            PocketMessageFail::Pending(pending) => Self::FailOpenPendingOrder(pending),
        }
    }
}

impl FailOpenOrder {
    pub fn new(error: impl ToString, amount: f64, asset: impl ToString) -> Self {
        Self {
            error: error.to_string(),
            amount,
            asset: asset.to_string(),
        }
    }
}

impl std::cmp::PartialEq<Uuid> for Deal {
    fn eq(&self, other: &Uuid) -> bool {
        &self.id == other
    }
}

pub fn serialize_action<S>(action: &Action, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match action {
        Action::Call => 0.serialize(serializer),
        Action::Put => 1.serialize(serializer),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        fs::{File, read_to_string},
        io::BufReader,
    };

    use crate::pocketoption::{parser::message::WebSocketMessage, types::info::MessageInfo};

    use super::*;

    #[test]
    fn test_descerialize_closed_deals() -> Result<(), Box<dyn Error>> {
        let history_raw = File::open("tests/update_closed_deals.json")?;
        let bufreader = BufReader::new(history_raw);
        let deals: UpdateClosedDeals = serde_json::from_reader(bufreader)?;
        let deals2 = WebSocketMessage::parse_with_context(
            read_to_string("tests/update_closed_deals.json")?,
            &MessageInfo::UpdateClosedDeals,
        );
        if let WebSocketMessage::UpdateClosedDeals(d) = deals2 {
            assert_eq!(d, deals);
        } else {
            panic!("WebSocketMessage should be UpdateClosedDeals variant")
        }

        Ok(())
    }
    #[test]
    fn test_descerialize_close_order() -> Result<(), Box<dyn Error>> {
        let history_raw = File::open("tests/update_close_order.json")?;
        let bufreader = BufReader::new(history_raw);
        let deals: SuccessCloseOrder = serde_json::from_reader(bufreader)?;
        let deals2 = WebSocketMessage::parse_with_context(
            read_to_string("tests/update_close_order.json")?,
            &MessageInfo::SuccesscloseOrder,
        );
        if let WebSocketMessage::SuccesscloseOrder(d) = deals2 {
            assert_eq!(d, deals);
        } else {
            panic!("WebSocketMessage should be UpdateClosedDeals variant")
        }
        Ok(())
    }

    #[test]
    fn test_descerialize_open_order() -> Result<(), Box<dyn Error>> {
        let order_raw = File::open("tests/success_open_order.json")?;
        let bufreader = BufReader::new(order_raw);
        let order: Deal = serde_json::from_reader(bufreader)?;
        dbg!(order);
        Ok(())
    }

    #[test]
    fn test_descerialize_update_opened_deals() -> anyhow::Result<()> {
        let order_raw = File::open("tests/update_opened_deals.json")?;
        let bufreader = BufReader::new(order_raw);
        let order: UpdateOpenedDeals = serde_json::from_reader(bufreader)?;
        dbg!(order);
        Ok(())
    }
}
