use std::io::Read;
use bevy::prelude::*;
use crate::ClientState;

pub fn login(
    mut state: ResMut<ClientState>
) {
    println!("Username: ");
    let mut user_name = String::default();
    std::io::stdin().read_to_string(&mut user_name).expect("Cannot read user name");
    state.user_name = user_name;

}