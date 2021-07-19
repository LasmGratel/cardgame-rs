use crate::card::Card;
use serde::{Deserialize, Serialize};
use crate::user::User;

/// 玩家类型
#[derive(PartialEq, Serialize, Deserialize)]
pub enum PlayerType {
    /// 农民
    Farmer,

    /// 地主
    Landlord,
}

/// 游戏玩家
/// 生命周期：整个游戏流程
#[derive(PartialEq)]
pub struct Player<'game> {
    pub user: &'game User,
    pub cards: Vec<Card>,
    pub player_type: PlayerType,
}

impl<'user> Player<'user> {
    pub fn new(user: &'user User) -> Player<'user> {
        Player {
            user,
            cards: vec![],
            player_type: PlayerType::Farmer,
        }
    }
}
