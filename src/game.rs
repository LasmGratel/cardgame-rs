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

    /// 最后出牌的玩家
    pub last_index: usize,
}

impl Game {
    pub fn new() -> Game {
        Game {
            players: vec![],
            state: GameState::WaitingForPlayers,
            index: 0,
            landlord_index: 0,
            last_index: 0,
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

    pub fn move_landlord_index(&mut self) {
        self.landlord_index += 1;
        if self.landlord_index > 2 {
            self.landlord_index = 0;
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

    pub fn print_landlord(&self) {
        println!(
            "{} 你要叫地主吗？",
            self.players[self.landlord_index].data.id
        )
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

    pub fn win(&mut self) {
        match self.current_player().player_type {
            PlayerType::Landlord => {
                println!("{} 赢了！", self.current_player().data.id);
            }
            PlayerType::Farmer => {
                for p in self
                    .players
                    .iter()
                    .filter(|x| x.player_type == PlayerType::Farmer)
                {
                    println!("{} 赢了！", self.current_player().data.id);
                }
            }
        }
        self.print_cards();
        self.state = GameState::WaitingForPlayers;
    }

    pub fn pass(&mut self) -> Result<(), &str> {
        if self.state != GameState::Running {
            Ok(())
        } else if self.index == self.last_index {
            Err("过你马呢，该你出牌了")
        } else {
            self.move_index();
            self.print_player();
            Ok(())
        }
    }

    pub fn submit_cards(&mut self, cards: Vec<Card>) -> Result<(), &str> {
        let rule = match_rule(&cards);
        if rule_matches(&self.last_rule, &cards) {
            let option = to_card_groups(&self.current_player().cards) - to_card_groups(&cards);
            if option.is_none() {
                return Err("你手里没有这些牌！");
            }
            print!("{} 出牌：", self.current_player().data.id);
            for c in cards.iter() {
                print!("[{}]", c.to_string());
            }
            println!();

            // 赢得胜利
            if self.current_player().cards.is_empty() {
                self.win();
                return Ok(());
            }

            self.players[self.index].cards = option.unwrap().to_cards();
            self.last_rule = rule;
            self.last_cards = cards;
            self.last_index = self.index;
            self.move_index();

            self.print_cards();
            self.print_player();
            Ok(())
        } else {
            Err("你出的牌不符合规则！")
        }
    }

    pub fn run(&mut self) -> Result<(), &str> {
        if self.players.len() != 3 {
            return Err("玩家数不够!");
        }
        if self.state != GameState::WaitingForLandlord {
            return Err("游戏已经开始!");
        }

        // 发地主牌
        self.index = self.landlord_index;
        self.last_index = self.landlord_index;
        for c in self.landlord_cards.iter() {
            self.players[self.index].cards.push(*c);
        }
        self.players[self.index].cards.sort();
        self.players[self.index].player_type = PlayerType::Landlord;

        self.state = GameState::Running;

        // 显示信息
        self.print_cards();
        self.print_player();

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
