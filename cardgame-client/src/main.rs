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

fn run_network_thread(
    server_id: Endpoint,
    handler: NodeHandler<Signal>,
    listener: NodeListener<Signal>,
    sender: Sender<S2CMessage>,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
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
                    sender.send(message).expect("Cannot send message");
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
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let line = read_line("请输入用户名：");
        let line = line.trim();
        if line == "exit" {
            return;
        }
        let data = bincode::serialize(&C2SMessage::Login(String::from(line))).unwrap();
        handler.network().send(server_id.clone(), &data);
        let msg = rx.recv().unwrap();
        match msg {
            S2CMessage::LoggedIn => {
                println!("Logged in!");
            }
            _ => {
                println!("Unknown message from server");
                return;
            }
        }
        loop {
            let line = read_line("请输入用户名：");
            let line = line.trim();
            if line == "exit" {
                return;
            }

            match line {
                "游戏列表" => {
                    let data = bincode::serialize(&C2SMessage::QueryLobbyList).unwrap();
                    handler.network().send(server_id.clone(), &data);
                    let msg = rx.recv().unwrap();
                    if let S2CMessage::LobbyList(lobbies) = msg {
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
    })
}

fn run() {
    println!("Len: {}", gen_cards().len());

    let mut game = Game::new();
    game.players.push(Player::new(114514, 0));
    game.players.push(Player::new(1919810, 0));
    game.players.push(Player::new(123456, 0));
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
}

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

    handler.signals().send(crate::Signal::Greet);

    let (tx, rx) = channel();

    run_network_thread(server_id.clone(), handler.clone(), listener, tx);
    run_console_thread(server_id.clone(), handler.clone(), rx).join();
}
