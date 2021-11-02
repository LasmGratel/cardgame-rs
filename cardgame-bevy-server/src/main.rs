use bevy::prelude::*;
use cardgame::{Game};
use cardgame_common::message::{C2SMessage, S2CMessage};
use bevy::app::ScheduleRunnerSettings;
use std::time::Duration;
use bevy_spicy_networking::{NetworkServer, ServerPlugin, NetworkData, ServerNetworkEvent, AppNetworkServerMessage};
use std::net::SocketAddr;
use bevy::log::LogPlugin;
use cardgame::user::User;
use crate::lobby::Lobby;

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
    commands.spawn().insert(Lobby::default());
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
    mut network_events: EventReader<ServerNetworkEvent>,
) {
    for event in network_events.iter() {
        match event {
            ServerNetworkEvent::Connected(connection_id) => {
                println!("{} Connected", connection_id.address());
            }
            ServerNetworkEvent::Disconnected(connection_id) => {
                println!("{} Disconnected", connection_id.address());
            }
            ServerNetworkEvent::Error(err) => {
                error!("{:?}", err);
            }
        }
    }
}

// Receiving a new message is as simple as listening for events of `NetworkData<T>`
fn handle_messages(
    mut lobby: Query<&mut Lobby>,
    mut new_messages: EventReader<NetworkData<C2SMessage>>,
    net: Res<NetworkServer>,
) {
    let lobby = lobby.single_mut().unwrap();
    for message in new_messages.iter() {
        let user = message.source();
        let err = match &**message {
            C2SMessage::Ping => {
                println!("Ping!");
                net.send_message(user, S2CMessage::Pong)
            }
            C2SMessage::Ping2 => {
                println!("Ping2!");
                net.send_message(user, S2CMessage::Pong2)
            }
            C2SMessage::JoinRoom(room_name) => {

                Ok(())
            }
            C2SMessage::QueryRoomList => {
                net.send_message(user, S2CMessage::RoomList(lobby.rooms.iter().map(|x| x.name.to_string()).collect()))
            }
            _ => {
                Ok(())
            }
        };
        if let Err(e) = err {
            error!("{:?}", e);
        }
    }
}

fn join_room(net: Res<NetworkServer>) {

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