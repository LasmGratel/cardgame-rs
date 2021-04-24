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

    println!("请出牌：");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("not a valid input");
    let cards: Vec<Card> = parse_input(input.as_str());
    for c in cards.iter() {
        print!("[{}]", c.to_string());
    }
    println!();
    println!("{}", match_rule(cards).to_string());

    let mut game = Game::new();
    game.players.push(Player {
        data: PlayerData { id: 1, score: 1 },
        cards: vec![],
        player_type: PlayerType::Farmer,
    });
}
