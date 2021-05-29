use std::collections::HashMap;

pub struct LobbyUser {
    pub name: String,
}

pub struct LobbyTable {
    pub name: String,
}

pub struct Lobby {
    pub users: Vec<LobbyUser>,
    pub waiting_list: Vec<LobbyUser>,
    pub table: HashMap<String, LobbyTable>,
}

impl Lobby {
    pub fn new() -> Lobby {
        Lobby {
            users: vec![],
            waiting_list: vec![],
            table: HashMap::new(),
        }
    }
}
