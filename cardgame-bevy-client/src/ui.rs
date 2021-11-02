use crate::chat::{ChatMessage, ChatMessages};
use bevy::prelude::*;
use bevy_spicy_networking::{NetworkClient, NetworkSettings};
use std::net::SocketAddr;
use cardgame_common::message::C2SMessage;

pub struct MainPanel;
pub struct ConnectButton;
pub struct GlobalChatSettings {
    chat_style: TextStyle,
    author_style: TextStyle,
}
pub struct MessageButton;

pub struct RoomListButton;
pub struct RoomList;

pub struct ChatArea;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup_ui.system());

        app.add_system(handle_connect_button.system());
        app.add_system(handle_message_button.system());
        app.init_resource::<GlobalChatSettings>();
        app.add_system_to_stage(CoreStage::PostUpdate, handle_chat_area.system());
    }
}

fn handle_connect_button(
    mut net: ResMut<NetworkClient>,
    interaction_query: Query<
        (&Interaction, &Children),
        (Changed<Interaction>, With<ConnectButton>),
    >,
    mut text_query: Query<&mut Text>,
    mut messages: Query<&mut ChatMessages>,
) {
    let mut messages = messages.single_mut().unwrap();

    for (interaction, children) in interaction_query.iter() {
        let mut text = text_query.get_mut(children[0]).unwrap();
        if let Interaction::Clicked = interaction {
            if net.is_connected() {
                net.disconnect();
            } else {
                text.sections[0].value = String::from("Connecting...");
                messages.add(ChatMessage::SystemMessage("Connecting to server...".to_string()));

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
        }
    }
}

impl FromWorld for GlobalChatSettings {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();

        GlobalChatSettings {
            chat_style: TextStyle {
                font: asset_server.load("fonts/GlowSans.otf"),
                font_size: 20.,
                color: Color::BLACK,
            },
            author_style: TextStyle {
                font: asset_server.load("fonts/GlowSans.otf"),
                font_size: 20.,
                color: Color::RED,
            },
        }
    }
}

fn handle_message_button(
    net: Res<NetworkClient>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<MessageButton>)>,
    mut messages: Query<&mut ChatMessages>,
) {
    let mut messages = messages.single_mut().unwrap();

    for interaction in interaction_query.iter() {
        if let Interaction::Clicked = interaction {
            match net.send_message(C2SMessage::Ping2) {
                Ok(()) => (),
                Err(err) => messages.add(ChatMessage::SystemMessage(format!(
                    "Could not send message: {:?}",
                    err
                ))),
            }
        }
    }
}

fn handle_chat_area(
    chat_settings: Res<GlobalChatSettings>,
    messages: Query<&ChatMessages, Changed<ChatMessages>>,
    mut chat_text_query: Query<&mut Text, With<ChatArea>>,
) {
    let messages = if let Ok(messages) = messages.single() {
        messages
    } else {
        return;
    };

    let sections = messages
        .messages
        .iter()
        .flat_map(|msg| {
            std::array::IntoIter::new([
                TextSection {
                    value: format!("{}: ", msg.get_author()),
                    style: chat_settings.author_style.clone(),
                },
                TextSection {
                    value: format!("{}\n", msg.get_text()),
                    style: chat_settings.chat_style.clone(),
                },
            ])
        })
        .collect::<Vec<_>>();

    let mut text = chat_text_query.single_mut().unwrap();

    text.sections = sections;
}

fn setup_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(UiCameraBundle::default());

    commands.spawn_bundle((ChatMessages::new(),));

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                justify_content: JustifyContent::SpaceBetween,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .insert(MainPanel)
        .with_children(|parent| {
            // Chat Area
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(80.), Val::Percent(90.)),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent
                        .spawn_bundle(TextBundle {
                            ..Default::default()
                        })
                        .insert(ChatArea);
                });
            parent.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(20.), Val::Percent(90.)),
                    ..Default::default()
                },
                ..Default::default()
            }).with_children(|parent| {
                parent
                    .spawn_bundle(TextBundle {
                        ..Default::default()
                    });
            });

            // Button Bar
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.), Val::Percent(10.)),
                        ..Default::default()
                    },
                    material: materials.add(Color::GRAY.into()),
                    ..Default::default()
                })
                .with_children(|parent_button_bar| {
                    parent_button_bar
                        .spawn_bundle(ButtonBundle {
                            style: Style {
                                size: Size::new(Val::Percent(50.), Val::Percent(100.)),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(MessageButton)
                        .with_children(|button| {
                            button.spawn_bundle(TextBundle {
                                text: Text::with_section(
                                    "Send Message!",
                                    TextStyle {
                                        font: asset_server.load("fonts/GlowSans.otf"),
                                        font_size: 40.,
                                        color: Color::BLACK,
                                    },
                                    TextAlignment {
                                        vertical: VerticalAlign::Center,
                                        horizontal: HorizontalAlign::Center,
                                    },
                                ),
                                ..Default::default()
                            });
                        });

                    parent_button_bar
                        .spawn_bundle(ButtonBundle {
                            style: Style {
                                size: Size::new(Val::Percent(50.), Val::Percent(100.)),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(ConnectButton)
                        .with_children(|button| {
                            button.spawn_bundle(TextBundle {
                                text: Text::with_section(
                                    "Connect to server",
                                    TextStyle {
                                        font: asset_server.load("fonts/GlowSans.otf"),
                                        font_size: 40.,
                                        color: Color::BLACK,
                                    },
                                    TextAlignment {
                                        vertical: VerticalAlign::Center,
                                        horizontal: HorizontalAlign::Center,
                                    },
                                ),
                                ..Default::default()
                            });
                        });
                });
        });
}