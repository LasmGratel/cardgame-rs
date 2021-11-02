#![allow(clippy::type_complexity)]

use bevy::prelude::*;
use bevy_spicy_networking::{ClientNetworkEvent, NetworkClient, NetworkData, NetworkSettings, AppNetworkClientMessage};
use std::net::SocketAddr;
use cardgame_common::message::{C2SMessage, S2CMessage};
use bevy::log::LogPlugin;
use std::sync::mpsc;
use std::{thread, time, io};
use std::sync::mpsc::{TryRecvError, Receiver};
use crate::ui::{UIPlugin, ConnectButton};
use crate::chat::{ChatMessages, ChatMessage};
use crate::chat::ChatMessage::SystemMessage;

#[derive(Clone, PartialEq)]
struct ConsoleInput(String);

fn main() {
    let mut app = App::build();

    app.add_plugins(DefaultPlugins);

    // You need to add the `ClientPlugin` first before you can register
    // `ClientMessage`s
    app.add_plugin(bevy_spicy_networking::ClientPlugin);
    app.add_plugin(UIPlugin);
    // app.add_startup_system(connect_to_server.system());
    app.add_system(handle_incoming_messages.system());
    app.add_system(handle_network_events.system());
    register_messages(&mut app);
    app.run();
}

fn register_messages(builder: &mut AppBuilder) {
    builder.listen_for_client_message::<S2CMessage>();
}

///////////////////////////////////////////////////////////////
////////////// Incoming Message Handler ///////////////////////
///////////////////////////////////////////////////////////////

fn connect_to_server(mut net: ResMut<NetworkClient>) {
    let ip_address = "127.0.0.1".parse().unwrap();

    info!("Address of the server: {}", ip_address);

    let socket_address = SocketAddr::new(ip_address, 9999);

    net.connect(
        socket_address,
        NetworkSettings {
            max_packet_length: 10 * 1024 * 1024,
        },
    );
}

fn handle_incoming_messages(
    mut messages: Query<&mut ChatMessages>,
    mut new_messages: EventReader<NetworkData<S2CMessage>>,
) {
    let mut messages = messages.single_mut().unwrap();
    for message in new_messages.iter() {
        if **message == S2CMessage::Pong {
            messages.add(SystemMessage("Pong!".to_string()));
        } else if **message == S2CMessage::Pong2 {
            messages.add(SystemMessage("Pong2!".to_string()));
        } else {
            messages.add(SystemMessage("Other message".to_string()));
        }
    }
}

fn handle_network_events(
    connect_button_query: Query<&Children, With<ConnectButton>>,
    mut text_query: Query<&mut Text>,
    mut new_network_events: EventReader<ClientNetworkEvent>,
    net: Res<NetworkClient>,
    mut messages: Query<&mut ChatMessages>,
) {
    let connect_children = connect_button_query.single().unwrap();
    let mut text = text_query.get_mut(connect_children[0]).unwrap();
    let mut messages = messages.single_mut().unwrap();
    for event in new_network_events.iter() {
        match event {
            ClientNetworkEvent::Connected => {
                println!("Connected to server");
                messages.add(ChatMessage::SystemMessage(
                    "Successfully connected to server!".to_string(),
                ));
                text.sections[0].value = String::from("Disconnect");
                net.send_message(C2SMessage::Ping);
            }

            ClientNetworkEvent::Disconnected => {
                messages.add(ChatMessage::SystemMessage(
                    "Disconnected".to_string(),
                ));
                text.sections[0].value = String::from("Connect to server");
                println!("Server disconnected");
            }
            ClientNetworkEvent::Error(err) => {
                messages.add(ChatMessage::SystemMessage(
                    format!("Error: {:?}", err)
                ));
                error!("{:?}", err);
            }
        }
    }
}

pub mod ui;
pub mod chat;
pub mod game_render;