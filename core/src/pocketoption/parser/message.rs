use serde::Deserialize;
use serde_json::from_str;

use crate::pocketoption::{error::PocketOptionError, types::update::{UpdateAssets, UpdateBalance, UpdateHistoryNew, UpdateStream}};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum WebSocketMessage {
    UpdateStream(UpdateStream),
    UpdateHistoryNew(UpdateHistoryNew),
    UpdateAssets(UpdateAssets),
    UpdateBalance(UpdateBalance),
}



impl WebSocketMessage {
    pub fn parse(data: impl ToString) -> Result<Self, PocketOptionError> {
        let data = data.to_string();
        let message: Result<Self, serde_json::Error> = from_str(&data);
        match message {
            Ok(message) => Ok(message),
            Err(e) => {
                if let Ok(assets) = from_str::<UpdateAssets>(&data) {
                    return Ok(Self::UpdateAssets(assets));
                }
                if let Ok(history) = from_str::<UpdateHistoryNew>(&data) {
                    return Ok(Self::UpdateHistoryNew(history));
                }
                if let Ok(stream) = from_str::<UpdateStream>(&data) {
                    return Ok(Self::UpdateStream(stream));
                }
                if let Ok(balance) = from_str::<UpdateBalance>(&data) {
                    return Ok(Self::UpdateBalance(balance));
                }
                Err(e.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{error::Error, fs::File, io::{BufReader, Read}};

    #[test]
    fn test_descerialize_message() -> Result<(), Box<dyn Error>> {
        let tests = [
            r#"[["AUS200_otc",1732830010,6436.06]]"#,
            r#"[["AUS200_otc",1732830108.205,6435.96]]"#,
            r#"[["AEDCNY_otc",1732829668.352,1.89817]]"#,
            r#"[["CADJPY_otc",1732830170.793,109.442]]"#,
        ];
        for item in tests.iter() {
            let val = WebSocketMessage::parse(item)?;
            dbg!(&val);
        }      
        let mut history_raw = File::open("tests/update_history_new.txt")?;
        let mut content = String::new();
        history_raw.read_to_string(&mut content)?;
        let history_new: WebSocketMessage = WebSocketMessage::parse(content)?;
        dbg!(history_new);
        
        let mut assets_raw = File::open("tests/data.txt")?;
        let mut content = String::new();
        assets_raw.read_to_string(&mut content)?;
        let assets_raw = WebSocketMessage::parse(content)?;
        dbg!(assets_raw);

        Ok(())
    }
}