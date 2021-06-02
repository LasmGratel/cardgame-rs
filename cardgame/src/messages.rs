use crate::player::PlayerData;
use crate::Card;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum S2CMessage {
    UpdateCards(Vec<u8>),
    UpdateData(PlayerData),
    Pong,
    LoggedIn,
    RoomList(Vec<String>),
    RoomJoined,
    RoomFull,
    GameStarted(Vec<Card>),
    GameNotStarted(String),
}

#[derive(Serialize, Deserialize)]
pub enum C2SMessage {
    SubmitCards(Vec<u8>),
    ChooseLandlord,
    Ping,
    Login(String),
    QueryRoomList,
    JoinRoom(String),
    Matchmake,
    StartGame(String),
}
