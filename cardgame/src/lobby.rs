use std::collections::HashMap;
use crate::user::User;
use crate::{Game, Player};
use serde::{Serialize, Deserialize};
use std::slice::Iter;

pub struct Room {
    pub name: String,
    pub users: Vec<User>,
    pub game: Game,
    pub state: RoomState
}

#[derive(Serialize, Deserialize, Eq, PartialEq)]
pub enum RoomState {
    Waiting, Ready
}

impl Room {
    pub fn new(name: String) -> Room {
        Room {
            name,
            users: vec![],
            game: Game::new(),
            state: RoomState::Waiting
        }
    }

    pub fn push(&mut self, user: User) {
        self.users.push(user.clone());
        self.game.add_player(Player::new(user));
    }

    /// 开始游戏
    /// 返回第一个叫地主的玩家或房间错误
    pub fn start_game(&mut self) -> Result<(&Player, Iter<Player>), RoomError> {
        if self.state != RoomState::Ready {
            Err(RoomError::NotReady)
        } else {
            self.game.start().map_err(|e| RoomError::NotReady) // TODO
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum RoomError {
    /// 未准备好（人数不足）
    NotReady,

    /// 游戏尚未开始
    NotStarted,

    /// 不是当前正在选择是否叫地主的玩家
    NotLandlordPlayer
}

/// 游戏大厅，用于加入房间和匹配玩家。
/// 生命周期：大约是服务器的生命周期
pub struct Lobby {
    pub users: Vec<User>,
    pub waiting_list: Vec<User>,
    pub rooms: HashMap<String, Room>,
    pub games: HashMap<String, Game>,
}

impl Lobby {
    pub fn new() -> Lobby {
        Lobby {
            users: vec![],
            waiting_list: vec![],
            rooms: HashMap::new(),
            games: HashMap::new(),
        }
    }

    pub fn join_room(&mut self, room_name: &String, user: User) -> Result<&Room, LobbyError> {
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

#[derive(Serialize, Deserialize)]
pub enum LobbyError {
    /// 已经加入房间
    HasJoinedRoom,

    /// 房间已满
    RoomFull,
}