use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSymbol {
    asset: String,
    period: i64
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Auth {
    session: String,
    is_demo: u32,
    uid: u64,
    platform: u32
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SubscribeSymbol(String);