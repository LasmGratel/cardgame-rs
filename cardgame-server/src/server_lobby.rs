use std::collections::HashMap;

use message_io::network::NetworkController;

use cardgame::user::{UserId, UserState};
use cardgame::{Game, Lobby, LobbyError, Room, RoomState};

/// 大厅的服务器实现。
pub struct ServerLobby<'a> {
    /// 大厅玩家列表
    pub users: Vec<UserId>,

    /// 玩家状态
    pub user_states: HashMap<UserId, UserState>,

    /// 匹配中的玩家列表
    pub waiting_list: Vec<UserId>,

    /// 所有房间
    pub rooms: HashMap<String, Room>,

    /// 所有进行中的游戏
    pub games: HashMap<String, Game>,

    network: &'a NetworkController,
}

impl ServerLobby<'_> {
    pub fn new(network: &NetworkController) -> ServerLobby {
        ServerLobby {
            users: vec![],
            user_states: HashMap::new(),
            waiting_list: vec![],
            rooms: HashMap::new(),
            games: HashMap::new(),
            network,
        }
    }

    pub fn join_room(&mut self, room_name: &String, user: String) -> Result<&Room, LobbyError> {
        if !self.rooms.contains_key(room_name) {
            self.rooms
                .insert(room_name.to_string(), Room::new(room_name.to_string()));
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

impl Lobby for ServerLobby<'_> {
    fn login(&mut self, user: String) {
        self.users.push(user.clone());
        // if !self.user_states.contains_key(&user) {
        //     self.user_states.insert(user, UserState::Idle);
        // }
    }

    fn disconnect(&mut self, user: &str) {
        if let Some(pos) = self.users.iter().position(|x| x == user) {
            self.users.remove(pos);
        }
    }
}

pub struct NetworkManager<'a> {
    network: &'a NetworkController,
}
