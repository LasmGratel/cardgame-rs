#![allow(clippy::type_complexity)]

use bevy::prelude::*;
use bevy_spicy_networking::{ClientNetworkEvent, NetworkClient, NetworkData, NetworkSettings, AppNetworkClientMessage};
use std::net::SocketAddr;
use cardgame_common::message::{C2SMessage, S2CMessage};
use bevy::log::LogPlugin;
use std::sync::mpsc;
use std::{thread, time, io};
use std::sync::mpsc::{TryRecvError, Receiver};
use std::time::{Duration, SystemTime};
use bevy::app::{ScheduleRunnerPlugin, ScheduleRunnerSettings};
use regex::Regex;
use cardgame::Card;
use cardgame::error::{GameError, RoomError};
use crate::chat::{ChatMessages, ChatMessage};
use crate::chat::ChatMessage::SystemMessage;
use crate::ClientStatus::{Gaming, Idle, NotLoggedIn, WaitingForLandlord};
use crate::console_plugin::{ConsoleDebugPlugin, InputEvent};
use crate::logon::login;
use crate::ui::UIPlugin;

#[derive(Clone, PartialEq)]
struct ConsoleInput(String);

fn main() {
    let mut app = App::build();

    app.add_plugins(DefaultPlugins);

    // You need to add the `ClientPlugin` first before you can register
    // `ClientMessage`s
    app.add_plugin(bevy_spicy_networking::ClientPlugin::default());
    // app.add_plugin(ConsoleDebugPlugin);
    app.add_plugin(UIPlugin);
    app.add_startup_system(connect_to_server.system());
    app.add_system(handle_incoming_messages.system());
    app.add_system(handle_network_events.system());
    app.add_system(handle_input.system());
    app.insert_resource(ClientState::default());
    //app.add_startup_system(login.system());
    register_messages(&mut app);
    app.add_plugin(ScheduleRunnerPlugin::default());
    app.insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_millis(
        50 // 20 tps
    )));
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

/// 客户端状态
#[derive(Eq, PartialEq, Clone)]
pub enum ClientStatus {
    /// 未登入
    NotLoggedIn,

    /// 空闲
    Idle,

    /// 在房间中等待玩家
    WaitingForPlayers(String),

    /// 等待重新比赛
    WaitingForRematch,

    /// 等待叫地主
    WaitingForLandlord,

    /// 游戏中
    Gaming,
}

pub struct ClientState {
    pub cards: Vec<Card>,
    pub user_name: String,
    pub landlord_name: String,
    pub status: ClientStatus,
    pub last_packet_time: SystemTime,
}

impl Default for ClientState {
    fn default() -> Self {
        ClientState {
            status: NotLoggedIn,
            last_packet_time: SystemTime::now(),
            landlord_name: String::default(),
            cards: vec![],
            user_name: String::default(),
        }
    }
}

pub fn parse_input(input: &str) -> Option<Vec<Card>> {
    let input = input
        .to_ascii_uppercase()
        .replace("10", "ß")
        .replace("1", "0")
        .replace("ß", "1");
    let cards_regex = Regex::new("([1-9jqkaJQKA]|鬼|王)+").unwrap();
    let result = cards_regex.find(input.as_str())?;
    let result = result.as_str().replace("10", "1");

    let mut vec: Vec<Card> = Vec::new();
    for c in result.chars() {
        vec.push(Card::from_char(&c));
    }
    vec.retain(|&i| i != Card::Unknown);
    if vec.is_empty() {
        None
    } else {
        Some(vec)
    }
}

fn handle_input(
    net: Res<NetworkClient>,
    mut state: ResMut<ClientState>,
    mut events: EventReader<InputEvent>
) {
    for event in events.iter() {
        let line: &str = &event.0;
        match line {
            "list_rooms" => {
                net.send_message(C2SMessage::QueryRoomList);
            }
            "ping" => {
                state.last_packet_time = SystemTime::now();
                net.send_message(C2SMessage::Ping);
            }
            "叫地主" => {
                if state.status != ClientStatus::WaitingForLandlord {
                    println!("此时还不能叫地主！");
                } else if state.landlord_name.ne(&state.user_name) {
                    println!("不是你叫地主！")
                } else {
                    net.send_message(C2SMessage::ChooseLandlord(true));
                }
            }
            "不叫" => {
                if state.status != ClientStatus::WaitingForLandlord {
                    println!("此时还不能叫地主！");
                } else if state.landlord_name.ne(&state.user_name) {
                    println!("不是你叫地主！")
                } else {
                    net.send_message(C2SMessage::ChooseLandlord(false));
                }
            }
            "再来一局" => {
                if state.status == ClientStatus::WaitingForRematch {
                    net.send_message(C2SMessage::RematchVote(true));
                } else {
                    println!("此时还不能进行重新比赛投票！");
                }
            }
            "摸了" => {
                if state.status == ClientStatus::WaitingForRematch {
                    net.send_message(C2SMessage::RematchVote(false));
                } else {
                    println!("此时还不能进行重新比赛投票！");
                }
            }
            "开始游戏" => {
                match &state.status {
                    ClientStatus::Idle => {
                        println!("你还没加入一个房间！");
                    }
                    ClientStatus::WaitingForPlayers(room_name) => {
                        net.send_message(C2SMessage::StartGame(room_name.clone()));
                    }
                    _ => {
                        println!("游戏已开始或无法开始！")
                    }
                }
            }
            "pass" => {
                if state.status == ClientStatus::Gaming {
                    net.send_message(C2SMessage::Pass);
                } else {
                    println!("你现在还不能过牌！");
                }
            }
            "匹配" => {
                if state.status != ClientStatus::Idle {
                    println!("此时还不能加入匹配队列！");
                } else {
                    net.send_message(C2SMessage::Matchmake);
                }
            }
            "游戏列表" => {
                net.send_message(C2SMessage::QueryRoomList);
            }
            _ => {
                let splitted: Vec<&str> = line.split(" ").collect();
                match splitted[0] {
                    "出牌" => {
                        if state.status == ClientStatus::Gaming {
                            let str = splitted[1];
                            let cards = parse_input(str);
                            if let Some(cards) = cards {
                                net.send_message(C2SMessage::SubmitCards(cards));
                            } else {
                                println!("你没有出任何牌！")
                            }
                        } else {
                            println!("你现在还不能出牌！");
                        }
                    }
                    "join" => {
                        let room = splitted[1];
                        if state.status != ClientStatus::Idle {
                            println!("此时无法加入房间。");
                            return;
                        } else {
                            net.send_message(C2SMessage::JoinRoom(room.to_string()));
                        }
                    }
                    "login" => {
                        if let Some(user_name) = splitted.get(1) {
                            state.user_name = user_name.to_string();
                            net.send_message(C2SMessage::Login(user_name.to_string()));
                        } else {
                            println!("Please input a username");
                        }
                    }
                    _ => {
                        println!("Unknown command {}", line);
                    }
                }
            }
        }
    }
}

fn handle_incoming_messages(
    mut state: ResMut<ClientState>,
    mut new_messages: EventReader<NetworkData<S2CMessage>>,
) {
    for message in new_messages.iter() {
        match &**message {
            S2CMessage::Chat(msg) => {
                println!("Chat: {}", msg);
            }
            S2CMessage::Pong => {
                let recv_time = SystemTime::now();
                let send_time = state.last_packet_time.clone();

                let duration = recv_time.duration_since(send_time).expect("Time went backwards");
                println!("砰！延迟为 {}", humantime::format_duration(duration));
            }
            S2CMessage::RoomJoined(room) => {
                println!("加入房间：{}", room);
                if state.status == Idle {
                    state.status = ClientStatus::WaitingForPlayers(room.to_string());
                }
            }
            S2CMessage::RoomList(lobbies) => {
                for name in lobbies.iter() {
                    print!("{}, ", name)
                }
            }
            S2CMessage::LandlordMove(landlord) => {
                let user_name = &state.user_name;
                if user_name == landlord {
                    println!("{} 你是否叫地主？", landlord);
                } else {
                    println!("等待 {} 叫地主", landlord);
                }

                state.landlord_name = landlord.to_string();
            }
            S2CMessage::CardsSubmitted(player, cards) => {
                println!("{} 出牌：{}", player, cards_to_string(&cards));
                if &state.user_name == player {
                    let mut cards_mut = &mut state.cards;
                    for card in cards.iter() {
                        let pos = cards_mut.iter().position(|x| x == card).unwrap();
                        cards_mut.remove(pos);
                    }
                    print!("你的手牌：");
                    print_cards(&cards_mut);
                }
            }
            S2CMessage::Move(player) => {
                println!("{} 请出牌", player);
            }
            S2CMessage::LordCards(landlord, cards) => {
                println!("{} 叫地主，地主牌为 {}", landlord, cards_to_string(&cards));
                if &state.user_name == landlord {
                    let mut player_cards = &mut state.cards;
                    for card in cards.iter() {
                        player_cards.push(*card);
                        player_cards.sort();
                    }
                    print!("你的手牌：");
                    print_cards(&player_cards);
                }
                state.status = Gaming;
            }
            S2CMessage::RoomErr(err) => {
                match err {
                    RoomError::NotReady => {
                        println!("房间未准备好！");
                    }
                    RoomError::NotStarted => {
                        println!("游戏还未开始！");
                    }
                    RoomError::NotLandlordPlayer => {
                        println!("不是你叫地主！");
                    }
                    RoomError::RoomFull => {
                        println!("Room is full");
                    }
                }
            }
            S2CMessage::GameErr(err) => {
                match err {
                    GameError::NotYourTurn => {
                        println!("你还不能出牌！");
                    }
                    GameError::NoSuchCards => {
                        println!("你没有这些牌");
                    }
                    GameError::WrongRule => {
                        println!("你出的牌不满足当前规则");
                    }
                    GameError::NoRule => {
                        println!("你出的牌不匹配任何规则")
                    }
                    GameError::Win(player, player_type, score) => {
                        println!("{} 赢了。", player);
                        state.status = Idle;
                        state.landlord_name = String::default();
                        state.cards.clear();

                        println!("现在你可以输入 再来一局|摸了 来进行重新比赛投票，也可以安全地离开房间。");
                    }
                    _ => {}
                }
            }
            S2CMessage::RematchVote(player, rematch, count) => {
                if *rematch {
                    println!("{} 同意再来一局。({}/3)", player, count);
                } else {
                    println!("{} 不同意再来一局，房间销毁。", player);
                    state.status = Idle;
                    state.landlord_name = String::new();
                    state.cards.clear();
                }
            }
            S2CMessage::GameStarted(cards, landlord) => {
                let user_name = &state.user_name;
                print!("你的手牌: ");
                print_cards(&cards);

                print!("游戏开始，");
                if user_name == landlord {
                    println!("{} 你是否叫地主？", landlord);
                } else {
                    println!("等待 {} 叫地主", landlord);
                }

                state.cards = cards.clone();
                state.landlord_name = landlord.to_string();
                state.status = WaitingForLandlord;
            }
            S2CMessage::MatchmakeStatus(count, expected_time, remaining_time) => {
                if remaining_time.is_zero() {
                    println!("无法匹配到玩家，请重试。");
                    state.status = Idle;
                } else {
                    println!("当前匹配队列共有 {} 位玩家，剩余匹配时间：{}s", count, remaining_time.as_secs());
                }
            }
            S2CMessage::LoggedIn => {
                state.status = Idle;
                println!("Logged in!");
            }
            _ => {
                println!("Other message");
            }
        }
    }
}

fn print_cards(cards: &[Card]) {
    println!("{}", cards_to_string(cards));
}

fn cards_to_string(cards: &[Card]) -> String {
    let mut s = String::new();
    for c in cards.iter() {
        s += "[";
        s += c.to_string();
        s += "]";
    }
    s
}

fn handle_network_events(
    mut new_network_events: EventReader<ClientNetworkEvent>,
    net: Res<NetworkClient>,
) {
    for event in new_network_events.iter() {
        match event {
            ClientNetworkEvent::Connected => {
                println!("Connected to server");
                net.send_message(C2SMessage::Ping);
            }

            ClientNetworkEvent::Disconnected => {
                println!("Server disconnected");
            }
            ClientNetworkEvent::Error(err) => {
                error!("Network error: {:?}", err);
            }
        }
    }
}

pub mod chat;
pub mod game_render;
pub mod console_plugin;
pub mod logon;
pub(crate) mod ui;