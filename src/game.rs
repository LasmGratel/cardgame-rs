extern crate regex;

use regex::Regex;
use crate::card::*;

pub fn parse_input(input: &str) -> CardGroups {
    let cards_regex = Regex::new("([2-9jqkaJQKA]|10|鬼|王)+").unwrap();
    let result = cards_regex.find(input).expect("Cards not valid");
    let result = result.as_str().replace("10", "1");

    let mut vec: Vec<Card> = Vec::new();
    for c in result.chars() {
        vec.push(Card::from_char(c));
    }
    vec.retain(|&i| i != Card::Unknown);
    to_card_groups(&vec)
}