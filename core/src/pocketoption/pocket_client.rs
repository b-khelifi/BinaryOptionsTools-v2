// use std::{ops::Deref, time::Duration};

// use tracing::debug;
// use uuid::Uuid;

// use crate::{
//     error::BinaryOptionsResult,
//     general::{client::WebSocketClient, types::Data},
//     pocketoption::ws::ssid::Ssid,
// };

// use super::{
//     error::PocketOptionError,
//     parser::message::WebSocketMessage,
//     types::{
//         data_v2::PocketData,
//         info::MessageInfo,
//         order::{Deal, OpenOrder},
//     },
//     validators::order_validator,
//     ws::{connect::PocketConnect, listener::Handler, ssid::PocketCreds},
// };

// #[derive(Clone)]
// pub struct PocketOption {
//     client: WebSocketClient<WebSocketMessage, Handler, PocketConnect, PocketCreds, PocketData>,
// }

// impl Deref for PocketOption {
//     type Target =
//         WebSocketClient<WebSocketMessage, Handler, PocketConnect, PocketCreds, PocketData>;

//     fn deref(&self) -> &Self::Target {
//         &self.client
//     }
// }

// impl PocketOption {
//     pub async fn new(ssid: impl ToString, demo: bool) -> BinaryOptionsResult<Self> {
//         let ssid = Ssid::parse(ssid)?;
//         let creds = PocketCreds {
//             ssid: ssid.clone(),
//             demo,
//         };
//         let data = Data::new(PocketData::default());
//         let handler = Handler::new(ssid);
//         let timeout = Duration::from_millis(500);
//         let client = WebSocketClient::init(creds, PocketConnect {}, data, handler, timeout).await?;
//         Ok(Self { client })
//     }

//     pub async fn buy(
//         &self,
//         asset: impl ToString,
//         amount: f64,
//         time: u32,
//     ) -> BinaryOptionsResult<(Uuid, Deal)> {
//         let order = OpenOrder::call(
//             amount,
//             asset.to_string(),
//             time,
//             self.credentials.demo as u32,
//         )?;
//         let request_id = order.request_id;
//         let res = self
//             .send_message(
//                 WebSocketMessage::OpenOrder(order),
//                 MessageInfo::SuccessopenOrder,
//                 order_validator(request_id),
//             )
//             .await?;
//         if let WebSocketMessage::SuccessopenOrder(order) = res {
//             debug!("Successfully opened buy trade!");
//             return Ok((order.id, order));
//         }
//         Err(PocketOptionError::UnexpectedIncorrectWebSocketMessage(res.info()).into())
//     }
// }

// #[cfg(test)]
// mod tests {
//     use tokio::time::sleep;

//     use crate::utils::tracing::start_tracing;

//     use super::*;

//     #[tokio::test]
//     async fn test_pocket_option() -> anyhow::Result<()> {
//         start_tracing()?;
//         let ssid = r#"42["auth",{"session":"t0mc6nefcv7ncr21g4fmtioidb","isDemo":1,"uid":90000798,"platform":2}]	"#;
//         let demo = true;
//         let api = PocketOption::new(ssid, demo).await?;
//         let mut loops = 0;
//         while loops < 100 {
//             loops += 1;
//             sleep(Duration::from_millis(100)).await;
//         }
//         dbg!(api.buy("EURUSD_otc", 1.0, 60).await?);
//         Ok(())
//     }
// }
