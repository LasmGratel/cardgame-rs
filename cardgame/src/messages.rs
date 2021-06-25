use crate::player::PlayerData;
use crate::Card;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum S2CMessage {
    UpdateData(PlayerData),
    Pong,
    LoggedIn,
    RoomList(Vec<String>),
    RoomJoined,
    RoomFull,
    GameStarted(Vec<Card>, String),
    GameNotStarted(String),
    LandlordMove(String),
    LordCards(String, Vec<Card>),
}

#[derive(Serialize, Deserialize)]
pub enum C2SMessage {
    SubmitCards(Vec<u8>),
    ChooseLandlord,
    LandlordSelected(String),
    Ping,
    Login(String),
    QueryRoomList,
    JoinRoom(String),
    Matchmaking,
    StartGame(String),
}
