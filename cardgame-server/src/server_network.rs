use cardgame::S2CMessage;
use message_io::network::{SendStatus, Endpoint, NetworkController};
use bimap::{BiMap, BiHashMap};
use cardgame::user::UserId;

pub type MessagePacket = (MessageTarget, S2CMessage);

pub enum MessageTarget {
    All,
    Reply,
    Endpoint(Endpoint),
    User(UserId),
    Room(String)
}

pub fn send_to_endpoint(controller: &NetworkController, message: &S2CMessage, endpoint: Endpoint) -> Option<SendStatus> {
    let to_send = bincode::serialize(message).ok()?;
    Some(controller.send(endpoint, &to_send))
}

#[derive(Clone)]
pub struct NetworkManager<'a> {
    pub user_map: BiMap<UserId, Endpoint>,

    controller: &'a NetworkController,
}

impl NetworkManager<'_> {
    pub fn new(controller: &NetworkController) -> NetworkManager {
        NetworkManager {
            user_map: BiHashMap::new(),
            controller
        }
    }

    pub fn connect(&mut self, user_id: UserId, endpoint: Endpoint) {
        self.user_map.insert(user_id, endpoint);
    }

    pub fn send_to_endpoint(&self, message: &S2CMessage, endpoint: Endpoint) -> Option<SendStatus> {
        let to_send = bincode::serialize(message).ok()?;
        Some(self.controller.send(endpoint, &to_send))
    }

    pub fn send_to_user(&self, user_id: &str, message: &S2CMessage) -> Option<SendStatus> {
        let endpoint = self.user_map.get_by_left(user_id).expect("No such client found").clone();
        send_to_endpoint(self.controller, message, endpoint)
    }

    /// Send a message to all clients registered
    ///
    /// # Arguments
    ///
    /// * `message`: message to send
    ///
    /// returns: Result<usize, usize> amount of clients, error index
    ///
    pub fn send_to_all(&self, message: &S2CMessage) -> Result<usize, usize> {
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