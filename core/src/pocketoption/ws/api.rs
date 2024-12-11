use uuid::Uuid;

use crate::pocketoption::{error::{PocketOptionError, PocketResult}, parser::message::WebSocketMessage, types::{info::MessageInfo, order::{OpenOrder, SuccessOpenOrder}, user::UserRequest}, validators::order_validator};

use super::{basic::WebSocketClient, listener::EventListener};

impl<T: EventListener> WebSocketClient<T> {
    pub async fn send_message(&self, msg: WebSocketMessage, response_type: MessageInfo, validator: impl Fn(&WebSocketMessage) -> bool + Send + Sync + 'static) -> PocketResult<WebSocketMessage> {
        let (request, reciever) = UserRequest::new(msg, response_type, validator);
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
            return Ok((order.id, order))
        }
        Err(PocketOptionError::UnexpectedIncorrectWebSocketMessage(res.info()))
    }

    pub async fn sell(&self, asset: impl ToString, amount: f64, time: u32) -> PocketResult<(Uuid, SuccessOpenOrder)> {
        let order = OpenOrder::put(amount, asset.to_string(), time, self.demo as u32)?;
        let request_id = order.request_id;
        let res = self.send_message(WebSocketMessage::OpenOrder(order), MessageInfo::SuccessopenOrder, order_validator(request_id)).await?;
        if let WebSocketMessage::SuccessopenOrder(order) = res {
            return Ok((order.id, order))
        }
        Err(PocketOptionError::UnexpectedIncorrectWebSocketMessage(res.info()))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::time::sleep;

    use crate::pocketoption::{ws::listener::Handler, WebSocketClient};

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
    async fn test_all_otcs() -> anyhow::Result<()> {
        tracing_subscriber::fmt::init();
        let ssid = r#"42["auth",{"session":"looc69ct294h546o368s0lct7d","isDemo":1,"uid":87742848,"platform":2}]	"#;
        let demo = true;
        let client = WebSocketClient::<Handler>::new(ssid, demo).await?;
        // let mut threads = Vec::new();
        let symbols = include_str!("../../../tests/assets.txt").lines().filter(|l| l.ends_with("otc")).collect::<Vec<&str>>();
        for chunk in symbols.chunks(20) {
            for &symbol in chunk.iter() {
                let res = client.buy(symbol, 1.0, 60).await?;
                dbg!(res);    
            }
            dbg!("Sleeping...");
            sleep(Duration::from_secs(60)).await;
        }
        Ok(())
    }

    #[tokio::test]
    #[should_panic]
    async fn test_force_error() {
        let ssid = r#"42["auth",{"session":"looc69ct294h546o368s0lct7d","isDemo":1,"uid":87742848,"platform":2}]	"#;
        let demo = true;
        let client = WebSocketClient::<Handler>::new(ssid, demo).await.unwrap();
        let mut loops = 0;
        while loops < 1000 {
            loops += 1;
            client.sell("EURUSD_otc", 20000.0, 60).await.unwrap();
        }
    }
    

}