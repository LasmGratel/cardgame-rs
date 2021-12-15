extern crate cardgame;

use cardgame::messages::*;
use cardgame::Signal;
use cardgame::*;
use message_io::network::*;
use message_io::node;
use message_io::node::*;
use std::sync::mpsc::*;
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use std::process::exit;
use cardgame::error::{GameError, RoomError};

// 共享状态
type SharedStates = (Arc<Mutex<String>>, Arc<Mutex<ClientState>>, Arc<Mutex<Vec<Card>>>, Arc<Mutex<String>>, Arc<Mutex<SystemTime>>);

/// 客户端状态
#[derive(Eq, PartialEq, Clone)]
enum ClientState {
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



fn run_network_thread(
    _server_id: Endpoint,
    handler: NodeHandler<Signal>,
    listener: NodeListener<Signal>,
    sender: Sender<S2CMessage>,
    mutexs: SharedStates
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let (user_name, client_state, cards_mutex, landlord_name, last_packet_time) = mutexs;
        listener.for_each(move |event| match event {
            NodeEvent::Signal(signal) => match signal {
                cardgame::Signal::Greet => {
                    /*
                    let message = C2SMessage::Ping;
                    let output_data = bincode::serialize(&message).unwrap();
                    handler.network().send(server_id, &output_data);
                    handler
                        .signals()
                        .send_with_timer(crate::Signal::Greet, Duration::from_secs(1));*/
                }
                _ => {}
            },
            NodeEvent::Network(net_event) => match net_event {
                NetEvent::Message(_, input_data) => {
                    let message: S2CMessage = bincode::deserialize(input_data).unwrap();
                    match message {
                        S2CMessage::Pong => {
                            let recv_time = SystemTime::now();
                            let send_time = *last_packet_time.lock().unwrap();

                            let duration = recv_time.duration_since(send_time).expect("Time went backwards");
                            println!("砰！延迟为 {}", humantime::format_duration(duration));
                        }
                        S2CMessage::RoomJoined(room) => {
                            let mut state = client_state.lock().unwrap();
                            println!("加入房间：{}", room);
                            *state = ClientState::WaitingForPlayers(room);
                        }
                        S2CMessage::LandlordMove(landlord) => {
                            let user_name = &*user_name.lock().unwrap();
                            if user_name == &landlord {
                                println!("{} 你是否叫地主？", landlord);
                            } else {
                                println!("等待 {} 叫地主", landlord);
                            }

                            *landlord_name.lock().unwrap() = landlord;
                        }
                        S2CMessage::CardsSubmitted(player, cards) => {
                            println!("{} 出牌：{}", player, cards_to_string(&cards));
                            if user_name.lock().unwrap().eq(&player) {
                                let mut cards_mut = cards_mutex.lock().unwrap();
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
                            if *user_name.lock().unwrap() == landlord {
                                let mut player_cards = cards_mutex.lock().unwrap();
                                for card in cards.iter() {
                                    player_cards.push(*card);
                                    player_cards.sort();
                                }
                                print!("你的手牌：");
                                print_cards(&player_cards);
                            }
                            *client_state.lock().unwrap() = ClientState::Gaming;
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
                                    *client_state.lock().unwrap() = ClientState::Idle;
                                    *landlord_name.lock().unwrap() = String::new();
                                    cards_mutex.lock().unwrap().clear();

                                    println!("现在你可以输入 再来一局|摸了 来进行重新比赛投票，也可以安全地离开房间。");
                                }
                                _ => {}
                            }
                        }
                        S2CMessage::RematchVote(player, rematch, count) => {
                            if rematch {
                                println!("{} 同意再来一局。({}/3)", player, count);
                            } else {
                                println!("{} 不同意再来一局，房间销毁。", player);
                                *client_state.lock().unwrap() = ClientState::Idle;
                                cards_mutex.lock().unwrap().clear();
                            }
                        }
                        S2CMessage::GameStarted(cards, landlord) => {
                            let user_name = &*user_name.lock().unwrap();
                            print!("你的手牌: ");
                            print_cards(&cards);

                            print!("游戏开始，");
                            if user_name == &landlord {
                                println!("{} 你是否叫地主？", landlord);
                            } else {
                                println!("等待 {} 叫地主", landlord);
                            }

                            *cards_mutex.lock().unwrap() = cards;
                            *landlord_name.lock().unwrap() = landlord;
                            *client_state.lock().unwrap() = ClientState::WaitingForLandlord;
                        }
                        S2CMessage::MatchmakeStatus(count, expected_time, remaining_time) => {
                            if remaining_time.is_zero() {
                                println!("无法匹配到玩家，请重试。");
                                *client_state.lock().unwrap() = ClientState::Idle;
                            } else {
                                println!("当前匹配队列共有 {} 位玩家，剩余匹配时间：{}s", count, remaining_time.as_secs());
                            }
                        }
                        _ => {
                            sender.send(message).expect("Cannot send message");
                        }
                    }
                }
                NetEvent::Connected(_, _) => unreachable!(), // Only generated when a listener accepts
                NetEvent::Disconnected(_) => {
                    println!("服务器断开，程序关闭。");
                    handler.stop();
                    exit(0);
                },
            },
        });
    })
}

fn read_line(prompt: &str) -> String {
    let mut line = String::new();
    loop {
        println!("{}", prompt);
        std::io::stdin()
            .read_line(&mut line)
            .expect("Invalid input");

        if !line.trim().is_empty() {
            break;
        }
    }
    line
}

fn run_console_thread(
    server_id: Endpoint,
    handler: NodeHandler<Signal>,
    rx: Receiver<S2CMessage>,
    mutexs: SharedStates
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let (user_name, client_state, cards_mutex, landlord_name, last_packet_time) = mutexs;

        let send_to_server = |msg: &C2SMessage| {
            let data = bincode::serialize(msg).unwrap();
            handler.network().send(server_id, &data);
        };

        let line = read_line("请输入用户名：");
        let line = line.trim();
        if line == "exit" {
            return;
        }
        let user = line.to_string();
        *user_name.lock().unwrap() = user.clone();
        let data = bincode::serialize(&C2SMessage::Login(user.clone())).unwrap();
        handler.network().send(server_id, &data);
        let msg = rx.recv().unwrap();
        match msg {
            S2CMessage::LoggedIn => {
                println!("Logged in!");
                *client_state.lock().unwrap() = ClientState::Idle;
            }
            _ => {
                println!("Unknown message from server");
                return;
            }
        }
        loop {
            let line = read_line("请输入命令：");
            let line = line.trim().to_string();
            if line == "exit" {
                return;
            }

            if line.starts_with("加入 ") {
                let room = line.trim_start_matches("加入 ");
                if *client_state.lock().unwrap() != ClientState::Idle {
                    println!("此时无法加入房间。");
                    return;
                } else {
                    let data = bincode::serialize(&C2SMessage::JoinRoom(String::from(room))).unwrap();
                    handler.network().send(server_id, &data);
                }
            } else if line.starts_with("出牌 ") {
                if *client_state.lock().unwrap() == ClientState::Gaming {
                    let str = line.trim_start_matches("出牌 ");
                    let cards = parse_input(str);
                    if let Some(cards) = cards {
                        let data = bincode::serialize(&C2SMessage::SubmitCards(cards)).unwrap();
                        handler.network().send(server_id, &data);
                    } else {
                        println!("你没有出任何牌！")
                    }
                } else {
                    println!("你现在还不能出牌！");
                }
            } else {
                match line.as_str() {
                    "叫地主" => {
                        if *client_state.lock().unwrap() != ClientState::WaitingForLandlord {
                            println!("此时还不能叫地主！");
                        } else if landlord_name.lock().unwrap().ne(&user) {
                            println!("不是你叫地主！")
                        } else {
                            send_to_server(&C2SMessage::ChooseLandlord(true));
                        }
                    }
                    "不叫" => {
                        if *client_state.lock().unwrap() != ClientState::WaitingForLandlord {
                            println!("此时还不能叫地主！");
                        } else if landlord_name.lock().unwrap().ne(&user) {
                            println!("不是你叫地主！")
                        } else {
                            send_to_server(&C2SMessage::ChooseLandlord(false));
                        }
                    }
                    "再来一局" => {
                        if client_state.lock().unwrap().clone() == ClientState::WaitingForRematch {
                            let data =
                                bincode::serialize(&C2SMessage::RematchVote(true))
                                    .unwrap();
                            handler.network().send(server_id, &data);
                        } else {
                            println!("此时还不能进行重新比赛投票！");
                        }
                    }
                    "摸了" => {
                        if client_state.lock().unwrap().clone() == ClientState::WaitingForRematch {
                            let data =
                                bincode::serialize(&C2SMessage::RematchVote(false))
                                    .unwrap();
                            handler.network().send(server_id, &data);
                        } else {
                            println!("此时还不能进行重新比赛投票！");
                        }
                    }
                    "开始游戏" => {
                        match client_state.lock().unwrap().clone() {
                            ClientState::Idle => {
                                println!("你还没加入一个房间！");
                            }
                            ClientState::WaitingForPlayers(room_name) => {
                                let data =
                                    bincode::serialize(&C2SMessage::StartGame(room_name.clone()))
                                        .unwrap();
                                handler.network().send(server_id, &data);
                            }
                            _ => {
                                println!("游戏已开始或无法开始！")
                            }
                        }
                    }
                    "延迟" => {
                        *last_packet_time.lock().unwrap() = SystemTime::now();
                        send_to_server(&C2SMessage::Ping);
                    }
                    "pass" => {
                        if *client_state.lock().unwrap() == ClientState::Gaming {
                            let data = bincode::serialize(&C2SMessage::Pass).unwrap();
                            handler.network().send(server_id, &data);
                        } else {
                            println!("你现在还不能过牌！");
                        }
                    }
                    "匹配" => {
                        if *client_state.lock().unwrap() != ClientState::Idle {
                            println!("此时还不能加入匹配队列！");
                        } else {
                            send_to_server(&C2SMessage::Matchmake);
                        }
                    }
                    "游戏列表" => {
                        let data = bincode::serialize(&C2SMessage::QueryRoomList).unwrap();
                        handler.network().send(server_id, &data);
                        let msg = rx.recv().unwrap();
                        if let S2CMessage::RoomList(lobbies) = msg {
                            for name in lobbies.iter() {
                                print!("{}, ", name)
                            }
                        } else {
                            println!("无法获取游戏列表")
                        }
                    }
                    _ => {
                        println!("未知指令");
                    }
                }
            }
        }
    })
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
/*
fn run() {
    println!("Len: {}", gen_cards().len());

    let mut game = Game::new();
    game.players.push(Player::new(String::from("114514"), 0));
    game.players.push(Player::new(String::from("1919810"), 0));
    game.players.push(Player::new(String::from("123456"), 0));
    game.start().expect("Game not started!");
    loop {
        game.print_landlord();
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("not a valid input");
        if input.trim() == "叫" {
            println!("叫地主");

            let result = game.run();
            if result.is_err() {
                println!("{}", result.unwrap_err());
            } else {
                break;
            }
        } else {
            game.move_landlord_index();
        }
    }

    loop {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("not a valid input");

        if input.as_str().trim() == "pass" {
            let result = game.pass();
            if result.is_err() {
                println!("{}", result.unwrap_err());
            }
        } else {
            let cards: Vec<Card> = parse_input(input.as_str().trim());

            let result = game.submit_cards(cards);
            if result.is_err() {
                println!("{}", result.unwrap_err());
            }
        }
        if game.state == GameState::WaitingForPlayers {
            break;
        }
    }
}*/

fn main() {
    let transport = Transport::FramedTcp;
    let remote_addr = "127.0.0.1:3042".to_remote_addr().unwrap();
    let (handler, listener) = node::split();

    let server_id = match handler.network().connect(transport, remote_addr.clone()) {
        Ok((server_id, local_addr)) => {
            println!(
                "连接至服务器 {}", remote_addr
            );
            println!("本地端口: {}", local_addr.port());
            server_id
        }
        Err(_) => {
            return println!(
                "无法使用 {} 协议连接到 {}",
                transport, remote_addr
            )
        }
    };

    let user_name = Arc::new(Mutex::new(String::new()));
    let client_state = Arc::new(Mutex::new(ClientState::NotLoggedIn));
    let cards = Arc::new(Mutex::new(vec![]));
    let landlord_name = Arc::new(Mutex::new(String::new()));
    let last_packet_time = Arc::new(Mutex::new(SystemTime::now()));

    let mutexs = (user_name, client_state, cards, landlord_name, last_packet_time);

    handler.signals().send(crate::Signal::Greet);

    let (tx, rx) = channel();

    run_network_thread(server_id, handler.clone(), listener, tx, mutexs.clone());
    run_console_thread(server_id, handler, rx, mutexs).join().unwrap();
}

pub mod device;
