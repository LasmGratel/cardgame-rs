use std::collections::HashMap;

use message_io::network::{NetworkController, Endpoint, SendStatus};

use cardgame::user::{UserId, UserState};
use cardgame::{Game, Lobby, LobbyError, Room, RoomState, S2CMessage, RoomError, GameState, Card, GameError};
use bimap::{BiMap, BiHashMap};
use anyhow::Error;
use crate::server_network::{NetworkManager, MessagePacket, MessageTarget};

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

    pub fn connect(&mut self, user_id: UserId, endpoint: Endpoint) {
        self.login(user_id.clone());
        self.network.connect(user_id.clone(), endpoint);

        match self.user_states.get(&user_id) {
            None => {
                self.user_states.insert(user_id, UserState::Idle);
            }
            Some(state) => {
                match state {
                    UserState::Idle => {
                    }
                    UserState::Matchmaking => {
                        // 断线后取消匹配
                        self.user_states.insert(user_id, UserState::Idle);
                    }
                    UserState::Playing(_room) => {
                        // TODO 断线重连
                    }
                }
            }
        }
    }

    pub fn join_room_by_endpoint(&mut self, room_name: &str, endpoint: Endpoint) -> Result<MessagePacket, LobbyError> {
        self.join_room(room_name, self.get_user(&endpoint).expect("User not found").to_string())
    }

    pub fn join_room(&mut self, room_name: &str, user: UserId) -> Result<MessagePacket, LobbyError> {
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

                match room.start_game() {
                    Ok((landlord_player, players)) => {
                        for player in players {
                            self.network.send_to_user(&player.user, &S2CMessage::GameStarted(player.cards.clone(), landlord_player.user.clone()));
                            self.user_states.insert(player.user.clone(), UserState::Playing(room_name.to_string()));
                        }
                    }
                    Err(err) => {
                        return Err(LobbyError::RoomErr(err));
                    }
                }
            }
            Ok((MessageTarget::Reply, S2CMessage::RoomJoined(room_name.to_string())))
        }
    }

    pub fn start_game(&mut self, room: &mut Room) -> Result<(), Error> {
        let room_name = room.name.clone();
        match room.start_game() {
            Ok((landlord_player, players)) => {
                for player in players {
                    self.network.send_to_user(&player.user, &S2CMessage::GameStarted(player.cards.clone(), landlord_player.user.clone()));
                    self.user_states.insert(player.user.clone(), UserState::Playing(room_name.clone()));
                }
            }
            Err(err) => {
                self.send_to_room(&room.name, &S2CMessage::RoomErr(err.clone()));
                return Err(Error::from(err));
            }
        }
        Ok(())
    }

    pub fn start_game_by_name(&mut self, room_name: &str) -> Result<&mut Room, RoomError> {
        let room = self.rooms.get_mut(room_name);
        if room.is_none() {
            return Err(RoomError::NotReady);
        }
        let room = room.unwrap();
        match room.start_game() {
            Ok((landlord_player, players)) => {
                for player in players {
                    self.network.send_to_user(&player.user, &S2CMessage::GameStarted(player.cards.clone(), landlord_player.user.clone()));
                    self.user_states.insert(player.user.clone(), UserState::Playing(room_name.to_string()));
                }
            }
            Err(err) => {
                return Err(err);
            }
        }
        Ok(room)
    }

    pub fn rematch_vote(&self, user: &str, vote: bool) -> Result<(u32, bool), RoomError> {
        if let Some(room) = self.get_room_by_user(user) {
            let current_state = room.state.clone();
            if let RoomState::WaitingForRematch(count) = current_state {
                return if vote {
                    Ok((count + 1, vote))
                } else {
                    Ok((count, vote))
                }
            }
        }
        Err(RoomError::NotReady)
    }

    pub fn send_packet(&self, packet: MessagePacket) -> Option<SendStatus> {
        match packet.0 {
            MessageTarget::All => {
                self.network.send_to_all(&packet.1).ok().map(|_| SendStatus::Sent)
            }
            MessageTarget::Reply => {
                None
            }
            MessageTarget::Endpoint(endpoint) => {
                self.network.send_to_endpoint(&packet.1, endpoint)
            }
            MessageTarget::User(user) => {
                self.network.send_to_user(&user, &packet.1)
            }
            MessageTarget::Room(room) => {
                self.send_to_room(&room, &packet.1)
            }
        }
    }

    pub fn submit_cards(&mut self, endpoint: &Endpoint, cards: Vec<Card>) -> Result<String, GameError> {
        let user_id = self.network.user_map.get_by_right(endpoint).unwrap().to_string();
        let room = self.get_room_by_endpoint_mut(endpoint).unwrap();
        return if room.game.state != GameState::Running {
            Err(GameError::NotRunning)
        } else if room.game.current_player().user != user_id {
            Err(GameError::NotYourTurn)
        } else {
            room.game.submit_cards(cards)
        }
    }

    pub fn choose_landlord_by_endpoint(&mut self, endpoint: &Endpoint, choose: bool) -> Result<MessagePacket, Error> {
        let user = self.get_user(endpoint).expect("No user found by endpoint").clone();
        if let Some(room) = self.get_room_by_user_mut(&user) {
            // 尚未开始叫地主阶段
            if room.game.state != GameState::WaitingForLandlord {
                return Err(Error::from(RoomError::NotStarted));
            }
            // 不是当前的地主玩家
            if room.game.players[room.game.landlord_index].user != user {
                return Err(Error::from(RoomError::NotLandlordPlayer));
            }
            if choose { // 叫地主
                room.game.run().expect("Game cannot run");
                // 通知所有玩家地主人选

                Ok((MessageTarget::Room(room.name.clone()), S2CMessage::LordCards(room.game.current_player().user.clone(), room.game.landlord_cards.clone())))
            } else { // 不叫
                room.game.move_landlord_index();
                // 通知下一个地主

                Ok((MessageTarget::Room(room.name.clone()), S2CMessage::LandlordMove(room.game.landlord_player().user.clone())))
            }
        } else {
            // 房间还未准备好
            Err(Error::from(RoomError::NotReady))
        }
    }

    pub fn get_user(&self, endpoint: &Endpoint) -> Option<&UserId> {
        self.network.user_map.get_by_right(endpoint)
    }

    pub fn get_room_by_user(&self, user_id: &str) -> Option<&Room> {
        if let Some(UserState::Playing(room_name)) = self.user_states.get(user_id) {
            self.rooms.get(room_name)
        } else {
            None
        }
    }

    pub fn get_room_by_user_mut(&mut self, user_id: &str) -> Option<&mut Room> {
        if let Some(UserState::Playing(room_name)) = self.user_states.get(user_id) {
            self.rooms.get_mut(room_name)
        } else {
            None
        }
    }

    pub fn get_room_by_endpoint(&self, endpoint: &Endpoint) -> Option<&Room> {
        self.get_room_by_user(self.get_user(endpoint)?)
    }

    pub fn get_room_by_endpoint_mut(&mut self, endpoint: &Endpoint) -> Option<&mut Room> {
        let user = self.get_user(endpoint)?.to_string();
        self.get_room_by_user_mut(&user)
    }

    pub fn send_to_room(&self, room_name: &str, message: &S2CMessage) -> Option<SendStatus> {
        let room = self.get_room_by_user(room_name)?;
        for user in room.users.iter() {
            let status = self.network.send_to_user(user, message)?;
            if status != SendStatus::Sent {
                return None
            }
        }
        Some(SendStatus::Sent)
    }
}

impl Lobby for ServerLobby<'_> {
    fn login(&mut self, user: String) {
        if !self.user_states.contains_key(&user) {
             self.user_states.insert(user.clone(), UserState::Idle);
        }
        self.users.push(user);
    }

    fn disconnect(&mut self, user: &str) {
        if let Some(UserState::Playing(game_name)) = self.user_states.get(user) {
            self.rooms.remove(game_name);
        }
        if let Some(pos) = self.users.iter().position(|x| x == user) {
            self.users.remove(pos);
            self.network.user_map.remove_by_left(user);
        }
    }
}