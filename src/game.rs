use crate::card::*;
use crate::player::*;
use crate::rule::*;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use rand::Rng;
use regex::Regex;

#[derive(PartialEq, Eq)]
pub enum GameState {
    WaitingForPlayers,
    WaitingForLandlord,
    Running,
}

pub struct Game {
    pub players: Vec<Player>,
    pub state: GameState,
    pub index: usize,
    pub landlord_index: usize,
    pub last_cards: Vec<Card>,
    pub landlord_cards: Vec<Card>,
    pub last_rule: Box<dyn Rule>,
}

impl Game {
    pub fn new() -> Game {
        Game {
            players: vec![],
            state: GameState::WaitingForPlayers,
            index: 0,
            landlord_index: 0,
            last_cards: vec![],
            landlord_cards: vec![],
            last_rule: Box::new(RuleNone),
        }
    }

    pub fn add_player(&mut self, player: Player) -> bool {
        if self.players.len() < 3 {
            self.players.push(player);
            true
        } else {
            false
        }
    }

    pub fn current_player(&self) -> &Player {
        &self.players[self.index]
    }

    pub fn move_index(&mut self) {
        self.index += 1;
        if self.index > 2 {
            self.index = 0;
        }
    }

    pub fn print_cards(&self) {
        for p in self.players.iter() {
            print!("{}: ", p.data.id);
            for c in p.cards.iter() {
                print!("[{}]", c.to_string());
            }
            println!();
        }
    }

    pub fn print_player(&self) {
        println!("轮到 {} 出牌", self.current_player().data.id);
    }

    pub fn start(&mut self) -> Result<(), &str> {
        if self.players.len() != 3 {
            return Err("玩家数不够!");
        }
        if self.state != GameState::WaitingForPlayers {
            return Err("游戏已经开始!");
        }

        self.state = GameState::WaitingForLandlord;
        self.landlord_index = thread_rng().gen_range(0..3);

        let cards = gen_cards();

        for i in 0..3 {
            for j in 0..18 {
                self.players[i].cards.push(cards[i * 17 + j]);
            }
            self.players[i].cards.sort();
        }
        for i in 51..54 {
            self.landlord_cards.push(cards[i]);
        }
        self.print_cards();

        Ok(())
    }

    pub fn submit_cards(&mut self, cards: Vec<Card>) -> bool {
        if self.current_player().cards.inter
        let rule = match_rule(&cards);
        if rule_matches(&self.last_rule, &cards) {
            self.last_rule = rule;
            self.last_cards = cards;
            for c in cards.iter() {
                if let Some(pos) = vec.iter().position(|x| *x == needle) {
                    vec.remove(pos);
                }
            }
            self.move_index();
            true
        } else {
            false
        }
    }

    pub fn run(&mut self) -> Result<(), &str> {
        if self.players.len() != 3 {
            return Err("玩家数不够!");
        }
        if self.state != GameState::WaitingForLandlord {
            return Err("游戏已经开始!");
        }

        self.index = self.landlord_index;

        Ok(())
    }
}

pub fn gen_cards() -> Vec<Card> {
    let mut cards: Vec<Card> = vec![];
    for i in 1..14 {
        for _ in 0..4 {
            cards.push(Card::from_value(&i));
        }
    }
    cards.push(Card::CardGhost);
    cards.push(Card::CardKing);

    cards.shuffle(&mut thread_rng());
    return cards;
}

pub fn parse_input(input: &str) -> Vec<Card> {
    let input = input.to_ascii_uppercase().replace("10", "ß");
    let input = input.replace("1", "0");
    let input = input.replace("ß", "1");
    let cards_regex = Regex::new("([1-9jqkaJQKA]|鬼|王)+").unwrap();
    let result = cards_regex.find(input.as_str()).expect("Cards not valid");
    let result = result.as_str().replace("10", "1");

    let mut vec: Vec<Card> = Vec::new();
    for c in result.chars() {
        vec.push(Card::from_char(&c));
    }
    vec.retain(|&i| i != Card::Unknown);
    vec
}
