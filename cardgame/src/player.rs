use crate::card::Card;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq)]
pub struct PlayerData {
    pub id: String,
    pub score: i64,
}

#[derive(PartialEq, Serialize, Deserialize)]
pub enum PlayerType {
    Farmer,
    Landlord,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Player {
    pub data: PlayerData,
    pub cards: Vec<Card>,
    pub player_type: PlayerType,
}

impl Player {
    pub fn new(id: String, score: i64) -> Player {
        Player {
            data: PlayerData { id, score },
            cards: vec![],
            player_type: PlayerType::Farmer,
        }
    }
}
