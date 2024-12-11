use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use crate::pocketoption::{error::PocketResult, parser::message::WebSocketMessage};

use super::{info::MessageInfo, order::{Deal, UpdateClosedDeals, UpdateOpenedDeals}, update::{UpdateAssets, UpdateBalance}};

type HashMapData = HashMap<MessageInfo, Vec<(Box<dyn Fn(&WebSocketMessage) -> bool + Send + Sync>, tokio::sync::oneshot::Sender<WebSocketMessage>)>>;

#[derive(Default, Clone)]
pub struct Data {
    balance: Arc<Mutex<UpdateBalance>>,
    opened_deals: Arc<Mutex<UpdateOpenedDeals>>,
    closed_deals: Arc<Mutex<UpdateClosedDeals>>,
    payout_data: Arc<Mutex<HashMap<String, i32>>>,
    pending_requests: Arc<Mutex<HashMapData>>,
    server_tyme: Arc<Mutex<i64>>
}

impl From<UpdateAssets> for HashMap<String, i32> {
    fn from(value: UpdateAssets) -> Self {
        value.0.iter().map(|a| (a.symbol.clone(), a.payout)).collect()
    }
}

impl Data {
    pub async fn update_balance(&self, balance: UpdateBalance)  {
        let mut blnc = self.balance.lock().await;
        *blnc = balance;
    }

    pub async fn get_balance(&self) -> UpdateBalance {
        self.balance.lock().await.clone()
    }

    pub async fn update_opened_deals(&self, deals: UpdateOpenedDeals) {
        let mut opened = self.opened_deals.lock().await;
        *opened = deals;
    }

    pub async fn get_opened_deals(&self) -> Vec<Deal> {
        self.opened_deals.lock().await.clone().0
    }

    pub async fn update_closed_deals(&self, deals: UpdateClosedDeals) {
        let mut closed = self.closed_deals.lock().await;
        *closed = deals;
    }

    pub async fn get_closed_deals(&self) -> Vec<Deal> {
        self.closed_deals.lock().await.clone().0
    }

    pub async fn update_payout_data(&self, payout: UpdateAssets) {
        let mut data = self.payout_data.lock().await;
        *data = payout.into();
    }

    pub async fn get_full_payout(&self) -> HashMap<String, i32> {
        self.payout_data.lock().await.clone()
    }

    pub async fn get_payout(&self, asset: impl ToString) -> Option<i32> {
        self.payout_data.lock().await.get(&asset.to_string()).cloned()
    }

    pub async fn add_user_request(&self, info: MessageInfo, validator: impl Fn(&WebSocketMessage) -> bool + Send + Sync + 'static, sender: tokio::sync::oneshot::Sender<WebSocketMessage>) {
        let mut requests = self.pending_requests.lock().await;
        if let Some(reqs) = requests.get_mut(&info) {
            reqs.push((Box::new(validator), sender));
            return;
        }

        requests.insert(info, vec![(Box::new(validator), sender)]);
    }

    pub async fn get_request(&self, message: &WebSocketMessage) -> PocketResult<Option<tokio::sync::oneshot::Sender<WebSocketMessage>>> {
        let mut requests = self.pending_requests.lock().await;
        let info = message.info();
        
        if let Some(reqs) = requests.get_mut(&info) {
            // Find the index of the matching validator
            let element = reqs.iter().enumerate().position(|(_, (v, _))| v(message));
            
            if let Some(idx) = element {
                // Remove the validator and return the sender
                let (_, s) = reqs.remove(idx);
                return Ok(Some(s));
            }
        }
        if let WebSocketMessage::FailOpenOrder(fail) = message {
            if let Some(reqs) = requests.remove(&MessageInfo::SuccessopenOrder) {
                for (_, sender) in reqs.into_iter() {
                    sender.send(WebSocketMessage::FailOpenOrder(fail.clone()))?;
                }

            }
        }
        Ok(None)
    }

    pub async fn update_server_time(&self, time: i64) {
        let mut s_time = self.server_tyme.lock().await;
        *s_time = time;
    }

    pub async fn get_server_time(&self) -> i64 {
        *self.server_tyme.lock().await
    }
}