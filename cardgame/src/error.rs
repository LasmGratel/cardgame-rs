use std::fmt::{Debug, Display, Formatter};
use serde::{Serialize, Deserialize};
use crate::PlayerType;
use crate::user::UserId;

#[derive(Serialize, Deserialize, Clone, PartialEq, thiserror::Error)]
pub enum RoomError {
    /// 未准备好（人数不足）
    NotReady,

    /// 游戏尚未开始
    NotStarted,

    /// 不是当前正在选择是否叫地主的玩家
    NotLandlordPlayer,

    /// 房间已满
    RoomFull,
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
            RoomError::RoomFull => {
                write!(f, "RoomError::RoomFull")
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
            RoomError::RoomFull => {
                write!(f, "房间已满")
            }
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, thiserror::Error)]
pub enum LobbyError {
    /// 已经加入房间
    HasJoinedRoom,

    /// 房间已满
    RoomFull,

    RoomErr(RoomError),
    OtherError
}

impl Display for LobbyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            LobbyError::RoomFull => {
                write!(f, "Room is full")
            }
            _ => {
                write!(f, "Lobby error")
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, thiserror::Error)]
pub enum GameError {
    NotEnoughPlayers,
    NotRunning,
    IsRunning,
    NotReady,
    NotYourTurn,
    NoSuchCards,
    WrongRule,
    NoRule,

    /// 这把赢了
    /// 参数：最后出掉牌的玩家，玩家类型，获得的积分
    Win(UserId, PlayerType, u32),

    /// 过你马呢
    YourTurn
}

impl Display for GameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            _ => {
                write!(f, "Game error")
            }
        }
    }
}