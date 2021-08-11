use cardgame::{Lobby, Room, Game, LobbyError, RoomState};
use cardgame::user::{User, UserId};
use std::collections::HashMap;

pub struct ServerLobby {
    /// 大厅玩家列表
    pub users: Vec<UserId>,

    /// 匹配中的玩家列表
    pub waiting_list: Vec<UserId>,

    /// 所有房间
    pub rooms: HashMap<String, Room>,

    /// 所有进行中的游戏
    pub games: HashMap<String, Game>,
}

impl ServerLobby {
    pub fn new() -> ServerLobby {
        ServerLobby {
            users: vec![],
            waiting_list: vec![],
            rooms: HashMap::new(),
            games: HashMap::new(),
        }
    }

    pub fn join_room(&mut self, room_name: &String, user: String) -> Result<&Room, LobbyError> {
        if !self.rooms.contains_key(room_name) {
            self.rooms.insert(room_name.clone(), Room::new(room_name.clone()));
            println!("创建房间: {}", room_name);
        };
        let mut room = self.rooms.get_mut(room_name).unwrap();
        if room.users.contains(&user) {
            Err(LobbyError::HasJoinedRoom)
        } else if room.users.len() == 3 {
            Err(LobbyError::RoomFull)
        } else {
            room.push(user);

            if room.users.len() == 3 {
                room.state = RoomState::Ready;
            }
            Ok(room)
        }
    }
}

impl Lobby for ServerLobby {
    fn login(&mut self, user: String) {
        self.users.push(user);
    }

    fn disconnect(&mut self, user: &String) {
        if let Some(pos) = self.users.iter().position(|x| x == user) {
            self.users.remove(pos);
        }
    }
}