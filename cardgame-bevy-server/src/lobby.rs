use std::collections::HashMap;

use cardgame::user::{UserId, UserState};
use cardgame::{Card, Game, GameState, Lobby, Room, RoomState};
use bimap::{BiHashMap, BiMap};
use anyhow::Error;
use bevy_spicy_networking::{ConnectionId, NetworkServer};
use bevy_spicy_networking::NetworkError;
use cardgame::error::{GameError, LobbyError, RoomError};
use cardgame_common::message::S2CMessage;
use crate::server_network::{MessagePacket, MessageTarget, NetworkManager};

/// 大厅的服务器实现。
#[derive(Default)]
pub struct ServerLobby {
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

    network: NetworkManager,
}

impl ServerLobby {
    pub fn connect(&mut self, user_id: UserId, connection_id: ConnectionId) {
        self.login(user_id.clone());
        self.network.connect(user_id.clone(), connection_id);

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

    pub fn join_room_by_connection_id(&mut self, net: &NetworkServer, room_name: &str, connection_id: ConnectionId) -> Result<(), LobbyError> {
        self.join_room(net, room_name, self.get_user(&connection_id).expect("User not found").to_string())
    }

    pub fn join_room(&mut self, net: &NetworkServer, room_name: &str, user: UserId) -> Result<(), LobbyError> {
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
            room.push(user.clone());
            self.user_states.insert(user.to_string(), UserState::Playing(room_name.to_string()));

            if room.users.len() == 3 {
                room.state = RoomState::Ready;

                match room.start_game() {
                    Ok((landlord_player, players)) => {
                        for player in players {
                            self.network.send_to_user(net, &player.user, S2CMessage::GameStarted(player.cards.clone(), landlord_player.user.clone()));
                        }
                    }
                    Err(err) => {
                        return Err(LobbyError::RoomErr(err));
                    }
                }
            }
            self.network.send_to_user(net, &user, S2CMessage::RoomJoined(room_name.to_string()));
            Ok(())
        }
    }

    pub fn start_game(&mut self, net: &NetworkServer, room: &mut Room) -> Result<(), Error> {
        let room_name = room.name.clone();
        match room.start_game() {
            Ok((landlord_player, players)) => {
                for player in players {
                    self.network.send_to_user(net, &player.user, S2CMessage::GameStarted(player.cards.clone(), landlord_player.user.clone()));
                    self.user_states.insert(player.user.clone(), UserState::Playing(room_name.clone()));
                }
            }
            Err(err) => {
                self.send_to_room_by_name(net, &room.name, S2CMessage::RoomErr(err.clone()));
                return Err(Error::from(err));
            }
        }
        Ok(())
    }

    pub fn send_message(&self, net: &NetworkServer, user: &UserId, message: S2CMessage) {
        if let Some(conn) = self.network.user_map.get_by_left(user) {
            net.send_message(conn.clone(), message);
        }
    }

    pub fn start_game_by_name(&mut self, net: &NetworkServer, room_name: &str) -> Result<&mut Room, RoomError> {
        let room = self.rooms.get_mut(room_name);
        if room.is_none() {
            return Err(RoomError::NotReady);
        }
        let room = room.unwrap();
        match room.start_game() {
            Ok((landlord_player, players)) => {
                for player in players {
                    self.network.send_to_user(net, &player.user, S2CMessage::GameStarted(player.cards.clone(), landlord_player.user.clone()));
                    self.user_states.insert(player.user.clone(), UserState::Playing(room_name.to_string()));
                }
            }
            Err(err) => {
                return Err(err);
            }
        }
        Ok(room)
    }

    pub fn rematch_vote(&self, conn: &ConnectionId, vote: bool) -> Result<(u32, bool), RoomError> {
        if let Some(user) = self.network.user_map.get_by_right(conn) {
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
        }
        Err(RoomError::NotReady)
    }

    pub fn send_packet(&self, net: &NetworkServer, packet: MessagePacket) -> Result<(), Error> {
        match packet.0 {
            MessageTarget::All => {
                self.network.send_to_all(net, packet.1)
            }
            MessageTarget::Reply => {
                Ok(())
            }
            MessageTarget::ConnectionId(connection_id) => {
                net.send_message(connection_id, packet.1)
            }
            MessageTarget::User(user) => {
                self.network.send_to_user(net, &user, packet.1)
            }
            MessageTarget::Room(room) => {
                self.send_to_room_by_name(net, &room, packet.1)
            }
        }.map_err(|x| Error::from(x))
    }

    pub fn submit_cards(&mut self, connection_id: &ConnectionId, cards: Vec<Card>) -> Result<String, GameError> {
        let user_id = self.network.user_map.get_by_right(connection_id).unwrap().to_string();
        let room = self.get_room_by_connection_id_mut(connection_id).unwrap();
        return if room.game.state != GameState::Running {
            Err(GameError::NotRunning)
        } else if room.game.current_player().user != user_id {
            Err(GameError::NotYourTurn)
        } else {
            room.game.submit_cards(cards)
        }
    }

    pub fn choose_landlord_by_connection_id(&mut self, connection_id: &ConnectionId, choose: bool) -> Result<MessagePacket, Error> {
        let user = self.get_user(connection_id).expect("No user found by connection_id").clone();
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

    pub fn get_user(&self, connection_id: &ConnectionId) -> Option<&UserId> {
        self.network.user_map.get_by_right(connection_id)
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

    pub fn get_room_by_connection_id(&self, connection_id: &ConnectionId) -> Option<&Room> {
        self.get_room_by_user(self.get_user(connection_id)?)
    }

    pub fn get_room_by_connection_id_mut(&mut self, connection_id: &ConnectionId) -> Option<&mut Room> {
        let user = self.get_user(connection_id)?.to_string();
        self.get_room_by_user_mut(&user)
    }

    pub fn send_to_room(&self, net: &NetworkServer, room: &Room, message: S2CMessage) -> Result<(), NetworkError> {
        for user in room.users.iter() {
            self.network.send_to_user(net, &user, message.clone())?;
        }
        Ok(())
    }

    pub fn send_to_room_by_name(&self, net: &NetworkServer, room_name: &str, message: S2CMessage) -> Result<(), NetworkError> {
        let room = self.get_room_by_user(room_name).expect("Cannot get room");
        self.send_to_room(net, room, message)
    }

    pub fn disconnect_by_endpoint(&mut self, conn: &ConnectionId) {
        if let Some(user) = self.network.user_map.get_by_right(conn).map(|x| x.clone()) {
            self.disconnect(&user);
        }
    }
}

impl Lobby for ServerLobby {
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