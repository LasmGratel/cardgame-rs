use cardgame::Room;

pub struct Lobby {
    pub rooms: Vec<Room>
}

impl Default for Lobby {
    fn default() -> Self {
        Lobby {
            rooms: vec![]
        }
    }
}