use bimap::BiHashMap;
use cardgame::*;
use message_io::network::*;
use message_io::node;
use message_io::node::*;
use std::time::Duration;
use cardgame::user::{UserManager, UserState, User};
use std::collections::HashMap;
use crate::server_mod::ServerLobby;

pub mod server_mod;

pub fn main() {
    // Create a node, the main message-io entity. It is divided in 2 parts:
    // The 'handler', used to make actions (connect, send messages, signals, stop the node...)
    // The 'listener', used to read events from the network or signals.
    let (handler, listener) = node::split::<()>();

    let network_handle = std::thread::spawn(|| {
        let addr = "0.0.0.0:3042";

        // Listen for TCP, UDP and WebSocket messages at the same time.
        handler
            .network()
            .listen(Transport::FramedTcp, addr)
            .expect("Unable to listen on the address!");

        println!("Server listening on {}", addr);

        let mut lobby = Lobby::new();
        let mut clients: Vec<Endpoint> = vec![];
        let mut client_map: BiHashMap<String, Endpoint> = BiHashMap::new();
        let mut user_manager = UserManager::new(String::from("users"));
        let mut user_states: HashMap<String, UserState> = HashMap::new();

        println!("Server initialized");

        // Read incoming network events.
        listener.for_each(move |event| match event.network() {
            NetEvent::Connected(endpoint, _) => {
                println!("{} Connected", endpoint);
                clients.push(endpoint);
            }
            NetEvent::Message(endpoint, data) => {
                let get_user = || -> Option<User> {
                    client_map.get_by_right(&endpoint).map(|x| user_manager.get_user_safe(x).unwrap())
                };
                let send_to_client = |msg: &S2CMessage| -> () {
                    let to_send = bincode::serialize(msg).unwrap();
                    handler.network().send(endpoint, &to_send);
                };
                let send_to_user = |user: &User, msg: &S2CMessage| -> () {
                    let to_send = bincode::serialize(msg).unwrap();
                    let user_endpoint = client_map.get_by_left(&user.id).unwrap();
                    handler.network().send(user_endpoint.clone(), &to_send);
                };

                let message: C2SMessage = bincode::deserialize(&data).unwrap();
                match message {
                    C2SMessage::Ping => {
                        println!("Ping from client")
                    }
                    C2SMessage::Login(username) => {
                        println!("User {} Logged in", username);
                        send_to_client(&S2CMessage::LoggedIn);

                        client_map.insert(username.clone(), endpoint.clone());

                        let user = client_map.get_by_right(&endpoint).map(|x| user_manager.get_user_safe(x).unwrap()).unwrap();
                        match user_states.get(&username) {
                            None => {
                                user_states.insert(username, UserState::Idle);
                            }
                            Some(state) => {
                                match state {
                                    UserState::Idle => {
                                        lobby.login(user);

                                    }
                                    UserState::Matchmaking => {
                                        // 断线后取消匹配
                                        user_states.insert(user.id.clone(), UserState::Idle);
                                        lobby.login(user);
                                    }
                                    UserState::Playing(room) => {
                                    }
                                }
                            }
                        }
                    }
                    // 加入房间
                    C2SMessage::JoinRoom(room_name) => {
                        let user = get_user().unwrap();
                        let result = lobby.join_room(&room_name, user.clone());
                        match result {
                            Ok(room) => {
                                send_to_client(&S2CMessage::RoomJoined(room_name.clone()));
                                println!(
                                    "{} 加入 {} 房间，共有 {} 人",
                                    user.id,
                                    room_name,
                                    room.users.len()
                                );

                                if room.users.len() == 3 {
                                    // 开始游戏
                                    if let Some(room) = lobby.rooms.get_mut(&room_name) {
                                        match room.start_game() {
                                            Ok((landlord_player, players)) => {
                                                for player in players {
                                                    send_to_user(&player.user, &S2CMessage::GameStarted(player.cards.clone(), landlord_player.user.id.clone()));
                                                    user_states.insert(player.user.id.clone(), UserState::Playing(room_name.clone()));
                                                }
                                            }
                                            Err(err) => {
                                                send_to_client(&S2CMessage::RoomErr(err));
                                            }
                                        }
                                    } else {
                                        send_to_client(&S2CMessage::RoomErr(RoomError::NotReady));
                                    }
                                }
                            }
                            Err(err) => {
                                send_to_client(&S2CMessage::LobbyErr(err));
                            }
                        }
                    }
                    C2SMessage::StartGame(room_name) => {
                        if let Some(room) = lobby.rooms.get_mut(&room_name) {
                            match room.start_game() {
                                Ok((landlord_player, players)) => {
                                    for player in players {
                                        send_to_user(&player.user, &S2CMessage::GameStarted(player.cards.clone(), landlord_player.user.id.clone()));
                                        user_states.insert(player.user.id.clone(), UserState::Playing(room_name.clone()));
                                    }
                                }
                                Err(err) => {
                                    send_to_client(&S2CMessage::RoomErr(err));
                                }
                            }
                        } else {
                            send_to_client(&S2CMessage::RoomErr(RoomError::NotReady));
                        }
                    }
                    C2SMessage::ChooseLandlord(choose) => {
                        let room = if let Some(UserState::Playing(room_name)) = user_states.get(client_map.get_by_right(&endpoint).unwrap()) {
                            lobby.rooms.get_mut(room_name)
                        } else {
                            None
                        };

                        if let Some(room) = room {
                            if room.game.state != GameState::WaitingForLandlord {
                                send_to_client(&S2CMessage::RoomErr(RoomError::NotStarted));
                            }
                            if room.game.players[room.game.landlord_index].user != get_user().unwrap() {
                                send_to_client(&S2CMessage::RoomErr(RoomError::NotLandlordPlayer));
                            }
                            if choose {
                                room.game.run();
                                for player in room.game.players.iter() {
                                    send_to_user(&player.user, &S2CMessage::LordCards(room.game.current_player().user.id.clone(), room.game.landlord_cards.clone()))
                                }
                            } else {
                                room.game.move_landlord_index();
                                for player in room.game.players.iter() {
                                    send_to_user(&player.user, &S2CMessage::LandlordMove(room.game.landlord_player().user.id.clone()))
                                }
                            }
                        } else {
                            send_to_client(&S2CMessage::RoomErr(RoomError::NotReady));
                        }
                    }
                    C2SMessage::Pass => {
                        let room = if let Some(UserState::Playing(room_name)) = user_states.get(client_map.get_by_right(&endpoint).unwrap()) {
                            lobby.rooms.get_mut(room_name)
                        } else {
                            None
                        };

                        if let Some(room) = room {
                            if room.game.state != GameState::Running {
                                send_to_client(&S2CMessage::RoomErr(RoomError::NotReady));
                            } else if room.game.current_player().user != get_user().unwrap() {
                                send_to_client(&S2CMessage::GameErr(GameError::NotYourTurn));
                            } else {
                                match room.game.pass() {
                                    Ok(next_player) => {
                                        for user in room.users.iter() {
                                            send_to_user(user, &S2CMessage::Move(next_player.clone()));
                                        }
                                    }
                                    Err(e) => {
                                        send_to_client(&S2CMessage::GameErr(e));
                                    }
                                }
                            }
                        } else {
                            send_to_client(&S2CMessage::RoomErr(RoomError::NotReady));
                        }
                    }
                    C2SMessage::SubmitCards(cards) => {
                        let room = if let Some(UserState::Playing(room_name)) = user_states.get(client_map.get_by_right(&endpoint).unwrap()) {
                            lobby.rooms.get_mut(room_name)
                        } else {
                            None
                        };

                        if let Some(room) = room {
                            if room.game.state != GameState::Running {
                                send_to_client(&S2CMessage::RoomErr(RoomError::NotReady));
                            } else if room.game.current_player().user != get_user().unwrap() {
                                send_to_client(&S2CMessage::GameErr(GameError::NotYourTurn));
                            } else {
                                match room.game.submit_cards(cards.clone()) {
                                    Ok(next_player) => {
                                        for user in room.users.iter() {
                                            send_to_user(user, &S2CMessage::CardsSubmitted(client_map.get_by_right(&endpoint).unwrap().clone(), cards.clone()));
                                            send_to_user(user, &S2CMessage::Move(next_player.clone()));
                                        }
                                    }
                                    Err(e) => {
                                        send_to_client(&S2CMessage::GameErr(e));
                                    }
                                }
                            }
                        } else {
                            send_to_client(&S2CMessage::RoomErr(RoomError::NotReady));
                        }
                    }
                    C2SMessage::QueryRoomList => {
                        let data = lobby.rooms.keys().map(|x| x.to_string()).collect();
                        send_to_client(&S2CMessage::RoomList(data));
                    }
                    _ => {
                        println!("Unknown message")
                    }
                }
            }
            NetEvent::Disconnected(endpoint) => {
                println!("{} Disconnected", endpoint);
                if let Some(user) = client_map.get_by_right(&endpoint).map(|x| user_manager.get_user(x)) {
                    lobby.disconnect(user);
                    user_states.remove(&user.id);
                }
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

fn login(manager: UserManager, user_id: String) {

}

fn get_line(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Not a valid input");
    input
}
