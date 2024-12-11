use core::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::pocketoption::{error::PocketResult, utils::basic::get_index};

use super::update::float_time;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Call, // Buy
    Put // Sell
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailOpenOrder {
    error: String,
    amount: f64,
    asset: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpenOrder {
    asset: String,
    action: Action,
    amount: f64,
    is_demo: u32,
    option_type: u32,
    pub request_id: u64,
    time: u32
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateClosedDeals(pub Vec<Deal>);

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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
    pub amount: u64,
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
    pub is_AI: Option<bool>,
    pub currency: String,
    pub amount_usd: Option<f64>,
    pub amount_USD: f64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuccessOpenOrder {
    pub id: Uuid,
    open_time: String,
    close_time: String,
    #[serde(with = "float_time")]
    open_timestamp: DateTime<Utc>,
    #[serde(with = "float_time")]
    close_timestamp: DateTime<Utc>,
    uid: u64,
    is_demo: u32,
    amount: f64,
    profit: f64,
    percent_profit: i32,
    percent_loss: i32,
    open_price: f64,
    copy_ticket: String,
    close_price: f64,
    command: i32,
    asset: String,
    pub request_id: u64,
    open_ms: i32,
    option_type: i32,
    is_copy_signal: bool,
    currency: String,
}

impl OpenOrder {
    pub fn new(amount: f64, asset: String, action: Action, duration: u32, demo: u32) -> PocketResult<Self> {
        Ok(Self {
            amount,
            asset,
            action,
            is_demo: demo,
            option_type: 100, // FIXME: Check why it always is 100
            request_id: get_index()?,
            time: duration
        })
    }

    pub fn put(amount: f64, asset: String, duration: u32, demo: u32) -> PocketResult<Self> {
        Self::new(amount, asset, Action::Put, duration, demo)
    }

    pub fn call(amount: f64, asset: String, duration: u32, demo: u32) -> PocketResult<Self> {
        Self::new(amount, asset, Action::Call, duration, demo)
    }
}

impl fmt::Display for FailOpenOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Error: {}", self.error);
        writeln!(f, "Max Allowed requests: {}", self.amount);
        writeln!(f, "Error for asset: {}", self.asset)
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, fs::{read_to_string, File}, io::BufReader};

    use crate::pocketoption::{parser::message::WebSocketMessage, types::info::MessageInfo};

    use super::*;

    #[test]
    fn test_descerialize_closed_deals() -> Result<(), Box<dyn Error>> {
        let history_raw = File::open("tests/update_closed_deals.json")?;
        let bufreader = BufReader::new(history_raw);
        let deals: UpdateClosedDeals = serde_json::from_reader(bufreader)?;
        let deals2 = WebSocketMessage::parse_with_context(read_to_string("tests/update_closed_deals.json")?, &MessageInfo::UpdateClosedDeals)?;
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
        let deals2 = WebSocketMessage::parse_with_context(read_to_string("tests/update_close_order.json")?, &MessageInfo::SuccesscloseOrder)?;
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
        let order: SuccessOpenOrder = serde_json::from_reader(bufreader)?;
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