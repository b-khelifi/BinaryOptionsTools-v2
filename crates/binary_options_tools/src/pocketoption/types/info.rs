use core::fmt;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use binary_options_tools_core::general::traits::MessageInformation;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum MessageInfo {
    OpenOrder,
    UpdateStream,
    UpdateHistoryNew,
    UpdateAssets,
    UpdateBalance,
    SuccesscloseOrder,
    Auth,
    ChangeSymbol,
    SuccessupdateBalance,
    SuccessupdatePending,
    Successauth,
    UpdateOpenedDeals,
    UpdateClosedDeals,
    SuccessopenOrder,
    UpdateCharts,
    SubscribeSymbol,
    LoadHistoryPeriod,
    FailopenOrder,
    GetCandles,
    OpenPendingOrder,
    SuccessopenPendingOrder,
    FailopenPendingOrder,
    None,
}

impl Display for MessageInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = serde_json::to_string(&self).map_err(|_| fmt::Error)?;
        write!(f, "{msg}")
    }
}

impl MessageInformation for MessageInfo {}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    #[test]
    fn test_parse_message_info() -> Result<(), Box<dyn Error>> {
        dbg!(serde_json::to_string(&MessageInfo::OpenOrder)?);
        Ok(())
    }
}
