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
        use std::time::Instant;
        use crate::{Card, CardStore, gen_cards, match_rule, parse_input, Rule};
        use crate::Card::{CardGhost, CardKing, CardQ};

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

        #[test]
        fn test_gen_card() {
            let mut cards = CardStore::default();
            let start = Instant::now();

            cards.raw = 1 << (52 - 1);
            cards.add(&CardKing);
            cards.add(&CardGhost);
            let duration = start.elapsed();

            println!("Time elapsed in expensive_function() is: {:?}", duration);

            let start = Instant::now();
            let cards = gen_cards();
            let duration = start.elapsed();
            println!("Time elapsed in expensive_function() is: {:?}", duration);
        }

        #[test]
        fn test_card_store() {
            let mut cards = CardStore::default();
            let mut generated = gen_cards();
            generated.sort();
            for card in generated.iter() {
                cards.add(card);
            }
            assert!(cards.get_cards().eq(&generated));
        }

        #[test]
        fn offset() {
            let mut cards = CardStore { raw: 17i64 };
            for i in 0..16 {
                let card = Card::from_value(i);
                cards.add(&card);
                let _count = cards.get_card_count(&card);
            }

            println!("{:#064b}", cards.raw);
            println!("{:?}", cards.get_cards());

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
pub mod error;

pub use card::*;
pub use game::*;
pub use lobby::*;
pub use messages::*;
pub use player::*;
pub use rule::*;
pub use rand::thread_rng;
pub use rand::seq::SliceRandom;
