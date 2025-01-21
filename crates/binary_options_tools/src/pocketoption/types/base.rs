use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSymbol {
    pub asset: String,
    pub period: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SubscribeSymbol(String);

impl ChangeSymbol {
    pub fn new(asset: String, period: i64) -> Self {
        Self { asset, period }
    }
}
