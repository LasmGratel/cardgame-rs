use crate::{Card, LobbyError, RoomError, GameError};
use serde::{Deserialize, Serialize};
use crate::user::User;

#[derive(Serialize, Deserialize)]
pub enum S2CMessage {
    UpdateData(User),
    Pong,
    LoggedIn,
    RoomList(Vec<String>),
    RoomJoined(String),
    RoomFull,
    RoomErr(RoomError),
    LobbyErr(LobbyError),
    GameErr(GameError),
    GameStarted(Vec<Card>, String),
    GameNotStarted(String),
    LandlordMove(String),
    LordCards(String, Vec<Card>),
    CardsSubmitted(String, Vec<Card>),

    /// 该谁出牌
    Move(String),
}

#[derive(Serialize, Deserialize)]
pub enum C2SMessage {
    /// 出牌
    SubmitCards(Vec<Card>),

    /// 过牌
    Pass,

    ChooseLandlord(bool),
    Ping,
    Login(String),
    QueryRoomList,
    JoinRoom(String),
    Matchmaking,
    StartGame(String),
}
