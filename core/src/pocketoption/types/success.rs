use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize, Serialize)]
pub struct SuccessAuth {
    id: String
}