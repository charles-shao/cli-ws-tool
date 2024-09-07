use sha3::{Digest, Sha3_256};
use std::{
    collections::HashMap,
    net::TcpStream,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::structs::payload::{MessageType, Payload};

use tungstenite::protocol::frame::{coding::CloseCode, CloseFrame};
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};

pub struct Client {
    pub socket: Box<WebSocket<MaybeTlsStream<TcpStream>>>,
    username: String,
    created_at: u64,
}

impl Client {
    pub fn socket_id(&self) -> String {
        let mut hasher = Sha3_256::new();
        hasher.update(&self.username);
        hasher.update(&self.created_at.to_string());
        let result = hasher.finalize();

        format!("{:x}", result)
    }
}

pub struct ClientServer {
    clients: HashMap<String, Client>,
}

impl ClientServer {
    pub fn new() -> ClientServer {
        ClientServer {
            clients: HashMap::with_capacity(256),
        }
    }

    pub fn get_clients(&self) -> Vec<(&String, &Client)> {
        let clients: Vec<(&String, &Client)> = self
            .clients
            .iter()
            .filter_map(|(user_id, client)| {
                if client.socket.can_write() {
                    Some((user_id, client))
                } else {
                    None
                }
            })
            .collect();

        return clients;
    }

    pub fn list_user_ids(&self, writable: bool) -> Vec<String> {
        return self
            .clients
            .iter()
            .filter_map(|(user_id, client)| {
                if !writable {
                    Some(user_id.clone())
                } else if client.socket.can_write() {
                    Some(user_id.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();
    }

    pub fn add_client(&mut self, user_id: &String) {
        let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        let mut hasher = Sha3_256::new();
        hasher.update(&user_id);
        hasher.update(&duration.as_secs().to_string());
        let result = hasher.finalize();

        let endpoint: String = format!("ws://127.0.0.1:8000/echo_channel/{:x}", &result);
        let (socket, _response) = connect(endpoint).expect("Can't connect");

        self.clients.insert(
            user_id.clone(),
            Client {
                socket: Box::new(socket),
                username: user_id.clone(),
                created_at: duration.as_secs(),
            },
        );

        let payload: Payload = Payload {
            from: user_id.clone(),
            kind: MessageType::System,
            text: format!("{} has connected.", user_id),
            timestamp: duration.as_secs(),
        };

        match self.clients.get_mut(user_id) {
            Some(client) => match client.socket.send(payload.to_message()) {
                Ok(()) => {}
                Err(error) => println!("error: {}", error),
            },

            None => {
                println!("send message error via add_client due to unfound user");
            }
        };
    }

    pub fn get_client(
        &mut self,
        user_id: &str,
    ) -> Option<&mut WebSocket<MaybeTlsStream<TcpStream>>> {
        match self.clients.get_mut(user_id) {
            Some(client) => return Some(&mut client.socket),
            None => {
                println!("Unable to find client: {}", user_id);
                return None;
            }
        }
    }

    pub fn close_client(&mut self, user_id: &str) {
        match self.clients.get_mut(user_id) {
            Some(client) => {
                let close_frame: CloseFrame = CloseFrame {
                    code: CloseCode::Normal,
                    reason: std::borrow::Cow::Borrowed("Client disconnected"),
                };

                match client.socket.close(Some(close_frame)) {
                    Ok(()) => println!("Closed socket for {}", user_id),
                    Err(err) => println!("Unable to close socket for {}; error: {}", user_id, err),
                }
            }
            None => {
                println!("Unable to close socket for {}", user_id)
            }
        }
    }

    pub fn write(&mut self, user_id: &str, payload: &Payload) {
        match self.get_client(user_id) {
            Some(socket) => {
                let message = match ::serde_json::to_string_pretty(payload) {
                    Ok(value) => Message::Text(value),
                    Err(_) => Message::Text(serde_json::to_string(payload).unwrap()),
                };

                match socket.send(message) {
                    Ok(()) => {}
                    Err(error) => println!("error: {}", error),
                }
            }
            None => {
                println!("User not found");
            }
        }
    }
}
