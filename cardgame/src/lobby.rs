use crate::user::UserId;
use crate::{Game, Player};
use serde::{Serialize, Deserialize};
use std::slice::Iter;
use std::error::Error;
use std::fmt::{Debug, Formatter, Display};

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
            self.game.start().map_err(|_| RoomError::NotReady) // TODO 处理错误
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum RoomError {
    /// 未准备好（人数不足）
    NotReady,

    /// 游戏尚未开始
    NotStarted,

    /// 不是当前正在选择是否叫地主的玩家
    NotLandlordPlayer
}

impl Debug for RoomError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            RoomError::NotReady => {
                write!(f, "RoomError::NotReady")
            }
            RoomError::NotStarted => {
                write!(f, "RoomError::NotStarted")
            }
            RoomError::NotLandlordPlayer => {
                write!(f, "RoomError::NotLandlordPlayer")
            }
        }
    }
}

impl Display for RoomError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            RoomError::NotReady => {
                write!(f, "房间未准备好")
            }
            RoomError::NotStarted => {
                write!(f, "游戏尚未开始")
            }
            RoomError::NotLandlordPlayer => {
                write!(f, "不是你叫地主")
            }
        }
    }
}

impl Error for RoomError {}

/// 游戏大厅，用于加入房间和匹配玩家。
pub trait Lobby {
    /// 玩家登入
    fn login(&mut self, user: String);

    /// 玩家断连
    fn disconnect(&mut self, user: &str);
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum LobbyError {
    /// 已经加入房间
    HasJoinedRoom,

    /// 房间已满
    RoomFull,

    RoomErr(RoomError),
    OtherError
}