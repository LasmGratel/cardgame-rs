extern crate cardgame;

use cardgame::*;
use cardgame::messages::*;
use message_io::network::*;
use message_io::node;
use message_io::node::*;
use std::time::Duration;
use std::sync::mpsc;
use std::io;
use std::thread::JoinHandle;
use cardgame::Signal;

fn run_network_thread(server_id: Endpoint, handler: NodeHandler<Signal>, listener: NodeListener<Signal>) -> JoinHandle<()> {
    std::thread::spawn(move || {
        listener.for_each(move |event| match event {
            NodeEvent::Signal(signal) => match signal {
                crate::Signal::Greet => {
                    let message = C2SMessage::Ping;
                    let output_data = bincode::serialize(&message).unwrap();
                    handler.network().send(server_id, &output_data);
                    handler
                        .signals()
                        .send_with_timer(crate::Signal::Greet, Duration::from_secs(1));
                }
            },
            NodeEvent::Network(net_event) => match net_event {
                NetEvent::Message(_, input_data) => {
                    let message: S2CMessage = bincode::deserialize(&input_data).unwrap();
                    match message {
                        S2CMessage::Pong => {
                            println!("Pong from server")
                        }
                        S2CMessage::Pu => {
                            println!("Pu!")
                        }
                        _ => {
                            println!("Unknown message")
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

fn run_console_thread(server_id: Endpoint, handler: NodeHandler<Signal>) -> JoinHandle<()> {
    std::thread::spawn(move || {
        loop {
            let mut line = String::new();
            std::io::stdin().read_line(&mut line).expect("Invalid input");

            let line = line.trim();
            if line == "exit" {
                break;
            }

            handler.network().send(server_id.clone(), line.as_bytes());
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

    run_network_thread(server_id.clone(), handler.clone(), listener);
    run_console_thread(server_id.clone(), handler.clone()).join();
}
