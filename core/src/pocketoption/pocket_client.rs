use std::{collections::HashMap, time::Duration};

use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::{
    error::{BinaryOptionsResult, BinaryOptionsToolsError},
    general::{client::WebSocketClient, types::Data},
    pocketoption::{
        parser::basic::LoadHistoryPeriod,
        validators::{candle_validator, order_result_validator},
        ws::ssid::Ssid,
    },
};

use super::{
    error::PocketOptionError,
    parser::message::WebSocketMessage,
    types::{
        base::ChangeSymbol,
        callback::PocketCallback,
        data_v2::PocketData,
        info::MessageInfo,
        order::{Action, Deal, OpenOrder},
        update::{DataCandle, UpdateBalance},
    },
    validators::{history_validator, order_validator},
    ws::{connect::PocketConnect, listener::Handler, stream::StreamAsset},
};

/// Class to connect automatically to Pocket Option's quick trade passing a valid SSID
pub type PocketOption =
    WebSocketClient<WebSocketMessage, Handler, PocketConnect, Ssid, PocketData, PocketCallback>;

impl PocketOption {
    pub async fn new(ssid: impl ToString) -> BinaryOptionsResult<Self> {
        let ssid = Ssid::parse(ssid)?;
        let data = Data::new(PocketData::default());
        let handler = Handler::new(ssid.clone());
        let timeout = Duration::from_millis(500);
        let callback = PocketCallback;
        let client = WebSocketClient::init(
            ssid,
            PocketConnect {},
            data,
            handler,
            timeout,
            Some(callback),
        )
        .await?;
        println!("Initialized!");
        Ok(client)
    }

    pub async fn trade(
        &self,
        asset: impl ToString,
        action: Action,
        amount: f64,
        time: u32,
    ) -> BinaryOptionsResult<(Uuid, Deal)> {
        let order = OpenOrder::new(
            amount,
            asset.to_string(),
            action,
            time,
            self.credentials.demo() as u32,
        )?;
        let request_id = order.request_id;
        let res = self
            .send_message(
                WebSocketMessage::OpenOrder(order),
                MessageInfo::SuccessopenOrder,
                order_validator(request_id),
            )
            .await?;
        if let WebSocketMessage::SuccessopenOrder(order) = res {
            debug!("Successfully opened buy trade!");
            return Ok((order.id, order));
        }
        Err(PocketOptionError::UnexpectedIncorrectWebSocketMessage(res.info()).into())
    }

    pub async fn buy(
        &self,
        asset: impl ToString,
        amount: f64,
        time: u32,
    ) -> BinaryOptionsResult<(Uuid, Deal)> {
        info!(target: "Buy", "Placing a buy trade for asset '{}', with amount '{}' and time '{}'", asset.to_string(), amount, time);
        self.trade(asset, Action::Call, amount, time).await
    }

    pub async fn sell(
        &self,
        asset: impl ToString,
        amount: f64,
        time: u32,
    ) -> BinaryOptionsResult<(Uuid, Deal)> {
        info!(target: "Sell", "Placing a sell trade for asset '{}', with amount '{}' and time '{}'", asset.to_string(), amount, time);
        self.trade(asset, Action::Put, amount, time).await
    }

    pub async fn check_results(&self, trade_id: Uuid) -> BinaryOptionsResult<Deal> {
        // TODO: Add verification so it doesn't try to wait if no trade has been made with that id
        info!(target: "CheckResults", "Checking results for trade of id {}", trade_id);
        if let Some(trade) = self
            .data
            .get_closed_deals()
            .await
            .iter()
            .find(|d| d.id == trade_id)
        {
            return Ok(trade.clone());
        }
        debug!("Trade result not found in closed deals list, waiting for closing order to check.");
        if !self
            .data
            .get_opened_deals()
            .await
            .iter()
            .any(|d| d == &trade_id)
        {
            warn!("No opened trade with the given uuid please check if you are passing the correct id");
            return Err(BinaryOptionsToolsError::Unallowed("Couldn't check result for a deal that is not in the list of opened trades nor closed trades.".into()));
        }
        let res = self
            .send_message(
                WebSocketMessage::None,
                MessageInfo::SuccesscloseOrder,
                order_result_validator(trade_id),
            )
            .await?;
        if let WebSocketMessage::SuccesscloseOrder(order) = res {
            return order
                .deals
                .iter()
                .find(|d| d.id == trade_id)
                .cloned()
                .ok_or(
                    PocketOptionError::UnreachableError("Error finding correct trade".into())
                        .into(),
                );
        }
        Err(PocketOptionError::UnexpectedIncorrectWebSocketMessage(res.info()).into())
    }

    pub async fn get_candles(
        &self,
        asset: impl ToString,
        period: i64,
        offset: i64,
    ) -> BinaryOptionsResult<Vec<DataCandle>> {
        info!(target: "GetCandles", "Retrieving candles for asset '{}' with period of '{}' and offset of '{}'", asset.to_string(), period, offset);
        let time = self.data.get_server_time().await.div_euclid(period) * period;
        if time == 0 {
            return Err(BinaryOptionsToolsError::GeneralParsingError(
                "Server time is invalid.".to_string(),
            ));
        }
        let request = LoadHistoryPeriod::new(asset.to_string(), time, period, offset)?;
        let index = request.index;
        debug!(
            "Sent get candles message, message: {:?}",
            WebSocketMessage::GetCandles(request).to_string()
        );
        let request = LoadHistoryPeriod::new(asset.to_string(), time, period, offset)?;
        let res = self
            .send_message(
                WebSocketMessage::GetCandles(request),
                MessageInfo::LoadHistoryPeriod,
                candle_validator(index),
            )
            .await?;
        if let WebSocketMessage::LoadHistoryPeriod(history) = res {
            return Ok(history.candle_data());
        }
        Err(PocketOptionError::UnexpectedIncorrectWebSocketMessage(res.info()).into())
    }

    pub async fn history(
        &self,
        asset: impl ToString,
        period: i64,
    ) -> BinaryOptionsResult<Vec<DataCandle>> {
        info!(target: "History", "Retrieving candles for asset '{}' with period of '{}'", asset.to_string(), period);

        let request = ChangeSymbol::new(asset.to_string(), period);
        let res = self
            .send_message(
                WebSocketMessage::ChangeSymbol(request),
                MessageInfo::UpdateHistoryNew,
                history_validator(asset.to_string(), period),
            )
            .await?;
        if let WebSocketMessage::UpdateHistoryNew(history) = res {
            return Ok(history.candle_data());
        }
        Err(PocketOptionError::UnexpectedIncorrectWebSocketMessage(res.info()).into())
    }

    pub async fn get_closed_deals(&self) -> Vec<Deal> {
        info!(target: "GetClosedDeals", "Retrieving list of closed deals");
        self.data.get_closed_deals().await
    }

    pub async fn clear_closed_deals(&self) {
        info!(target: "ClearClosedDeals", "Clearing list of closed deals");
        self.data.clean_closed_deals().await
    }

    pub async fn get_opened_deals(&self) -> Vec<Deal> {
        info!(target: "GetOpenDeals", "Retrieving list of open deals");
        self.data.get_opened_deals().await
    }

    pub async fn get_balance(&self) -> UpdateBalance {
        info!(target: "GetBalance", "Retrieving account balance");
        self.data.get_balance().await
    }

    pub async fn get_payout(&self) -> HashMap<String, i32> {
        info!(target: "GetPayout", "Retrieving payout for all the assets");
        self.data.get_full_payout().await
    }

    pub async fn subscribe_symbol(&self, asset: impl ToString) -> BinaryOptionsResult<StreamAsset> {
        info!(target: "SubscribeSymbol", "Subscribing to asset '{}'", asset.to_string());
        let _ = self.history(asset.to_string(), 1).await?;
        debug!("Created StreamAsset instance.");
        Ok(self.data.add_stream(asset.to_string()).await)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use futures_util::{
        future::{try_join3, try_join_all},
        StreamExt,
    };
    use tokio::{task::JoinHandle, time::sleep};
    use tracing::error;

    use crate::utils::tracing::start_tracing;

    use super::*;

    #[tokio::test]
    async fn test_pocket_option() -> anyhow::Result<()> {
        // start_tracing()?;
        let ssid = r#"42["auth",{"session":"looc69ct294h546o368s0lct7d","isDemo":1,"uid":87742848,"platform":2}]	"#;
        let api = PocketOption::new(ssid).await?;
        // let mut loops = 0;
        // while loops < 100 {
        //     loops += 1;
        //     sleep(Duration::from_millis(100)).await;
        // }
        let now = Instant::now();
        for i in 0..100 {
            let _ = api.buy("EURUSD_otc", 1.0, 60).await?;
            println!("Loop n°{i}, Elapsed time: {:.8?} ms", now.elapsed());
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_subscribe_symbol_v2() -> anyhow::Result<()> {
        start_tracing()?;
        fn to_future(stream: StreamAsset, id: i32) -> JoinHandle<anyhow::Result<()>> {
            tokio::spawn(async move {
                while let Some(item) = stream.to_stream().next().await {
                    error!("StreamAsset n°{}, price: {}", id, item?.close);
                }
                Ok(())
            })
        }
        // start_tracing()?;
        let ssid = r#"42["auth",{"session":"looc69ct294h546o368s0lct7d","isDemo":1,"uid":87742848,"platform":2}]	"#;
        let client = PocketOption::new(ssid).await?;
        let stream_asset1 = client.subscribe_symbol("EURUSD_otc").await?;
        let stream_asset2 = client.subscribe_symbol("#FB_otc").await?;
        let stream_asset3 = client.subscribe_symbol("YERUSD_otc").await?;

        let f1 = to_future(stream_asset1, 1);
        let f2 = to_future(stream_asset2, 2);
        let f3 = to_future(stream_asset3, 3);
        let _ = try_join3(f1, f2, f3).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_payout() -> anyhow::Result<()> {
        let ssid = r#"42["auth",{"session":"looc69ct294h546o368s0lct7d","isDemo":1,"uid":87742848,"platform":2}]	"#;
        let api = PocketOption::new(ssid).await?;
        tokio::time::sleep(Duration::from_secs(5)).await;
        dbg!(api.get_payout().await);
        Ok(())
    }

    #[tokio::test]
    async fn test_check_win() -> anyhow::Result<()> {
        start_tracing()?;
        let ssid = r#"42["auth",{"session":"t0mc6nefcv7ncr21g4fmtioidb","isDemo":1,"uid":90000798,"platform":2}]	"#;
        let client = PocketOption::new(ssid).await.unwrap();
        let mut test = 0;
        let mut checks = Vec::new();
        while test < 1000 {
            test += 1;
            if test % 100 == 0 {
                let res = client.sell("EURUSD_otc", 1.0, 15).await?;
                dbg!("Trade id: {}", res.0);
                let m_client = client.clone();
                let res: tokio::task::JoinHandle<Result<(), BinaryOptionsToolsError>> =
                    tokio::spawn(async move {
                        let result = m_client.check_results(res.0).await?;
                        dbg!("Trade result: {}", result.profit);
                        Ok(())
                    });
                checks.push(res);
            } else if test % 100 == 50 {
                let res = &client.buy("#AAPL_otc", 1.0, 5).await?;
                dbg!(res);
            }
            sleep(Duration::from_millis(100)).await;
        }
        try_join_all(checks).await?;
        Ok(())
    }
}
