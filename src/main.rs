#[macro_use]
extern crate num_derive;
extern crate bincode;
#[macro_use]
extern crate log;
extern crate message_io;
extern crate rand;
extern crate regex;
#[macro_use]
extern crate uuid;

mod card;
mod client;
mod game;
mod messages;
mod player;
mod rule;
mod server;

use card::*;
use client::client::run_client;
use game::*;
use messages::*;
use player::*;
use server::server::run_server;
use std::io;
use std::net::SocketAddr;
use std::time::Duration;

pub enum Signal {
    // This is a self event called every second.
    Greet,
    // Other signals here,
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
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 {
        match args[1].as_str().trim() {
            "server" => run_server(),
            "client" => run_client(),
            _ => {}
        }
    }
}
