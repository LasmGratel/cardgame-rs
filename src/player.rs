use crate::card::Card;

pub struct PlayerData {
    pub id: u32,
    pub score: i64,
}

pub enum PlayerType {
    Farmer,
    Landlord,
}

pub struct Player {
    pub data: PlayerData,
    pub cards: Vec<Card>,
    pub player_type: PlayerType,
}
