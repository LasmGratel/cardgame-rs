extern crate serde;
extern crate regex;
extern crate num_traits;
extern crate num_derive;
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub enum Signal {
    Greet
}

pub mod card;
pub mod game;
pub mod messages;
pub mod player;
pub mod rule;

pub use card::*;
pub use game::*;
pub use player::*;
pub use rule::*;