use crate::player::PlayerData;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum S2CMessage {
    UpdateCards(Vec<u8>),
    UpdateData(PlayerData),
    Pong,
    Pu,
    LoggedIn,
    LobbyList(Vec<String>),
}

#[derive(Serialize, Deserialize)]
pub enum C2SMessage {
    SubmitCards(Vec<u8>),
    ChooseLandlord,
    Ping,
    Login(String),
    QueryLobbyList,
}
