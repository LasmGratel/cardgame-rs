use crate::card::*;
use crate::player::*;
use crate::rule::*;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use rand::Rng;
use regex::Regex;
use serde::{Serialize, Deserialize};
use std::slice::Iter;
use crate::user::UserId;

/// 基础积分
const BASE_POINTS: u32 = 100;

#[derive(PartialEq, Eq)]
pub enum GameState {
    WaitingForPlayers,
    WaitingForLandlord,
    Running,
}

pub struct Game {
    /// 玩家
    pub players: Vec<Player>,

    /// 游戏状态
    pub state: GameState,

    /// 当前出牌玩家
    pub index: usize,

    /// 地主玩家
    pub landlord_index: usize,

    /// 最后出牌的玩家
    pub last_index: usize,

    /// 最后出的牌
    pub last_cards: Vec<Card>,

    /// 最后出牌所匹配的规则
    pub last_rule: Box<dyn Rule>,

    /// 地主牌
    pub landlord_cards: Vec<Card>,

    /// 积分倍率
    /// 炸弹，火箭会*=2
    /// 加倍 *=2，超级加倍 *=4
    pub score_multiplier: u32,
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
            score_multiplier: 1,
        }
    }

    /// “上桌”
    pub fn add_player(&mut self, player: Player) -> bool {
        if self.players.len() < 3 {
            self.players.push(player);
            true
        } else {
            false
        }
    }

    /// 获取当前玩家的引用
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

    #[cfg(debug_assertions)]
    pub fn print_cards(&self) {
        for p in self.players.iter() {
            print!("{}: ", p.user);
            for c in p.cards.iter() {
                print!("[{}]", c.to_string());
            }
            println!();
        }
    }

    #[cfg(not(debug_assertions))]
    pub fn print_cards(&self) {}

    #[cfg(debug_assertions)]
    pub fn print_player(&self) {
        println!("轮到 {} 出牌", self.current_player().user);
    }

    #[cfg(not(debug_assertions))]
    pub fn print_player(&self) {}

    pub fn landlord_player(&self) -> &Player {
        &self.players[self.landlord_index]
    }

    pub fn start(&mut self) -> Result<(&Player, Iter<Player>), &str> {
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

        Ok((&self.players[self.landlord_index], self.players.iter()))
    }

    pub fn win(&mut self) -> GameError {
        #[cfg(debug_assertions)]
        match self.current_player().player_type {
            PlayerType::Landlord => {
                println!("地主赢了！");
            }
            PlayerType::Farmer => {
                println!("农民赢了！");
            }
        }
        self.print_cards();

        self.state = GameState::WaitingForPlayers;
        GameError::Win(self.current_player().user.clone(), self.current_player().player_type.clone(), BASE_POINTS * self.score_multiplier)
    }

    pub fn pass(&mut self) -> Result<String, GameError> {
        if self.state != GameState::Running {
            Err(GameError::NotRunning)
        } else if self.index == self.last_index {
            Err(GameError::YourTurn)
        } else {
            self.move_index();
            self.print_player();
            Ok(self.current_player().user.clone())
        }
    }

    pub fn submit_cards(&mut self, cards: Vec<Card>) -> Result<String, GameError> {
        let rule = match_rule(&cards);
        if rule.is_none() {
            Err(GameError::NoRule)
        } else if rule_matches(&self.last_rule, &cards) || self.index == self.last_index {
            let option = to_card_groups(&self.current_player().cards) - to_card_groups(&cards);
            if option.is_none() {
                return Err(GameError::NoSuchCards);
            }

            #[cfg(debug_assertions)] {
                print!("{} 出牌：", self.current_player().user);
                for c in cards.iter() {
                    print!("[{}]", c.to_string());
                }
                println!();
            }

            // 炸弹积分翻倍
            if rule.bomb_priority() == 1 || rule.bomb_priority() == 2 {
                self.score_multiplier *= 2;
            }

            self.players[self.index].cards = option.unwrap().into_cards();

            // 赢得胜利
            if self.current_player().cards.is_empty() {
                return Err(self.win());
            }

            self.last_rule = rule;
            self.last_cards = cards;
            self.last_index = self.index;

            self.move_index();

            self.print_cards();
            self.print_player();
            Ok(self.current_player().user.clone())
        } else {
            Err(GameError::WrongRule)
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

        // 显示调试信息
        self.print_cards();
        self.print_player();

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum GameError {
    NotRunning, NotYourTurn, NoSuchCards, WrongRule, NoRule,

    /// 这把赢了
    /// 参数：最后出掉牌的玩家，玩家类型，获得的积分
    Win(UserId, PlayerType, u32),

    /// 过你马呢
    YourTurn
}

pub fn gen_cards() -> Vec<Card> {
    let mut cards: Vec<Card> = vec![];
    for i in 1..14 {
        for _ in 0..4 {
            cards.push(Card::from_value(i));
        }
    }
    cards.push(Card::CardGhost);
    cards.push(Card::CardKing);

    cards.shuffle(&mut thread_rng());
    return cards;
}

pub fn parse_input(input: &str) -> Option<Vec<Card>> {
    let input = input.to_ascii_uppercase().replace("10", "ß");
    let input = input.replace("1", "0");
    let input = input.replace("ß", "1");
    let cards_regex = Regex::new("([1-9jqkaJQKA]|鬼|王)+").unwrap();
    let result = cards_regex.find(input.as_str())?;
    let result = result.as_str().replace("10", "1");

    let mut vec: Vec<Card> = Vec::new();
    for c in result.chars() {
        vec.push(Card::from_char(&c));
    }
    vec.retain(|&i| i != Card::Unknown);
    if vec.is_empty() {
        None
    } else {
        Some(vec)
    }
}
