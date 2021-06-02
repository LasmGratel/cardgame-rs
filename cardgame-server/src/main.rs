use bimap::BiHashMap;
use cardgame::*;
use message_io::network::*;
use message_io::node;
use message_io::node::*;
use std::time::Duration;

pub fn main() {
    // Create a node, the main message-io entity. It is divided in 2 parts:
    // The 'handler', used to make actions (connect, send messages, signals, stop the node...)
    // The 'listener', used to read events from the network or signals.
    let (handler, listener) = node::split::<()>();

    // Listen for TCP, UDP and WebSocket messages at the same time.
    handler
        .network()
        .listen(Transport::FramedTcp, "0.0.0.0:3042")
        .unwrap();

    let mut lobby = Lobby::new();
    let mut clients: Vec<Endpoint> = vec![];
    let mut client_map: BiHashMap<String, Endpoint> = BiHashMap::new();

    let network_handle = std::thread::spawn(|| {
        // Read incoming network events.
        listener.for_each(move |event| match event.network() {
            NetEvent::Connected(endpoint, _) => {
                println!("{} Connected", endpoint);
                clients.push(endpoint);
            }
            NetEvent::Message(endpoint, data) => {
                let message: C2SMessage = bincode::deserialize(&data).unwrap();
                match message {
                    C2SMessage::Ping => {
                        println!("Ping from client")
                    }
                    C2SMessage::Login(username) => {
                        println!("User {} Logged in", username);
                        if client_map.left_values().find(|x| **x == username).is_none() {
                            lobby.users.push(LobbyUser {
                                name: username.clone(),
                            });
                            client_map.insert(username, endpoint);
                            let to_send = bincode::serialize(&S2CMessage::LoggedIn).unwrap();
                            handler.network().send(endpoint, &to_send);
                        }
                    }
                    C2SMessage::JoinRoom(room_name) => {
                        if !lobby.rooms.contains_key(&room_name) {
                            lobby.rooms.insert(
                                room_name.clone(),
                                Room {
                                    name: room_name.clone(),
                                    users: vec![],
                                },
                            );
                            println!("创建房间: {}", room_name)
                        }
                        let room = lobby.rooms.get_mut(&room_name).unwrap();
                        if room.users.len() < 3 {
                            let user_name = client_map.get_by_right(&endpoint).unwrap().clone();
                            room.users.push(LobbyUser {
                                name: user_name.clone(),
                            });
                            let to_send = bincode::serialize(&S2CMessage::RoomJoined).unwrap();
                            handler.network().send(endpoint, &to_send);
                            println!(
                                "{} 加入 {} 房间，共有 {} 人",
                                user_name,
                                room_name,
                                room.users.len()
                            );
                        } else {
                            println!("房间已满: {}", room_name);
                            let to_send = bincode::serialize(&S2CMessage::RoomFull).unwrap();
                            handler.network().send(endpoint, &to_send);
                        }
                    }
                    C2SMessage::StartGame(room_name) => {
                        if !lobby.rooms.contains_key(&room_name)
                            || lobby.rooms.get(&room_name).unwrap().users.len() != 3
                        {
                            let to_send = bincode::serialize(&S2CMessage::GameNotStarted(
                                String::from("房间不存在或未满！"),
                            ))
                            .unwrap();
                            handler.network().send(endpoint, &to_send);
                        } else {
                            let mut game = Game::new();
                            let room: &Room = lobby.rooms.get(&room_name).unwrap();
                            game.players = room
                                .users
                                .iter()
                                .map(|x| Player::new(x.name.clone(), 0))
                                .collect();
                            game.start().expect("Game failed to start!");
                            for p in game.players.iter() {
                                let to_send =
                                    bincode::serialize(&S2CMessage::GameStarted(p.cards.clone()))
                                        .unwrap();
                                handler.network().send(
                                    client_map.get_by_left(&p.data.id).unwrap().clone(),
                                    &to_send,
                                );
                            }
                        }
                    }
                    C2SMessage::QueryRoomList => {
                        let data: Vec<String> = lobby.rooms.keys().map(|x| x.to_string()).collect();
                        let to_send = bincode::serialize(&S2CMessage::RoomList(data)).unwrap();
                        handler.network().send(endpoint, &to_send);
                    }
                    _ => {
                        println!("Unknown message")
                    }
                }
            }
            NetEvent::Disconnected(endpoint) => {
                println!("{} Disconnected", endpoint);
                let client = clients
                    .iter()
                    .position(|x| *x == endpoint)
                    .expect("No such client found");
                clients.remove(client);
                client_map.remove_by_right(&endpoint);
            }
        });
    });

    network_handle.join();
}

fn get_line(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Not a valid input");
    input
}
