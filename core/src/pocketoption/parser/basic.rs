use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::pocketoption::{
    error::PocketResult, types::update::float_time, utils::basic::get_index,
};

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct UpdateStream {
    active: String,
    #[serde(with = "float_time")]
    time: DateTime<Utc>,
    value: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum AssetType {
    Stock,
    Currency,
    Commodity,
    Cryptocurrency,
    Index,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoadHistoryPeriod {
    pub asset: String,
    pub period: i64,
    pub time: i64,
    pub index: u64,
    pub offset: i64,
}

impl LoadHistoryPeriod {
    pub fn new(asset: impl ToString, time: i64, period: i64, offset: i64) -> PocketResult<Self> {
        Ok(LoadHistoryPeriod {
            asset: asset.to_string(),
            period,
            time,
            index: get_index()?,
            offset,
        })
    }
}
