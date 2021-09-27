use std::collections::HashMap;

use message_io::network::{NetworkController, Endpoint, SendStatus};

use cardgame::user::{UserId, UserState};
use cardgame::{Game, Lobby, LobbyError, Room, RoomState, S2CMessage};
use bimap::{BiMap, BiHashMap};

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

    network: NetworkManager<'a>,
}

impl ServerLobby<'_> {
    pub fn new(controller: &NetworkController) -> ServerLobby {
        ServerLobby {
            users: vec![],
            user_states: HashMap::new(),
            waiting_list: vec![],
            rooms: HashMap::new(),
            games: HashMap::new(),
            network: NetworkManager::new(controller),
        }
    }

    pub fn join_room(&mut self, room_name: &str, user: String) -> Result<&Room, LobbyError> {
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

    pub fn get_room(&self, user_id: &str) -> Option<&Room> {
        if let Some(UserState::Playing(room_name)) = self.user_states.get(user_id) {
            self.rooms.get(room_name)
        } else {
            None
        }
    }

    pub fn get_room_mut(&mut self, user_id: &str) -> Option<&mut Room> {
        if let Some(UserState::Playing(room_name)) = self.user_states.get(user_id) {
            self.rooms.get_mut(room_name)
        } else {
            None
        }
    }

    pub fn send_to_room(&self, room_name: &str, message: &S2CMessage) -> Option<()> {
        let room = self.get_room(room_name)?;
        for user in room.users.iter() {
            let status = self.network.send_to_user(user, message)?;
            if status != SendStatus::Sent {
                return None
            }
        }
        Some(())
    }
}

impl Lobby for ServerLobby<'_> {
    fn login(&mut self, user: String) {
        self.users.push(user);
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
    pub user_map: BiMap<UserId, Endpoint>,

    controller: &'a NetworkController,
}

impl NetworkManager<'_> {
    fn new(controller: &NetworkController) -> NetworkManager {
        NetworkManager {
            user_map: BiHashMap::new(),
            controller
        }
    }

    fn connect(&mut self, user_id: UserId, endpoint: Endpoint) {
        self.user_map.insert(user_id, endpoint);
    }

    fn send_to_user(&self, user_id: &str, message: &S2CMessage) -> Option<SendStatus> {
        let endpoint = self.user_map.get_by_left(user_id)?;
        let to_send = bincode::serialize(message).ok()?;
        Some(self.controller.send(*endpoint, &to_send))
    }

    /// Send a message to all clients registered
    ///
    /// # Arguments
    ///
    /// * `message`: message to send
    ///
    /// returns: Result<usize, usize> amount of clients, error index
    ///
    fn send_to_all(&self, message: &S2CMessage) -> Result<usize, usize> {
        let mut i: usize = 0;
        if let Ok(to_send) = bincode::serialize(message) {
            for endpoint in self.user_map.right_values() {
                if self.controller.send(*endpoint, &to_send) == SendStatus::Sent {
                    i += 1;
                } else {
                    return Err(i);
                }
            }
            Ok(i)
        } else {
            Err(0)
        }
    }
}