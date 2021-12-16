use bevy::prelude::*;
use cardgame::{Game, GameState, Lobby, Room, RoomState};
use cardgame_common::message::{C2SMessage, S2CMessage};
use bevy::app::ScheduleRunnerSettings;
use std::time::Duration;
use bevy_spicy_networking::{AppNetworkServerMessage, ConnectionId, NetworkData, NetworkServer, ServerNetworkEvent, ServerPlugin};
use std::net::SocketAddr;
use bevy::log::LogPlugin;
use cardgame::error::{GameError, LobbyError, RoomError};
use cardgame::user::{User, UserState};
use crate::lobby::ServerLobby;
use crate::server_network::MessageTarget;

/// 出卡计时器
struct SubmitTimer(Timer);

fn setup_network_system(mut net: ResMut<NetworkServer>) {
    let ip_address = "127.0.0.1".parse().expect("Could not parse ip address");

    info!("Address of the server: {}", ip_address);

    let socket_address = SocketAddr::new(ip_address, 9999);

    match net.listen(socket_address) {
        Ok(_) => (),
        Err(err) => {
            error!("Could not start listening: {:?}", err);
            panic!();
        }
    }

    info!("Started listening for new connections!");
}

fn setup_lobby(mut commands: Commands) {
    commands.insert_resource(ServerLobby::default());
}

fn games_system(query: Query<&Game>) {
    for game in query.iter() {
        // println!("{}", game.index);
    }
}

fn submit_timer_system(time: Res<Time>, mut timer: ResMut<SubmitTimer>, query: Query<&Game>) {
    if timer.0.tick(time.delta()).just_finished() {
        for name in query.iter() {
        }
    }
}

fn handle_connection_events(
    mut commands: Commands,
    net: Res<NetworkServer>,
    mut lobby: ResMut<ServerLobby>,
    mut network_events: EventReader<ServerNetworkEvent>,
) {
    for event in network_events.iter() {
        match event {
            ServerNetworkEvent::Connected(connection_id) => {
                println!("{} Connected", connection_id.address());
            }
            ServerNetworkEvent::Disconnected(connection_id) => {
                println!("{} Disconnected", connection_id.address());
                lobby.disconnect_by_endpoint(&connection_id);
            }
            ServerNetworkEvent::Error(err) => {
                error!("{:?}", err);
            }
        }
    }
}

// Receiving a new message is as simple as listening for events of `NetworkData<T>`
fn handle_messages(
    mut new_messages: EventReader<NetworkData<C2SMessage>>,
    mut lobby: ResMut<ServerLobby>,
    net: Res<NetworkServer>,
) {
    for message in new_messages.iter() {
        let user: ConnectionId = message.source();
        let user_id = lobby.get_user(&user).map(|x| x.clone());
        let err = match &**message {
            C2SMessage::Ping => {
                net.send_message(user, S2CMessage::Pong);
            }
            C2SMessage::Login(username) => {
                println!("玩家 {} 登入", username);
                lobby.connect(username.to_string(), user);
                net.send_message(user, S2CMessage::LoggedIn);
            }
            // 加入房间
            C2SMessage::JoinRoom(room_name) => {
                match lobby.join_room(&net, &room_name, user_id.unwrap().to_string()) {
                    Ok(_) => {}
                    Err(e) => {
                        net.send_message(user, S2CMessage::LobbyErr(e));
                    }
                }
            }
            C2SMessage::StartGame(room_name) => {
                match lobby.start_game_by_name(&net, room_name) {
                    Ok(_) => {}
                    Err(e) => {
                        net.send_message(user, S2CMessage::RoomErr(e));
                    }
                }
            }
            C2SMessage::ChooseLandlord(choose) => {
                let packet = lobby.choose_landlord_by_connection_id(&user, *choose).expect("Cannot choose landlord");
                lobby.send_packet(&net, packet);
            }
            C2SMessage::Pass => {
                let room = lobby.get_room_by_connection_id_mut(&user);

                if let Some(room) = room {
                    if room.game.state != GameState::Running {
                        net.send_message(user, S2CMessage::RoomErr(RoomError::NotReady));
                    } else if &room.game.current_player().user != &user_id.unwrap() {
                        net.send_message(user, S2CMessage::GameErr(GameError::NotYourTurn));
                    } else {
                        match room.game.pass() {
                            Ok(next_player) => {
                                let room_name = room.name.clone();
                                lobby.send_to_room_by_name(&net, &room_name, S2CMessage::Move(next_player.clone()));
                            }
                            Err(e) => {
                                net.send_message(user, S2CMessage::GameErr(e));
                            }
                        }
                    }
                } else {
                    net.send_message(user, S2CMessage::RoomErr(RoomError::NotReady));
                }
            }
            C2SMessage::RematchVote(rematch) => {
                match lobby.rematch_vote(&user, *rematch) {
                    Ok((count, vote)) => {
                        net.send_message(user, S2CMessage::RematchVote(user_id.as_ref().unwrap().to_string(), vote, count));
                        let room = lobby.get_room_by_user_mut(user_id.as_ref().unwrap()).unwrap();
                        if count == 3 {
                            room.game.reset();
                        } else {
                            room.state = RoomState::WaitingForRematch(count);
                        }
                    }
                    Err(err) => {
                    }
                }
            }
            C2SMessage::SubmitCards(cards) => {
                match lobby.submit_cards(&user, cards.clone()) {
                    Ok(next_player) => {
                        let room = lobby.get_room_by_user(user_id.as_ref().unwrap()).unwrap();
                        lobby.send_to_room_by_name(&net, &room.name, S2CMessage::CardsSubmitted(user_id.unwrap().to_string(), cards.clone()));
                        lobby.send_to_room_by_name(&net, &room.name, S2CMessage::Move(next_player.clone()));
                    }
                    Err(e) => {
                        net.send_message(user, S2CMessage::GameErr(e));
                    }
                }
            }
            C2SMessage::Matchmake => {
                // matchmake_timer = 120; // 重设等待玩家倒计时
                lobby.waiting_list.push(user_id.as_ref().unwrap().clone());
                lobby.user_states.insert(user_id.as_ref().unwrap().clone(), UserState::Matchmaking);
                // signals.send(Signal::Matchmake);
            }
            C2SMessage::QueryRoomList => {
                let data = lobby.rooms.keys().map(|x| x.to_string()).collect();
                net.send_message(user, S2CMessage::RoomList(data));
            }
            _ => {
                println!("Unknown message")
            }
        };
    }
}

fn join_room(net: Res<NetworkServer>, room_name: &str, connection_id: &ConnectionId, lobby: &mut ServerLobby) {
    let user_id = lobby.get_user(connection_id).expect("Unknown user").clone();

    let room = lobby.rooms
        .entry(room_name.to_string())
        .or_insert(Room::new(room_name.to_string()));

    if room.users.contains(&user_id) {
        net.send_message(connection_id.clone(), S2CMessage::LobbyErr(LobbyError::HasJoinedRoom));
        return;
    }

    if room.users.len() == 3 {
        net.send_message(connection_id.clone(), S2CMessage::LobbyErr(LobbyError::RoomFull));
        return;
    }

    room.push(user_id);

}

fn register_messages(builder: &mut AppBuilder) {
    builder.listen_for_server_message::<C2SMessage>();
}

fn main() {
    let mut builder = App::build();
    builder
        .add_plugins(MinimalPlugins)
        .add_plugin(LogPlugin::default())
        .add_plugin(ServerPlugin::default())
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_millis(
            50 // 20 tps
        )))
        .insert_resource(SubmitTimer(Timer::from_seconds(2.0, true)));
    register_messages(&mut builder);
    builder.add_startup_system(setup_lobby.system())
        .add_startup_system(setup_network_system.system())
        .add_system(games_system.system())
        .add_system(submit_timer_system.system())
        .add_system(handle_connection_events.system())
        .add_system(handle_messages.system());

    builder.run();
}

pub mod lobby;
pub mod server_network;