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

    let to_match = parse_input("3334445577");

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
    let rule = match_rule(&to_match);
    println!("{}, {}", rule.to_string(), match_rule(&cards).to_string());
    println!("{}", rule_matches(&rule, &cards));

    let mut game = Game::new();
    game.players.push(Player::new(114514, 0));
    game.players.push(Player::new(1919810, 0));
    game.players.push(Player::new(123456, 0));
    game.start().expect("Game not started!");
}
