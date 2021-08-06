use cardgame::Player;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct PlayerManager {
    path: String,
    cached_players: HashMap<String, Player>
}

impl PlayerManager {
    pub fn new() -> PlayerManager {
        let path = Path::new("./users");
        if !path.is_dir() {
            fs::create_dir(path);
        }
        PlayerManager {
            path: String::from("./users"),
            cached_players: HashMap::new()
        }
    }

    pub fn get_player(&self, id: &String) -> Option<&Player> {
        return self.cached_players.get(id);
    }

    pub fn get_or_create_player(&mut self, id: &String) -> &Player {
        return if self.cached_players.contains_key(id) {
            self.cached_players.get(id).unwrap()
        } else {
            let mut player = Player::new(id.clone(), 0);
            self.cached_players.insert(id.clone(), player);
            self.cached_players.get(id).unwrap()
        }
    }
}