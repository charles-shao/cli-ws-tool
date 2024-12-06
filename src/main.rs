use dotenv::dotenv;
use std::time::{SystemTime, UNIX_EPOCH};

mod structs;

use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};

use self::structs::client_server::ClientServer;
use self::structs::payload::{MessageType, Payload};

const CLI_COMMANDS: [&str; 6] = ["add", "ping", "message", "list", "close", "exit"];

fn main() {
    dotenv().ok();

    let mut user_client_server = ClientServer::new();

    loop {
        let cli_command = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Which command?")
            .default(0)
            .items(&CLI_COMMANDS[..])
            .interact_opt()
            .unwrap();

        match cli_command {
            Some(option) => match *(&CLI_COMMANDS[option]) {
                "ping" => {
                    let user_ids = user_client_server.list_user_ids(true);

                    if user_ids.len() > 0 {
                        let selection = Select::with_theme(&ColorfulTheme::default())
                            .with_prompt("Which user? (q for none)")
                            .default(0)
                            .items(&user_ids[..])
                            .interact_opt()
                            .unwrap();

                        match selection {
                            Some(option) => {
                                let user_id = &user_ids[option];
                                let duration =
                                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

                                let payload: Payload = Payload {
                                    from: user_id.clone(),
                                    kind: MessageType::User,
                                    text: format!("pong!"),
                                    timestamp: duration.as_secs(),
                                };

                                user_client_server.write(user_id, &payload);
                            }
                            None => {
                                println!("No user selected.")
                            }
                        }
                    } else {
                        return println!("No users.");
                    }
                }
                "message" => {
                    let user_ids = user_client_server.list_user_ids(true);

                    if user_ids.len() > 0 {
                        let selection = Select::with_theme(&ColorfulTheme::default())
                            .with_prompt("Which user? (q for none)")
                            .default(0)
                            .items(&user_ids[..])
                            .interact_opt()
                            .unwrap();

                        match selection {
                            Some(option) => {
                                let user_id = &user_ids[option];
                                let duration =
                                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

                                let message: String = Input::with_theme(&ColorfulTheme::default())
                                    .with_prompt("Message?")
                                    .interact_text()
                                    .unwrap();

                                let payload: Payload = Payload {
                                    from: user_id.clone(),
                                    kind: MessageType::User,
                                    text: message,
                                    timestamp: duration.as_secs(),
                                };

                                user_client_server.write(user_id, &payload);
                            }
                            None => {
                                println!("No user selected.")
                            }
                        }
                    } else {
                        return println!("No users.");
                    }
                }
                "add" => {
                    let existing_user_ids: Vec<String> = user_client_server.list_user_ids(false);

                    let user_id: String = Input::with_theme(&ColorfulTheme::default())
                        .with_prompt("Username?")
                        .validate_with({
                            move |input: &String| -> Result<(), &str> {
                                if existing_user_ids.contains(input) {
                                    Err("User already exists")
                                } else {
                                    Ok(())
                                }
                            }
                        })
                        .interact_text()
                        .unwrap();

                    user_client_server.add_client(&user_id);

                    user_client_server.notify_presence(&user_id);

                    println!("Added client {}", &user_id);
                }
                "list" => {
                    println!("Listing clients");
                    for (user_id, client) in user_client_server.get_clients() {
                        println!(
                            " · {}; read: {}, write: {}; SHA256: {}",
                            user_id,
                            client.socket.can_read(),
                            client.socket.can_write(),
                            client.socket_id()
                        );
                    }
                }
                "close" => {
                    let user_ids = user_client_server.list_user_ids(true);

                    if user_ids.len() > 0 {
                        let selection = Select::with_theme(&ColorfulTheme::default())
                            .with_prompt("Close connection for which user?")
                            .default(0)
                            .items(&user_ids[..])
                            .interact_opt()
                            .unwrap();
                        match selection {
                            Some(option) => {
                                let user_id = &user_ids[option];

                                user_client_server.close_client(user_id);
                            }
                            None => {
                                println!("No user selected.")
                            }
                        }
                    } else {
                        return println!("No users.");
                    }
                }
                "exit" => {
                    let num_of_sessions = user_client_server.list_user_ids(true).len();
                    if Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt(format!(
                            "Do you want exit and terminate {} client(s)?",
                            num_of_sessions
                        ))
                        .interact()
                        .unwrap()
                    {
                        for user_id in user_client_server.list_user_ids(true) {
                            println!("Closing socket for {}", &user_id);
                            user_client_server.close_client(&user_id);
                        }

                        break;
                    } else {
                        println!("Resuming client server");
                    }
                }
                _ => {
                    println!("Out of bounds detected.")
                }
            },
            None => {
                println!("No command selected.")
            }
        };
    }
}
