use serde::{Deserialize, Serialize};
use tungstenite::Message;

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    pub from: String,
    pub kind: MessageType,
    pub text: String,
    pub timestamp: u64,
}

impl Payload {
    pub fn to_message(&self) -> Message {
        let message = match ::serde_json::to_string_pretty(&self) {
            Ok(value) => Message::Text(value),
            Err(_) => Message::Text(serde_json::to_string(&self).unwrap()),
        };

        return message;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
    User,
    System,
    Echo,
}
