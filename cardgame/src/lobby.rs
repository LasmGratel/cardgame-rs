use std::collections::HashMap;

#[derive(Clone, PartialEq, Eq)]
pub struct LobbyUser {
    pub name: String,
}

pub struct Room {
    pub name: String,
    pub users: Vec<LobbyUser>,
}

pub struct Lobby {
    pub users: Vec<LobbyUser>,
    pub waiting_list: Vec<LobbyUser>,
    pub rooms: HashMap<String, Room>,
}

impl Lobby {
    pub fn new() -> Lobby {
        Lobby {
            users: vec![],
            waiting_list: vec![],
            rooms: HashMap::new(),
        }
    }
}
