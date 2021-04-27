#[macro_use]
extern crate num_derive;

extern crate regex;

mod card;
mod game;
mod player;
mod rule;

use card::*;
use game::*;
use player::*;
use rule::*;
use std::io;

fn main() {
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
