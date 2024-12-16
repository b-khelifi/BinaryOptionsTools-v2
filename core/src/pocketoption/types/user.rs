use core::fmt;

use crate::{general::types::UserRequest, pocketoption::parser::message::WebSocketMessage};

pub type PocketUser = UserRequest<WebSocketMessage>;

impl fmt::Debug for PocketUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Response type: '{}'\nMessage: '{}'",
            self.info, self.message
        )
    }
}
