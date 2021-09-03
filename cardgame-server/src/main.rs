use bimap::BiHashMap;
use cardgame::*;
use message_io::network::*;
use message_io::node;
use cardgame::user::{UserManager, UserState, User, UserId};
use std::collections::HashMap;
use crate::server_lobby::ServerLobby;
use message_io::node::NodeEvent;
use std::time::Duration;

pub mod server_lobby;

pub fn main() {
    // Create a node, the main message-io entity. It is divided in 2 parts:
    // The 'handler', used to make actions (connect, send messages, signals, stop the node...)
    // The 'listener', used to read events from the network or signals.
    let (handler, listener) = node::split::<cardgame::Signal>();

    let network_handle = std::thread::spawn(|| {
        let address = "0.0.0.0:3042";

        // Listen for TCP, UDP and WebSocket messages at the same time.
        handler
            .network()
            .listen(Transport::FramedTcp, address)
            .expect("Unable to listen on the address!");

        println!("服务器在 {} 上监听", address);

        let mut lobby = ServerLobby::new();
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
            NodeEvent::Network(net_event) => match net_event {
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
                    let send_to_client = |msg: &S2CMessage| -> () {
                        let to_send = bincode::serialize(msg).unwrap();
                        handler.network().send(endpoint, &to_send);
                    };
                    let send_to_user = |user: &UserId, msg: &S2CMessage| -> () {
                        let to_send = bincode::serialize(msg).unwrap();
                        let user_endpoint = client_map.get_by_left(user).unwrap();
                        handler.network().send(user_endpoint.clone(), &to_send);
                    };

                    let message: C2SMessage = bincode::deserialize(&data).unwrap();
                    match message {
                        C2SMessage::Ping => {
                            println!("客户端 Ping")
                        }
                        C2SMessage::Login(username) => {
                            println!("玩家 {} 登入", username);
                            send_to_client(&S2CMessage::LoggedIn);

                            client_map.insert(username.clone(), endpoint.clone());

                            let user = client_map.get_by_right(&endpoint).map(|x| user_manager.get_user(x).unwrap()).unwrap();
                            match user_states.get(&username) {
                                None => {
                                    user_states.insert(username, UserState::Idle);
                                }
                                Some(state) => {
                                    match state {
                                        UserState::Idle => {
                                            lobby.login(user.id.clone());

                                        }
                                        UserState::Matchmaking => {
                                            // 断线后取消匹配
                                            user_states.insert(user.id.clone(), UserState::Idle);
                                            lobby.login(user.id.clone());
                                        }
                                        UserState::Playing(room) => {
                                            // TODO 断线重连
                                        }
                                    }
                                }
                            }
                        }
                        // 加入房间
                        C2SMessage::JoinRoom(room_name) => {
                            let user = get_user().unwrap();
                            let result = lobby.join_room(&room_name, user.id.clone());
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
                                                        send_to_user(&player.user, &S2CMessage::GameStarted(player.cards.clone(), landlord_player.user.clone()));
                                                        user_states.insert(player.user.clone(), UserState::Playing(room_name.clone()));
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
                                            send_to_user(&player.user, &S2CMessage::GameStarted(player.cards.clone(), landlord_player.user.clone()));
                                            user_states.insert(player.user.clone(), UserState::Playing(room_name.clone()));
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
                                lobby.rooms.get_mut(room_name)
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
                            let room = if let Some(UserState::Playing(room_name)) = user_states.get(user_id) {
                                lobby.rooms.get_mut(room_name)
                            } else {
                                None
                            };

                            if let Some(room) = room {
                                let current_state = room.state.clone();
                                let room_name = room.name.clone();
                                if let RoomState::WaitingForRematch(count) = current_state {
                                    if rematch {
                                        if count + 1 == 3 {
                                            room.game.reset();

                                            match room.start_game() {
                                                Ok((landlord_player, players)) => {
                                                    for player in players {
                                                        send_to_user(&player.user, &S2CMessage::GameStarted(player.cards.clone(), landlord_player.user.clone()));
                                                        user_states.insert(player.user.clone(), UserState::Playing(room_name.clone()));
                                                    }
                                                }
                                                Err(err) => {
                                                    send_to_client(&S2CMessage::RoomErr(err));
                                                }
                                            }
                                        }
                                        room.state = RoomState::WaitingForRematch(count + 1);
                                        send_to_client(&S2CMessage::RematchVote(user_id.clone(), rematch, count + 1));
                                    } else {
                                        let name = room.name.clone();
                                        *room = Room::new(name);
                                        send_to_client(&S2CMessage::RematchVote(user_id.clone(), rematch, count));
                                    }
                                } else {
                                    send_to_client(&S2CMessage::RoomErr(RoomError::NotReady));
                                }
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

                                                    user_states.insert(user.clone(), UserState::Idle);
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
                            lobby.waiting_list.push(user_id.clone());
                            user_states.insert(user_id.clone(), UserState::Matchmaking);
                            handler.signals().send(Signal::Matchmake);
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
                    if let Some(user) = client_map.get_by_right(&endpoint).map(|x| user_manager.get_user(x).unwrap()) {
                        lobby.disconnect(&user.id);
                        user_states.remove(&user.id);
                    }
                    let client = clients
                        .iter()
                        .position(|x| *x == endpoint)
                        .expect("No such client found");
                    clients.remove(client);
                    client_map.remove_by_right(&endpoint);
                },
            },
            NodeEvent::Signal(signal) => match signal {
                cardgame::Signal::Matchmake => {
                    let send_to_user = |user: &UserId, msg: &S2CMessage| -> () {
                        let to_send = bincode::serialize(msg).unwrap();
                        let user_endpoint = client_map.get_by_left(user).unwrap();
                        handler.network().send(user_endpoint.clone(), &to_send);
                    };

                    if lobby.waiting_list.len() == 0 {
                        // 没有人在等待队列，停止匹配
                        matchmake_timer = 0;
                    } else if matchmake_timer > 0 {
                        matchmake_timer -= 1;
                        handler.signals().send_with_timer(Signal::Matchmake, Duration::from_secs(1));

                        if lobby.waiting_list.len() >= 3 {
                            lobby.waiting_list.shuffle(&mut rng);

                            for i in 0..(lobby.waiting_list.len() / 3) {
                                matchmake_id += 1;
                                let room_name = format!("Matchmake Room #{}", matchmake_id);
                                for j in 0..3 {
                                    let user = lobby.waiting_list.get(j).unwrap().clone();

                                    let result = lobby.join_room(&room_name, user.clone());
                                    match result {
                                        Ok(room) => {
                                            send_to_user(&user, &S2CMessage::RoomJoined(room.name.clone()));
                                        }
                                        Err(err) => {
                                            // send_to_client(&S2CMessage::LobbyErr(err));
                                        }
                                    }
                                }
                                for _ in 0..3 {
                                    lobby.waiting_list.remove(0);
                                }

                                let room = lobby.rooms.get_mut(&room_name).unwrap();
                                // 开始游戏
                                match room.start_game() {
                                    Ok((landlord_player, players)) => {
                                        for player in players {
                                            send_to_user(&player.user, &S2CMessage::GameStarted(player.cards.clone(), landlord_player.user.clone()));
                                            user_states.insert(player.user.clone(), UserState::Playing(room_name.clone()));
                                        }
                                    }
                                    Err(err) => {
                                        // send_to_client(&S2CMessage::RoomErr(err));
                                    }
                                }
                            }
                        }

                        for user in lobby.waiting_list.iter() {
                            let msg = &S2CMessage::MatchmakeStatus(lobby.waiting_list.len() as u32, Duration::from_secs(120), Duration::from_secs(matchmake_timer));

                            let to_send = bincode::serialize(msg).unwrap();
                            let user_endpoint = client_map.get_by_left(user).unwrap();
                            handler.network().send(user_endpoint.clone(), &to_send);
                        }

                    }
                }
                _ => {}
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