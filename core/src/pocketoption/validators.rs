use uuid::Uuid;

use super::parser::message::WebSocketMessage;



pub fn order_validator(order_index: u64) -> impl Fn(&WebSocketMessage) -> bool + Send + Sync {
    move |message| {
        if let WebSocketMessage::SuccessopenOrder(order) = message {
            if order.request_id == order_index {
                return true;
            }
        }
        false
    }
}

pub fn candle_validator(asset: String, index: u64) -> impl Fn(&WebSocketMessage) -> bool + Send + Sync {
    move |message| {
        if let WebSocketMessage::LoadHistoryPeriod(history) = message {
            if history.asset == asset && history.index == index {
                return true;
            }
        }
        false
    }
}

pub fn order_result_validator(order_id: Uuid) -> impl Fn(&WebSocketMessage) ->  bool + Send + Sync {
    move |message| {
        if let WebSocketMessage::SuccesscloseOrder(orders) = message {
            if orders.deals.iter().any(|o| o.id == order_id) {
                return true;
            }
        }
        false
    }
}