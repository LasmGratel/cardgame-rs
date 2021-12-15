use bevy_spicy_networking::{ConnectionId, NetworkError, NetworkServer};
use bimap::{BiMap, BiHashMap};
use cardgame::user::UserId;
use cardgame_common::message::S2CMessage;

pub type MessagePacket = (MessageTarget, S2CMessage);

pub enum MessageTarget {
    All,
    Reply,
    ConnectionId(ConnectionId),
    User(UserId),
    Room(String)
}

pub fn send_to_endpoint(net: &NetworkServer, message: S2CMessage, endpoint: ConnectionId) -> Result<(), NetworkError> {
    net.send_message(endpoint, message)
}

#[derive(Clone, Default)]
pub struct NetworkManager {
    pub user_map: BiMap<UserId, ConnectionId>,
}

impl NetworkManager {
    pub fn connect(&mut self, user_id: UserId, endpoint: ConnectionId) {
        self.user_map.insert(user_id, endpoint);
    }

    pub fn send_to_user(&self, net: &NetworkServer, user_id: &str, message: S2CMessage) -> Result<(), NetworkError> {
        let endpoint = self.user_map.get_by_left(user_id).expect("No such client found").clone();
        net.send_message(endpoint, message)
    }

    /// Send a message to all clients registered
    ///
    /// # Arguments
    ///
    /// * `message`: message to send
    ///
    /// returns: Result<usize, usize> amount of clients, error index
    ///
    pub fn send_to_all(&self, net: &NetworkServer, message: S2CMessage) -> Result<(), NetworkError> {
        for endpoint in self.user_map.right_values() {
            net.send_message(endpoint.clone(), message.clone())?;
        }
        Ok(())
    }
}