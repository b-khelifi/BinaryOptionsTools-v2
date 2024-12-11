use tracing::debug;
use uuid::Uuid;

use crate::pocketoption::{error::{PocketOptionError, PocketResult}, parser::message::WebSocketMessage, types::{info::MessageInfo, order::{Deal, OpenOrder, SuccessOpenOrder}, user::UserRequest}, validators::{order_result_validator, order_validator}};

use super::{basic::WebSocketClient, listener::EventListener};

impl<T: EventListener> WebSocketClient<T> {
    pub async fn send_message(&self, msg: WebSocketMessage, response_type: MessageInfo, validator: impl Fn(&WebSocketMessage) -> bool + Send + Sync + 'static) -> PocketResult<WebSocketMessage> {
        let (request, reciever) = UserRequest::new(msg, response_type, validator);
        debug!("Sending request from user, expecting response: {}", request.response_type);
        self.sender.send(WebSocketMessage::UserRequest(Box::new(request))).await?;
        let resp = reciever.await?;
        if let WebSocketMessage::FailOpenOrder(fail) = resp {
            Err(PocketOptionError::from(fail))
        } else {
            Ok(resp)
        }
    }

    pub async fn buy(&self, asset: impl ToString, amount: f64, time: u32) -> PocketResult<(Uuid, SuccessOpenOrder)> {
        let order = OpenOrder::call(amount, asset.to_string(), time, self.demo as u32)?;
        let request_id = order.request_id;
        let res = self.send_message(WebSocketMessage::OpenOrder(order), MessageInfo::SuccessopenOrder, order_validator(request_id)).await?;
        if let WebSocketMessage::SuccessopenOrder(order) = res {
            debug!("Successfully opened buy trade!");
            return Ok((order.id, order))
        }
        Err(PocketOptionError::UnexpectedIncorrectWebSocketMessage(res.info()))
    }

    pub async fn sell(&self, asset: impl ToString, amount: f64, time: u32) -> PocketResult<(Uuid, SuccessOpenOrder)> {
        let order = OpenOrder::put(amount, asset.to_string(), time, self.demo as u32)?;
        let request_id = order.request_id;
        let res = self.send_message(WebSocketMessage::OpenOrder(order), MessageInfo::SuccessopenOrder, order_validator(request_id)).await?;
        if let WebSocketMessage::SuccessopenOrder(order) = res {
            debug!("Successfully opened sell trade!");
            return Ok((order.id, order))
        }
        Err(PocketOptionError::UnexpectedIncorrectWebSocketMessage(res.info()))
    }

    pub async fn check_results(&self, trade_id: Uuid) -> PocketResult<Deal> { // TODO: Add verification so it doesn't try to wait if no trade has been made with that id
        if let Some(trade) = self.data.get_closed_deals().await.iter().find(|d| d.id == trade_id) {
            return Ok(trade.clone())
        }
        debug!("Trade result not found in closed deals list, waiting for closing order to check.");
        let res = self.send_message(WebSocketMessage::None, MessageInfo::SuccesscloseOrder, order_result_validator(trade_id)).await?;
        if let WebSocketMessage::SuccesscloseOrder(order) = res {
            return order.deals.iter().find(|d| d.id == trade_id).cloned().ok_or(PocketOptionError::UnreachableError("Error finding correct trade".into()))
        }
        Err(PocketOptionError::UnexpectedIncorrectWebSocketMessage(res.info()))
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use futures_util::future::try_join_all;
    use tokio::{task::JoinHandle, time::sleep};
    use tracing::Level;

    use crate::pocketoption::{error::{PocketOptionError, PocketResult}, ws::listener::Handler, WebSocketClient};
    fn start_tracing() {
        tracing_subscriber::fmt().with_max_level(Level::DEBUG).pretty().init();
    }
    #[tokio::test]
    async fn test_websocket_client() -> anyhow::Result<()> {
        tracing_subscriber::fmt::init();
        let ssid = r#"42["auth",{"session":"looc69ct294h546o368s0lct7d","isDemo":1,"uid":87742848,"platform":2}]	"#;
        let demo = true;
        let client = WebSocketClient::<Handler>::new(ssid, demo).await?;
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
        start_tracing();
        let ssid = r#"42["auth",{"session":"looc69ct294h546o368s0lct7d","isDemo":1,"uid":87742848,"platform":2}]	"#;
        let demo = true;
        let client = Arc::new(WebSocketClient::<Handler>::new(ssid, demo).await?);
        // let mut threads = Vec::new();
        let symbols = include_str!("../../../tests/assets.txt").lines().collect::<Vec<&str>>();
        for chunk in symbols.chunks(20) {

            let futures = chunk.iter().map(|x| {
                let cl = client.clone();
                let x = *x;
                tokio::spawn(async move {
                    let res = cl.buy(x, 1.0, 60).await.inspect_err(|e| {dbg!(e);})?;
                    dbg!(&res);
                    let result = cl.check_results(res.0).await?;
                    dbg!("Trade result: {}", result.profit);    

                    Ok(())
                })
            }).collect::<Vec<JoinHandle<PocketResult<()>>>>();
            try_join_all(futures).await?;
        }
        Ok(())
    }

    #[tokio::test]
    #[should_panic]
    async fn test_force_error() {
        start_tracing();
        let ssid = r#"42["auth",{"session":"looc69ct294h546o368s0lct7d","isDemo":1,"uid":87742848,"platform":2}]	"#;
        let demo = true;
        let client = WebSocketClient::<Handler>::new(ssid, demo).await.unwrap();
        let mut loops = 0;
        while loops < 1000 {
            loops += 1;
            client.sell("EURUSD_otc", 20000.0, 60).await.unwrap();
        }
    }
    
    #[tokio::test]
    async fn test_check_win() -> anyhow::Result<()> {
        start_tracing();
        let ssid = r#"42["auth",{"session":"looc69ct294h546o368s0lct7d","isDemo":1,"uid":87742848,"platform":2}]	"#;
        let demo = true;
        let client = Arc::new(WebSocketClient::<Handler>::new(ssid, demo).await.unwrap());
        let mut test = 0;
        let mut checks = Vec::new();
        while test < 1000 {
            test += 1;
            if test % 100 == 0 {
                let res = client.sell("EURUSD_otc", 1.0, 120).await?;
                dbg!("Trade id: {}", res.0);
                let m_client = client.clone();
                let res: tokio::task::JoinHandle<Result<(), PocketOptionError>> = tokio::spawn(async move {
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