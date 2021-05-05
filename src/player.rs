use crate::card::Card;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PlayerData {
    pub id: u32,
    pub score: i64,
}

#[derive(PartialEq)]
pub enum PlayerType {
    Farmer,
    Landlord,
}

pub struct Player {
    pub data: PlayerData,
    pub cards: Vec<Card>,
    pub player_type: PlayerType,
}

impl Player {
    pub fn new(id: u32, score: i64) -> Player {
        Player {
            data: PlayerData { id, score },
            cards: vec![],
            player_type: PlayerType::Farmer,
        }
    }
}
