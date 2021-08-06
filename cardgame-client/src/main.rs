extern crate cardgame;

use cardgame::messages::*;
use cardgame::Signal;
use cardgame::*;
use message_io::network::*;
use message_io::node;
use message_io::node::*;
use std::io;
use std::sync::mpsc::*;
use std::thread::JoinHandle;
use std::time::Duration;
use async_std::task;
use std::sync::{Mutex, Arc};
use std::rc::Rc;

/// 客户端状态
#[derive(Eq, PartialEq, Clone)]
enum ClientState {
    /// 未登入
    NotLoggedIn,

    /// 空闲
    Idle,

    /// 在房间中等待玩家
    WaitingForPlayers(String),

    WaitingForLandlord
}

fn run_network_thread(
    server_id: Endpoint,
    handler: NodeHandler<Signal>,
    listener: NodeListener<Signal>,
    sender: Sender<S2CMessage>,
    mutexs: (Arc<Mutex<String>>, Arc<Mutex<ClientState>>, Arc<Mutex<Vec<Card>>>, Arc<Mutex<String>>)
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let (user_name, client_state, cards_mutex, landlord_name) = mutexs;
        listener.for_each(move |event| match event {
            NodeEvent::Signal(signal) => match signal {
                crate::Signal::Greet => {
                    /*
                    let message = C2SMessage::Ping;
                    let output_data = bincode::serialize(&message).unwrap();
                    handler.network().send(server_id, &output_data);
                    handler
                        .signals()
                        .send_with_timer(crate::Signal::Greet, Duration::from_secs(1));*/
                }
            },
            NodeEvent::Network(net_event) => match net_event {
                NetEvent::Message(_, input_data) => {
                    let message: S2CMessage = bincode::deserialize(&input_data).unwrap();
                    match message {
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
                        S2CMessage::LordCards(landlord, cards) => {
                            println!("{} 叫地主，地主牌为 {}", landlord, cards_to_string(&cards));
                            if *landlord_name.lock().unwrap() == landlord {
                                let mut player_cards = cards_mutex.lock().unwrap();
                                for card in cards.iter() {
                                    player_cards.push(card.clone());
                                }
                                print!("你的手牌：");
                                print_cards(&player_cards);
                            }
                        }
                        S2CMessage::RoomErr(err) => {
                            match err {
                                RoomError::NotReady => {
                                    println!("房间未准备好！");
                                }
                                RoomError::NotStarted => {
                                    println!("游戏还未开始！")
                                }
                                RoomError::NotLandlordPlayer => {
                                    println!("不是你叫地主！")
                                }
                            }
                        }
                        S2CMessage::GameErr(err) => {
                            match err {
                                GameError::NotYourTurn => {
                                    println!("你还不能出牌！");
                                }
                                GameError::NoSuchCards => {
                                    println!("你没有这些牌")
                                }
                                GameError::WrongRule => {
                                    println!("你出的牌不满足当前规则")
                                }
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
                        _ => {
                            sender.send(message).expect("Cannot send message");
                        }
                    }
                }
                NetEvent::Connected(_, _) => unreachable!(), // Only generated when a listener accepts
                NetEvent::Disconnected(_) => {
                    println!("Server is disconnected");
                    handler.stop();
                }
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
    mutexs: (Arc<Mutex<String>>, Arc<Mutex<ClientState>>, Arc<Mutex<Vec<Card>>>, Arc<Mutex<String>>)
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let (user_name, client_state, cards_mutex, landlord_name) = mutexs;

        let send_to_server = |msg: &C2SMessage| -> () {
            let data = bincode::serialize(msg).unwrap();
            handler.network().send(server_id.clone(), &data);
        };

        let mut current_room = String::new();
        let line = read_line("请输入用户名：");
        let line = line.trim();
        if line == "exit" {
            return;
        }
        let user = line.to_string();
        *user_name.lock().unwrap() = String::from(user.clone());
        let mut player_cards: Vec<Card> = vec![];
        let mut is_landlord_candidate = false;
        let data = bincode::serialize(&C2SMessage::Login(user.clone())).unwrap();
        handler.network().send(server_id.clone(), &data);
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
                let state = client_state.clone();
                let handler = handler.clone();
                let task = task::spawn(async move {
                    let room = line.trim_start_matches("加入 ");
                    if *state.lock().unwrap() != ClientState::Idle {
                        println!("此时无法加入房间。");
                        return;
                    } else {
                        let data = bincode::serialize(&C2SMessage::JoinRoom(String::from(room))).unwrap();
                        handler.network().send(server_id.clone(), &data);
                    }
                });
            } else {
                match line.as_str() {
                    "叫地主" => {
                        if *client_state.lock().unwrap() != ClientState::WaitingForLandlord {
                            println!("此时还不能叫地主！");
                        } else if landlord_name.lock().unwrap().ne(&user) {
                            println!("不是你叫地主！")
                        } else if !is_landlord_candidate {
                            send_to_server(&C2SMessage::ChooseLandlord);
                        }
                    }
                    "开始游戏" => {
                        if current_room.is_empty() {
                            println!("你还没加入一个房间！");
                        } else {
                            let data =
                                bincode::serialize(&C2SMessage::StartGame(current_room.clone()))
                                    .unwrap();
                            handler.network().send(server_id.clone(), &data);
                        }
                    }
                    "游戏列表" => {
                        let data = bincode::serialize(&C2SMessage::QueryRoomList).unwrap();
                        handler.network().send(server_id.clone(), &data);
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

fn run_readline_thread() -> JoinHandle<()> {
    std::thread::spawn(move || {})
}

fn print_cards(cards: &Vec<Card>) {
    println!("{}", cards_to_string(cards));
}

fn cards_to_string(cards: &Vec<Card>) -> String {
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
                "Connected to server by {} at {}",
                transport,
                server_id.addr()
            );
            println!("Client identified by local port: {}", local_addr.port());
            server_id
        }
        Err(_) => {
            return println!(
                "Can not connect to the server by {} to {}",
                transport, remote_addr
            )
        }
    };

    let user_name = Arc::new(Mutex::new(String::new()));
    let client_state = Arc::new(Mutex::new(ClientState::NotLoggedIn));
    let cards = Arc::new(Mutex::new(vec![]));
    let landlord_name = Arc::new(Mutex::new(String::new()));

    let mutexs = (user_name, client_state, cards, landlord_name);

    handler.signals().send(crate::Signal::Greet);

    let (tx, rx) = channel();

    run_network_thread(server_id.clone(), handler.clone(), listener, tx, mutexs.clone());
    run_console_thread(server_id.clone(), handler.clone(), rx, mutexs.clone()).join();
}

pub mod device;
