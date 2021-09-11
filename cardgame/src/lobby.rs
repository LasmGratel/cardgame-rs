use crate::user::UserId;
use crate::{Game, Player};
use serde::{Serialize, Deserialize};
use std::slice::Iter;

pub struct Room {
    pub name: String,
    pub users: Vec<UserId>,
    pub game: Game,
    pub game_name: String,
    pub state: RoomState,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum RoomState {
    Waiting, Ready, WaitingForRematch(u32)
}

impl Room {
    pub fn new(name: String) -> Room {
        Room {
            name,
            users: vec![],
            game: Game::new(),
            game_name: String::new(),
            state: RoomState::Waiting
        }
    }

    pub fn push(&mut self, user: UserId) {
        self.users.push(user.clone());
        self.game.add_player(Player::new(user));
    }

    /// 开始游戏
    /// 返回第一个叫地主的玩家或房间错误
    pub fn start_game(&mut self) -> Result<(&Player, Iter<Player>), RoomError> {
        if self.state != RoomState::Ready {
            Err(RoomError::NotReady)
        } else {
            self.game.start().map_err(|e| RoomError::NotReady) // TODO 处理错误
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum RoomError {
    /// 未准备好（人数不足）
    NotReady,

    /// 游戏尚未开始
    NotStarted,

    /// 不是当前正在选择是否叫地主的玩家
    NotLandlordPlayer
}

/// 游戏大厅，用于加入房间和匹配玩家。
pub trait Lobby {
    /// 玩家登入
    fn login(&mut self, user: String);

    /// 玩家断连
    fn disconnect(&mut self, user: &str);
}

#[derive(Serialize, Deserialize)]
pub enum LobbyError {
    /// 已经加入房间
    HasJoinedRoom,

    /// 房间已满
    RoomFull,
}