use serde::{Deserialize, Serialize};
use tungstenite::Message;

#[derive(Debug, Serialize, Deserialize)]
pub struct Presence {
    pub user_id: String,
    pub timestamp: u64,
}

impl Presence {
    pub fn to_message(&self) -> Message {
        let message = match ::serde_json::to_string_pretty(&self) {
            Ok(value) => Message::Text(value),
            Err(_) => Message::Text(serde_json::to_string(&self).unwrap()),
        };

        return message;
    }
}
