use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpenOrder {
    action: String,
    amount: f64,
    is_demo: u32,
    option_type: u32,
    request_id: u64,
    time: u32
}