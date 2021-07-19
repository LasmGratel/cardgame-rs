use cardgame::Lobby;
use cardgame::user::User;

pub trait ServerLobby {
    fn login(&mut self, user_id: String);
    fn disconnect(&mut self, user: &User);
}

impl ServerLobby for Lobby {
    fn login(&mut self, user: &User) {
        self.users.push(user);
    }

    fn disconnect(&mut self, user: &User) {
        if let Some(pos) = self.users.iter().position(|x| x.id == user.id) {
            self.users.remove(pos);
        }
    }
}