extern crate num_derive;
extern crate num_traits;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate rand;
extern crate anyhow;
#[cfg(test)]
mod tests {
    use crate::{parse_input, match_rule};

    mod rules {
        use crate::{match_rule, parse_input, Rule};

        fn rule_from_str(input: &str) -> Box<dyn Rule> {
            match_rule(&parse_input(input).unwrap())
        }

        /// 测试单牌规则
        #[test]
        fn one() {
            let rule = rule_from_str("4");
            assert!(rule.matches(&parse_input("2").unwrap()));
            assert!(!rule.matches(&parse_input("3").unwrap()));
        }

        #[test]
        fn two() {
            let rule = rule_from_str("44");
            assert!(rule.matches(&parse_input("22").unwrap()));
            assert!(!rule.matches(&parse_input("33").unwrap()));
        }
    }

    /// 测试不符合规则的乱敲
    #[test]
    fn random_inputs() {
        let input = "123";
        let cards = parse_input(input).unwrap();
        let rule = match_rule(&cards);
        assert!(rule.is_none());
    }
}

pub enum Signal {
    Greet, Matchmake
}

pub mod card;
pub mod game;
pub mod lobby;
pub mod messages;
pub mod player;
pub mod rule;
pub mod user;

pub use card::*;
pub use game::*;
pub use lobby::*;
pub use messages::*;
pub use player::*;
pub use rule::*;
pub use rand::thread_rng;
pub use rand::seq::SliceRandom;
