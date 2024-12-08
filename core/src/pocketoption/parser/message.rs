use core::fmt;

use serde::Deserialize;
use serde_json::from_str;

use crate::pocketoption::{error::{PocketOptionError, PocketResult}, types::{base::{Auth, ChangeSymbol, SubscribeSymbol}, info::MessageInfo, order::{OpenOrder, SuccessCloseOrder, SuccessOpenOrder, UpdateClosedDeals, UpdateOpenedDeals}, success::SuccessAuth, update::{UpdateAssets, UpdateBalance, UpdateHistoryNew, UpdateStream}}};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum WebSocketMessage {
    UpdateStream(UpdateStream),
    UpdateHistoryNew(UpdateHistoryNew),
    UpdateAssets(UpdateAssets),
    UpdateBalance(UpdateBalance),
    OpenOrder(OpenOrder),
    SuccessAuth(SuccessAuth),
    UpdateClosedDeals(UpdateClosedDeals),
    SuccesscloseOrder(SuccessCloseOrder),
    SuccessopenOrder(SuccessOpenOrder),
    ChangeSymbol(ChangeSymbol),
    SubscribeSymbol(SubscribeSymbol),
    SuccessupdateBalance(UpdateBalance),
    UpdateOpenedDeals(UpdateOpenedDeals),
    Auth(Auth),


    None
}



impl WebSocketMessage {
    pub fn parse(data: impl ToString) -> PocketResult<Self> {
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

    pub fn parse_with_context(data: impl ToString, previous: &MessageInfo) -> PocketResult<Self> {
        let data = data.to_string();
        match previous 
        {
            MessageInfo::OpenOrder => {
                if let Ok(order) = from_str::<OpenOrder>(&data) {
                    return Ok(Self::OpenOrder(order));
                }
            },
            MessageInfo::UpdateStream => {
                if let Ok(stream) = from_str::<UpdateStream>(&data) {
                    return Ok(Self::UpdateStream(stream));
                }
            },
            MessageInfo::UpdateHistoryNew => {
                if let Ok(history) = from_str::<UpdateHistoryNew>(&data) {
                    return Ok(Self::UpdateHistoryNew(history));
                }
            },
            MessageInfo::UpdateAssets => {
                if let Ok(assets) = from_str::<UpdateAssets>(&data) {
                    return Ok(Self::UpdateAssets(assets));
                }
            },
            MessageInfo::UpdateBalance => {
                if let Ok(balance) = from_str::<UpdateBalance>(&data) {
                    return Ok(Self::UpdateBalance(balance));
                }
            },
            MessageInfo::SuccesscloseOrder => {
                if let Ok(order) = from_str::<SuccessCloseOrder>(&data) {
                    return Ok(Self::SuccesscloseOrder(order));
                }
            },
            MessageInfo::Auth => {
                if let Ok(auth) = from_str::<Auth>(&data) {
                    return Ok(Self::Auth(auth));
                }
            },
            MessageInfo::ChangeSymbol => {
                if let Ok(symbol) = from_str::<ChangeSymbol>(&data) {
                    return Ok(Self::ChangeSymbol(symbol));
                }
            },
            MessageInfo::SuccessupdateBalance => {
                if let Ok(balance) = from_str::<UpdateBalance>(&data) {
                    return Ok(Self::SuccessupdateBalance(balance));
                }
            },
            MessageInfo::SuccessupdatePending => {},
            MessageInfo::SubscribeSymbol => {
                if let Ok(symbol) = from_str::<SubscribeSymbol>(&data) {
                    return Ok(Self::SubscribeSymbol(symbol));
                }
            },
            MessageInfo::Successauth => {
                if let Ok(auth) = from_str::<SuccessAuth>(&data) {
                    return Ok(Self::SuccessAuth(auth));
                }
            },
            MessageInfo::UpdateOpenedDeals => {
                if let Ok(deals) = from_str::<UpdateOpenedDeals>(&data) {
                    return Ok(Self::UpdateOpenedDeals(deals))
                }
            },
            MessageInfo::UpdateClosedDeals => {
                if let Ok(deals) = from_str::<UpdateClosedDeals>(&data) {
                    return Ok(Self::UpdateClosedDeals(deals));
                }
            },
            MessageInfo::SuccessopenOrder => {
                if let Ok(order) = from_str::<SuccessOpenOrder>(&data) {
                    return Ok(Self::SuccessopenOrder(order));
                }
            },
            MessageInfo::UpdateCharts => {
                // TODO: Add this 
            },
            MessageInfo::None => todo!(),
        }
        Err(PocketOptionError::GeneralParsingError("Error ".to_string()))
    }

    pub fn info(&self) -> MessageInfo {
        match self {
            Self::UpdateStream(_) => MessageInfo::UpdateStream,
            Self::UpdateHistoryNew(_) => MessageInfo::UpdateHistoryNew,
            Self::UpdateAssets(_) => MessageInfo::UpdateAssets,
            Self::UpdateBalance(_) => MessageInfo::UpdateBalance,
            Self::OpenOrder(_) => MessageInfo::OpenOrder,
            Self::SuccessAuth(_) => MessageInfo::Successauth,
            Self::UpdateClosedDeals(_) => MessageInfo::UpdateClosedDeals,
            Self::SuccesscloseOrder(_) => MessageInfo::SuccesscloseOrder,
            Self::SuccessopenOrder(_) => MessageInfo::SuccessopenOrder,
            Self::ChangeSymbol(_) => MessageInfo::ChangeSymbol,
            Self::Auth(_) => MessageInfo::Auth,
            Self::SuccessupdateBalance(_) => MessageInfo::SuccessupdateBalance,
            Self::UpdateOpenedDeals(_) => MessageInfo::UpdateOpenedDeals,
            Self::SubscribeSymbol(_) => MessageInfo::SubscribeSymbol,
            Self::None => MessageInfo::None,
        }
    }
}

impl fmt::Display for WebSocketMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebSocketMessage::UpdateStream(update_stream) => write!(f, "{:?}", update_stream),
            WebSocketMessage::UpdateHistoryNew(update_history_new) => write!(f, "{:?}", update_history_new),
            WebSocketMessage::UpdateAssets(update_assets) => write!(f, "{:?}", update_assets),
            WebSocketMessage::UpdateBalance(update_balance) => write!(f, "{:?}", update_balance),
            WebSocketMessage::OpenOrder(open_order) => write!(f, "{:?}", open_order),
            WebSocketMessage::SuccessAuth(success_auth) => write!(f, "{:?}", success_auth),
            WebSocketMessage::UpdateClosedDeals(update_closed_deals) => write!(f, "{:?}", update_closed_deals),
            WebSocketMessage::SuccesscloseOrder(success_close_order) => write!(f, "{:?}", success_close_order),
            WebSocketMessage::SuccessopenOrder(success_open_order) => write!(f, "{:?}", success_open_order),
            WebSocketMessage::ChangeSymbol(change_symbol) => {
                write!(f, "42[{},{}]", serde_json::to_string(&MessageInfo::ChangeSymbol).map_err(|_| fmt::Error)?, serde_json::to_string(&change_symbol).map_err(|_| fmt::Error)?)
            },
            WebSocketMessage::SubscribeSymbol(subscribe_symbol) => write!(f, "{:?}", subscribe_symbol),
            WebSocketMessage::SuccessupdateBalance(update_balance) => write!(f, "{:?}", update_balance),
            WebSocketMessage::UpdateOpenedDeals(update_opened_deals) => write!(f, "{:?}", update_opened_deals),
            WebSocketMessage::Auth(auth) => write!(f, "{:?}", auth),
            WebSocketMessage::None => write!(f, "None")}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{error::Error, fs::File, io::{BufReader, Read}};

    use std::fs;
    use std::path::Path;

    fn get_files_in_directory(path: &str) -> Result<Vec<String>, std::io::Error> {
        let dir_path = Path::new(path);
        
        match fs::read_dir(dir_path) {
            Ok(entries) => {
                let mut file_names = Vec::new();
                
                for entry in entries {
                    let file_name = entry?.file_name().to_string_lossy().to_string();
                    file_names.push(format!("{path}/{file_name}"));
                }
                
                Ok(file_names)
            },
            Err(e) => Err(e),
        }
    }

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
        let history_new: WebSocketMessage = from_str(&content)?;
        dbg!(history_new);
        
        let mut assets_raw = File::open("tests/data.json")?;
        let mut content = String::new();
        assets_raw.read_to_string(&mut content)?;
        let assets_raw: WebSocketMessage = from_str(&content)?;
        dbg!(assets_raw);

        Ok(())
    }

    #[test]
    fn deep_test_descerialize_message() -> anyhow::Result<()> {
        let dirs = get_files_in_directory("tests")?;
        for dir in dirs {
            dbg!(&dir);
            let file = File::open(dir)?;

            let reader = BufReader::new(file);
            let _: WebSocketMessage = serde_json::from_reader(reader)?;
        }
        
        Ok(())
    }
}