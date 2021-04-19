mod card;
mod game;

use std::io;
use card::*;
use game::*;

fn main() {
    println!("请出牌：");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("not a valid input");
    println!("{}", input);
    for c in parse_input(&input.to_string()).groups {
        println!("{}: {}", c.card.to_string(), c.count);
    }
}
