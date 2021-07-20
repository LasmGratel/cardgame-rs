use cardgame::Lobby;
use cardgame::user::User;

pub trait ServerLobby<'user> {
    fn login(&mut self, user: &'user User);
    fn disconnect(&mut self, user: &'user User);
}

impl<'user> ServerLobby<'user> for Lobby<'user> {
    fn login(&mut self, user: &'user User) {
        self.users.push(user);
    }

    fn disconnect(&mut self, user: &'user User) {
        if let Some(pos) = self.users.iter().position(|x| x.id == user.id) {
            self.users.remove(pos);
        }
    }
}