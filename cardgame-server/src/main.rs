use bimap::BiHashMap;
use cardgame::*;
use message_io::network::*;
use message_io::node;
use cardgame::user::{UserManager, UserState, User, UserId};
use std::collections::HashMap;
use crate::server_lobby::ServerLobby;
use message_io::node::NodeEvent;
use std::time::Duration;
use std::sync::Mutex;
use std::rc::Rc;
use crate::server_network::{MessagePacket, MessageTarget};

/// 大厅
pub mod server_lobby;

/// 网络管理
pub mod server_network;

pub fn main() {
    // Create a node, the main message-io entity. It is divided in 2 parts:
    // The 'handler', used to make actions (connect, send messages, signals, stop the node...)
    // The 'listener', used to read events from the network or signals.
    let (handler, listener) = node::split::<cardgame::Signal>();

    let network_handle = std::thread::spawn(move || {
        let address = "0.0.0.0:3042";

        // Listen for TCP, UDP and WebSocket messages at the same time.
        handler
            .network()
            .listen(Transport::FramedTcp, address)
            .expect("Unable to listen on the address!");

        let signals = handler.signals();
        let network = handler.network();

        println!("服务器在 {} 上监听", address);

        let lobby = Rc::new(Mutex::new(ServerLobby::new(network)));
        let mut clients: Vec<Endpoint> = vec![];
        let mut client_map: BiHashMap<String, Endpoint> = BiHashMap::new();
        let mut user_manager = UserManager::new(String::from("users"));
        let mut user_states: HashMap<String, UserState> = HashMap::new();

        let mut matchmake_timer = 0;
        let mut matchmake_id = 0;
        let mut rng = thread_rng();

        println!("初始化完毕");

        // Read incoming network events.
        listener.for_each(move |event| match event {
            NodeEvent::Network(net_event) => {
                match net_event {
                    NetEvent::Connected(endpoint, _) => {
                        println!("{} Connected", endpoint);
                        clients.push(endpoint);
                    }
                    NetEvent::Message(endpoint, data) => {
                        let get_user = || -> Option<User> {
                            client_map.get_by_right(&endpoint).map(|x| user_manager.get_user(x).unwrap())
                        };
                        let get_user_id = || -> Option<&UserId> {
                            client_map.get_by_right(&endpoint)
                        };
                        let send_to_client = |msg: &S2CMessage| {
                            let to_send = bincode::serialize(msg).unwrap();
                            network.send(endpoint, &to_send);
                        };
                        let send_packet = |packet: MessagePacket| {
                            match packet.0 {
                                MessageTarget::Reply => {
                                    lobby.lock().unwrap().send_packet((MessageTarget::Endpoint(endpoint), packet.1));
                                }
                                _ => {
                                    lobby.lock().unwrap().send_packet(packet);
                                }
                            }
                        };
                        let send_to_user = |user: &UserId, msg: &S2CMessage| {
                            let to_send = bincode::serialize(msg).unwrap();
                            let user_endpoint = client_map.get_by_left(user).unwrap();
                            network.send(*user_endpoint, &to_send);
                        };

                        let message: C2SMessage = bincode::deserialize(data).unwrap();
                        match message {
                            C2SMessage::Ping => {
                                send_to_client(&S2CMessage::Pong);
                            }
                            C2SMessage::Login(username) => {
                                println!("玩家 {} 登入", username);
                                send_to_client(&S2CMessage::LoggedIn);
                                lobby.lock().unwrap().connect(username, endpoint);
                            }
                            // 加入房间
                            C2SMessage::JoinRoom(room_name) => {
                                lobby.lock().unwrap().join_room_by_endpoint(&room_name, endpoint);
                            }
                            C2SMessage::StartGame(room_name) => {
                                lobby.lock().unwrap().start_game_by_name(&room_name);
                            }
                            C2SMessage::ChooseLandlord(choose) => {
                                let room = if let Some(UserState::Playing(room_name)) = user_states.get(client_map.get_by_right(&endpoint).unwrap()) {
                                    lobby.lock().unwrap().rooms.get_mut(room_name)
                                } else {
                                    None
                                };

                                if let Some(room) = room {
                                    // 尚未开始叫地主阶段
                                    if room.game.state != GameState::WaitingForLandlord {
                                        send_to_client(&S2CMessage::RoomErr(RoomError::NotStarted));
                                    }
                                    // 不是当前的地主玩家
                                    if room.game.players[room.game.landlord_index].user != get_user().unwrap().id {
                                        send_to_client(&S2CMessage::RoomErr(RoomError::NotLandlordPlayer));
                                    }
                                    if choose { // 叫地主
                                        room.game.run().expect("Game cannot run");
                                        // 通知所有玩家地主人选
                                        for player in room.game.players.iter() {
                                            send_to_user(&player.user, &S2CMessage::LordCards(room.game.current_player().user.clone(), room.game.landlord_cards.clone()))
                                        }
                                    } else { // 不叫
                                        room.game.move_landlord_index();
                                        // 通知下一个地主
                                        for player in room.game.players.iter() {
                                            send_to_user(&player.user, &S2CMessage::LandlordMove(room.game.landlord_player().user.clone()))
                                        }
                                    }
                                } else {
                                    // 房间还未准备好
                                    send_to_client(&S2CMessage::RoomErr(RoomError::NotReady));
                                }
                            }
                            C2SMessage::Pass => {
                                let room = if let Some(UserState::Playing(room_name)) = user_states.get(client_map.get_by_right(&endpoint).unwrap()) {
                                    lobby.lock().unwrap().rooms.get_mut(room_name)
                                } else {
                                    None
                                };

                                if let Some(room) = room {
                                    if room.game.state != GameState::Running {
                                        send_to_client(&S2CMessage::RoomErr(RoomError::NotReady));
                                    } else if room.game.current_player().user != get_user().unwrap().id {
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
                            C2SMessage::RematchVote(rematch) => {
                                let user_id = client_map.get_by_right(&endpoint).unwrap();
                                match lobby.lock().unwrap().rematch_vote(user_id, rematch) {
                                    Ok(packet) => {

                                    }
                                    Err(err) => {

                                    }
                                }
                            }
                            C2SMessage::SubmitCards(cards) => {
                                let room = if let Some(UserState::Playing(room_name)) = user_states.get(client_map.get_by_right(&endpoint).unwrap()) {
                                    lobby.lock().unwrap().rooms.get_mut(room_name)
                                } else {
                                    None
                                };

                                if let Some(room) = room {
                                    if room.game.state != GameState::Running {
                                        send_to_client(&S2CMessage::RoomErr(RoomError::NotReady));
                                    } else if room.game.current_player().user != get_user().unwrap().id {
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
                                                match e.clone() {
                                                    GameError::Win(user, player_type, score) => {
                                                        // TODO 结算并写入文件

                                                        user_states.insert(user, UserState::Idle);
                                                        room.state = RoomState::WaitingForRematch(0);
                                                    }
                                                    _ => {}
                                                }
                                                send_to_client(&S2CMessage::GameErr(e))
                                            }
                                        }
                                    }
                                } else {
                                    send_to_client(&S2CMessage::RoomErr(RoomError::NotReady));
                                }
                            }
                            C2SMessage::Matchmake => {
                                let user_id = get_user_id().unwrap();
                                matchmake_timer = 120; // 重设等待玩家倒计时
                                lobby.lock().unwrap().waiting_list.push(user_id.clone());
                                user_states.insert(user_id.clone(), UserState::Matchmaking);
                                signals.send(Signal::Matchmake);
                            }
                            C2SMessage::QueryRoomList => {
                                let data = lobby.lock().unwrap().rooms.keys().map(|x| x.to_string()).collect();
                                send_to_client(&S2CMessage::RoomList(data));
                            }
                            _ => {
                                println!("Unknown message")
                            }
                        }
                    }
                    NetEvent::Disconnected(endpoint) => {
                        println!("{} 已断开", endpoint);
                        if let Some(user) = client_map.get_by_right(&endpoint).map(|x| user_manager.get_user(x).unwrap()) {
                            lobby.lock().unwrap().disconnect(&user.id);
                            user_states.remove(&user.id);
                        }
                        let client = clients
                            .iter()
                            .position(|x| *x == endpoint)
                            .expect("No such client found");
                        clients.remove(client);
                        client_map.remove_by_right(&endpoint);
                    },
                }
            },
            NodeEvent::Signal(signal) => if let cardgame::Signal::Matchmake = signal {
                let send_to_user = |user: &UserId, msg: &S2CMessage| {
                    let to_send = bincode::serialize(msg).unwrap();
                    let user_endpoint = client_map.get_by_left(user).unwrap();
                    network.send(*user_endpoint, &to_send);
                };

                if lobby.lock().unwrap().waiting_list.len() == 0 {
                    // 没有人在等待队列，停止匹配
                    matchmake_timer = 0;
                } else if matchmake_timer > 0 {
                    matchmake_timer -= 1;
                    signals.send_with_timer(Signal::Matchmake, Duration::from_secs(1));

                    if lobby.lock().unwrap().waiting_list.len() >= 3 {
                        lobby.lock().unwrap().waiting_list.shuffle(&mut rng);

                        for i in 0..(lobby.lock().unwrap().waiting_list.len() / 3) {
                            matchmake_id += 1;
                            let room_name = format!("Matchmake Room #{}", matchmake_id);
                            for j in 0..3 {
                                let user = lobby.lock().unwrap().waiting_list.get(j).unwrap().clone();

                                let result = lobby.lock().unwrap().join_room(&room_name, user.clone());
                                match result {
                                    Ok(packet) => {
                                        lobby.lock().unwrap().send_packet(packet);
                                    }
                                    Err(err) => {
                                        // send_to_client(&S2CMessage::LobbyErr(err));
                                    }
                                }
                            }
                            for _ in 0..3 {
                                lobby.lock().unwrap().waiting_list.remove(0);
                            }

                            lobby.lock().unwrap().start_game_by_name(&room_name);
                        }
                    }

                    for user in lobby.lock().unwrap().waiting_list.iter() {
                        let msg = &S2CMessage::MatchmakeStatus(lobby.lock().unwrap().waiting_list.len() as u32, Duration::from_secs(120), Duration::from_secs(matchmake_timer));

                        let to_send = bincode::serialize(msg).unwrap();
                        let user_endpoint = client_map.get_by_left(user).unwrap();
                        network.send(*user_endpoint, &to_send);
                    }

                }
            }
        });
    });

    network_handle.join().unwrap();
}

/*
fn get_line(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Not a valid input");
    input
}
*/