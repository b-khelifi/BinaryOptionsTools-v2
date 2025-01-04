use std::collections::HashMap;

use tracing::debug;
use uuid::Uuid;

use crate::pocketoption::{
    error::{PocketOptionError, PocketResult},
    parser::{basic::LoadHistoryPeriod, message::WebSocketMessage},
    types::{
        base::ChangeSymbol, info::MessageInfo, order::{Deal, OpenOrder}, update::{DataCandle, UpdateBalance}, user::PocketUser
    },
    validators::{candle_validator, history_validator, order_result_validator, order_validator},
};

use super::{basic::WebSocketClient, listener::EventListener, stream::StreamAsset};

impl<T: EventListener> WebSocketClient<T> {
    pub async fn send_message(
        &self,
        msg: WebSocketMessage,
        response_type: MessageInfo,
        validator: impl Fn(&WebSocketMessage) -> bool + Send + Sync + 'static,
    ) -> PocketResult<WebSocketMessage> {
        let (request, reciever) = PocketUser::new(msg, response_type, validator);
        debug!(
            "Sending request from user, expecting response: {}",
            request.info
        );
        self.sender
            .send(WebSocketMessage::UserRequest(Box::new(request)))
            .await?;
        let resp = reciever.await?;
        if let WebSocketMessage::FailOpenOrder(fail) = resp {
            Err(PocketOptionError::from(fail))
        } else {
            Ok(resp)
        }
    }

    pub async fn buy(
        &self,
        asset: impl ToString,
        amount: f64,
        time: u32,
    ) -> PocketResult<(Uuid, Deal)> {
        let order = OpenOrder::call(amount, asset.to_string(), time, self.ssid.demo() as u32)?;
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
        Err(PocketOptionError::UnexpectedIncorrectWebSocketMessage(
            res.info(),
        ))
    }

    pub async fn sell(
        &self,
        asset: impl ToString,
        amount: f64,
        time: u32,
    ) -> PocketResult<(Uuid, Deal)> {
        let order = OpenOrder::put(amount, asset.to_string(), time, self.ssid.demo() as u32)?;
        let request_id = order.request_id;
        let res = self
            .send_message(
                WebSocketMessage::OpenOrder(order),
                MessageInfo::SuccessopenOrder,
                order_validator(request_id),
            )
            .await?;
        if let WebSocketMessage::SuccessopenOrder(order) = res {
            debug!("Successfully opened sell trade!");
            return Ok((order.id, order));
        }
        Err(PocketOptionError::UnexpectedIncorrectWebSocketMessage(
            res.info(),
        ))
    }

    pub async fn check_results(&self, trade_id: Uuid) -> PocketResult<Deal> {
        // TODO: Add verification so it doesn't try to wait if no trade has been made with that id
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
                .ok_or(PocketOptionError::UnreachableError(
                    "Error finding correct trade".into(),
                ));
        }
        Err(PocketOptionError::UnexpectedIncorrectWebSocketMessage(
            res.info(),
        ))
    }

    pub async fn get_candles(
        &self,
        asset: impl ToString,
        period: i64,
        offset: i64,
    ) -> PocketResult<Vec<DataCandle>> {
        let time = self.data.get_server_time().await.div_euclid(period) * period;
        if time == 0 {
            return Err(PocketOptionError::GeneralParsingError(
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
        Err(PocketOptionError::UnexpectedIncorrectWebSocketMessage(
            res.info(),
        ))
    }

    pub async fn history(&self, asset: impl ToString, period: i64) -> PocketResult<Vec<DataCandle>> {
        let request = ChangeSymbol::new(asset.to_string(), period);
        let res = self.send_message(WebSocketMessage::ChangeSymbol(request), MessageInfo::UpdateHistoryNew, history_validator(asset.to_string(), period)).await?;
        if let WebSocketMessage::UpdateHistoryNew(history) = res {
            return Ok(history.candle_data())
        }
        Err(PocketOptionError::UnexpectedIncorrectWebSocketMessage(
            res.info(),
        ))
    }

    pub async fn get_closed_deals(&self) -> Vec<Deal> {
        self.data.get_closed_deals().await
    }

    pub async fn get_opened_deals(&self) -> Vec<Deal> {
        self.data.get_opened_deals().await
    }

    pub async fn get_balance(&self) -> UpdateBalance {
        self.data.get_balance().await
    }

    pub async fn get_payout(&self) -> HashMap<String, i32> {
        self.data.get_full_payout().await
    }

    pub async fn subscribe_symbol(&self, asset: impl ToString) -> PocketResult<StreamAsset> {
        let _ = self.history(asset.to_string(), 1).await?;
        debug!("Created StreamAsset instance.");
        Ok(self.data.add_stream(asset.to_string()).await)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::OpenOptions, sync::Arc, time::Duration};

    use futures_util::{future::{try_join3, try_join_all}, StreamExt};
    use tokio::{task::JoinHandle, time::sleep};
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer};

    use crate::pocketoption::{
        error::{PocketOptionError, PocketResult},
        ws::{listener::Handler, stream::StreamAsset},
        WebSocketClient,
    };
    fn start_tracing() -> anyhow::Result<()> {
        let error_logs = OpenOptions::new()
            .append(true)
            .create(true)
            .open("../logs/errors.log")?;

        tracing_subscriber::registry()
            // .with(filtered_layer)
            .with(
                // log-error file, to log the errors that arise
                fmt::layer()
                    .with_ansi(false)
                    .with_writer(error_logs)
                    .with_filter(LevelFilter::WARN),
            )
            .with(
                fmt::Layer::default()
                    .pretty()
                    .with_filter(LevelFilter::DEBUG),
            )
            .try_init()?;
        Ok(())
    }
    #[tokio::test]
    async fn test_websocket_client() -> anyhow::Result<()> {
        tracing_subscriber::fmt::init();
        let ssid = r#"42["auth",{"session":"t0mc6nefcv7ncr21g4fmtioidb","isDemo":1,"uid":90000798,"platform":2}]	"#;
        let client = WebSocketClient::<Handler>::new(ssid).await?;
        let mut test = 0;
        // let mut threads = Vec::new();
        while test < 1000 {
            test += 1;
            if test % 100 == 0 {
                let res = client.sell("EURUSD_otc", 1.0, 60).await?;
                dbg!(res);
            } else if test % 100 == 50 {
                let res = client.buy("#AAPL_otc", 1.0, 60).await?;
                dbg!(res);
            }
            sleep(Duration::from_millis(100)).await;
        }
        Ok(())
    }
    #[tokio::test]
    async fn test_all_trades() -> anyhow::Result<()> {
        start_tracing()?;
        let ssid = r#"42["auth",{"session":"t0mc6nefcv7ncr21g4fmtioidb","isDemo":1,"uid":90000798,"platform":2}]	"#;
        let client = Arc::new(WebSocketClient::<Handler>::new(ssid).await?);
        // let mut threads = Vec::new();
        let symbols = include_str!("../../../tests/assets.txt")
            .lines()
            .collect::<Vec<&str>>();
        for chunk in symbols.chunks(20) {
            let futures = chunk
                .iter()
                .map(|x| {
                    let cl = client.clone();
                    let x = *x;
                    tokio::spawn(async move {
                        let res = cl.buy(x, 1.0, 60).await.inspect_err(|e| {
                            dbg!(e);
                        })?;
                        dbg!(&res);
                        let result = cl.check_results(res.0).await?;
                        dbg!("Trade result: {}", result.profit);

                        Ok(())
                    })
                })
                .collect::<Vec<JoinHandle<PocketResult<()>>>>();
            try_join_all(futures).await?;
        }
        Ok(())
    }

    #[tokio::test]
    #[should_panic]
    async fn test_force_error() {
        start_tracing().unwrap();
        let ssid = r#"42["auth",{"session":"t0mc6nefcv7ncr21g4fmtioidb","isDemo":1,"uid":90000798,"platform":2}]	"#;
        let client = WebSocketClient::<Handler>::new(ssid).await.unwrap();
        let mut loops = 0;
        while loops < 1000 {
            loops += 1;
            client.sell("EURUSD_otc", 20000.0, 60).await.unwrap();
        }
    }

    #[tokio::test]
    #[should_panic]
    async fn test_incorrect_asset_name() {
        start_tracing().unwrap();
        let ssid = r#"42["auth",{"session":"t0mc6nefcv7ncr21g4fmtioidb","isDemo":1,"uid":90000798,"platform":2}]	"#;
        let client = WebSocketClient::<Handler>::new(ssid).await.unwrap();

        client.sell("EUReUSD_otc", 1.0, 60).await.unwrap();
    }

    #[tokio::test]
    async fn test_check_win() -> anyhow::Result<()> {
        start_tracing()?;
        let ssid = r#"42["auth",{"session":"t0mc6nefcv7ncr21g4fmtioidb","isDemo":1,"uid":90000798,"platform":2}]	"#;
        let client = Arc::new(WebSocketClient::<Handler>::new(ssid).await.unwrap());
        let mut test = 0;
        let mut checks = Vec::new();
        while test < 1000 {
            test += 1;
            if test % 100 == 0 {
                let res = client.sell("EURUSD_otc", 1.0, 300).await?;
                dbg!("Trade id: {}", res.0);
                let m_client = client.clone();
                let res: tokio::task::JoinHandle<Result<(), PocketOptionError>> =
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

    #[tokio::test]
    async fn test_get_candles() -> anyhow::Result<()> {
        start_tracing()?;
        let ssid = r#"42["auth",{"session":"t0mc6nefcv7ncr21g4fmtioidb","isDemo":1,"uid":90000798,"platform":2}]	"#;
        // time: 1733040000, offset: 540000, period: 3600
        let client = Arc::new(WebSocketClient::<Handler>::new(ssid).await.unwrap());
        sleep(Duration::from_secs(5)).await;
        let candles = client.get_candles("EURUSD_otc", 60, 6000).await?;
        dbg!(candles);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_closed_orders() -> anyhow::Result<()> {
        start_tracing()?;
        let ssid = r#"42["auth",{"session":"t0mc6nefcv7ncr21g4fmtioidb","isDemo":1,"uid":90000798,"platform":2}]	"#;
        let client = Arc::new(WebSocketClient::<Handler>::new(ssid).await?);
        // let file = File::options().append(true).open("tes")
        let mut ids = Vec::new();
        let mut tasks: Vec<JoinHandle<PocketResult<()>>> = Vec::new();
        for i in 0..20 {
            let (id, _) = client.sell("EURUSD_otc", 1.0, 5).await?;
            ids.push(id);
            let m_client = client.clone();
            tasks.push(tokio::spawn(async move {
                let result = m_client.check_results(id).await?;
                dbg!("Trade result: {}", result.profit);
                dbg!("Trade number {}", i);
                Ok(())
            }));
        }
        let original_orders = client.get_closed_deals().await;
        try_join_all(tasks).await?;
        let orders = client.get_closed_deals().await;
        println!("Number of closed deals: {}", original_orders.len());
        println!("Number of closed deals: {}", orders.len());

        for id in ids {
            orders
                .iter()
                .find(|o| o.id == id)
                .ok_or(PocketOptionError::GeneralParsingError(
                    "Expected at least one id to match".into(),
                ))?;
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_get_open_orders() -> anyhow::Result<()> {
        start_tracing()?;
        let ssid = r#"42["auth",{"session":"looc69ct294h546o368s0lct7d","isDemo":1,"uid":87742848,"platform":2}]	"#;
        let client = Arc::new(WebSocketClient::<Handler>::new(ssid).await?);
        let original_orders = client.get_opened_deals().await;
        // let file = File::options().append(true).open("tes")
        let mut ids = Vec::new();
        let mut tasks: Vec<JoinHandle<PocketResult<()>>> = Vec::new();
        for i in 0..20 {
            let (id, _) = client.sell("EURUSD_otc", 1.0, 30).await?;
            ids.push(id);
            let m_client = client.clone();
            tasks.push(tokio::spawn(async move {
                let result = m_client.check_results(id).await?;
                dbg!("Trade result: {}", result.profit);
                dbg!("Trade number {}", i);
                Ok(())
            }));
        }

        let orders = client.get_opened_deals().await;
        try_join_all(tasks).await?;
        let end_orders = client.get_opened_deals().await;
        println!("Number of open deals before: {}", original_orders.len());
        println!("Number of open deals during: {}", orders.len());
        println!("Number of open deals after: {}", end_orders.len());
        // for id in ids {
        //     orders.iter().find(|o| o.id == id).ok_or(PocketOptionError::GeneralParsingError("Expected at least one id to match".into()))?;
        // }
        Ok(())
    }

    #[tokio::test]
    async fn test_real_account() -> anyhow::Result<()> {
        start_tracing()?;
        let ssid = r#"42["auth",{"session":"a:4:{s:10:\"session_id\";s:32:\"b718d584d524ee1bac02ef2ad56bbcc1\";s:10:\"ip_address\";s:14:\"191.113.153.59\";s:10:\"user_agent\";s:120:\"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36 OPR/114.\";s:13:\"last_activity\";i:1734375340;}a7ae2d152460e813f196b3a01636c13a","isDemo":0,"uid":87742848,"platform":2}]	"#;
        let client = Arc::new(WebSocketClient::<Handler>::new(ssid).await?);
        sleep(Duration::from_secs(10)).await;
        dbg!(client.get_balance().await);
        let candles = client.get_candles("EURUSD_otc", 60, 3600).await?;
        dbg!(&candles);
        dbg!("Candles length: {}", candles.len()); // 4172
        Ok(())
    }

    #[tokio::test]
    async fn test_history() -> anyhow::Result<()> {
        start_tracing()?;
        let ssid = r#"42["auth",{"session":"looc69ct294h546o368s0lct7d","isDemo":1,"uid":87742848,"platform":2}]	"#;
        let client = Arc::new(WebSocketClient::<Handler>::new(ssid).await?);
        let res = client.history("EURUSD_otc", 1).await?;
        dbg!(res);
        Ok(())
    }


    #[tokio::test]
    async fn test_subscribe_symbol() -> anyhow::Result<()> {
        fn to_future(stream: StreamAsset, id: i32) -> JoinHandle<anyhow::Result<()>> {
            tokio::spawn(async move {
                while let Some(item) = stream.to_stream().next().await {
                    dbg!("StreamAsset n°{} data: \n{}", id, item?);
                }
                Ok(())
            })
        }
        // start_tracing()?;
        let ssid = r#"42["auth",{"session":"looc69ct294h546o368s0lct7d","isDemo":1,"uid":87742848,"platform":2}]	"#;
        let client = Arc::new(WebSocketClient::<Handler>::new(ssid).await?);
        let stream_asset1 = client.subscribe_symbol("EURUSD_otc").await?;
        let stream_asset2 = client.subscribe_symbol("#FB_otc").await?;
        let stream_asset3 = client.subscribe_symbol("YERUSD_otc").await?;
        
        let f1 = to_future(stream_asset1, 1);
        let f2 = to_future(stream_asset2, 2);
        let f3 = to_future(stream_asset3, 3);
        let _ = try_join3(f1, f2, f3).await?;
        Ok(())
    }

}
