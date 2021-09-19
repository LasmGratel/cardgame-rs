use crate::card::*;

/// 出牌规则
pub trait Rule {
    /// 同规则下是否可被顶替
    fn matches(&self, cards: &[Card]) -> bool;

    /// 规则名称
    fn to_string(&self) -> &str;

    /// 是否为空规则
    fn is_none(&self) -> bool {
        false
    }

    /// 是否为炸弹
    ///
    /// 0 为普通规则，1 为炸弹，2 为火箭
    fn bomb_priority(&self) -> u32 {
        0
    }
}

/// 单
pub struct RuleOne {
    card: Card,
}

/// 对
pub struct RuleTwo {
    card: Card,
}

/// 三带一
pub struct RuleThreeWithOne {
    first: Card,
    second: Card,
}

/// 三带二
pub struct RuleThreeWithTwo {
    first: Card,
    second: Card,
}

/// 四带二
pub struct RuleFourWithTwo {
    first: Card,
    second: Card,
}

/// 炸弹
pub struct RuleBomb {
    card: Card,
}

/// 火箭
pub struct RuleRocket;

/// 顺子
pub struct RuleChain {
    first: Card,
    count: u32,
}

/// 单翼飞机
pub struct RuleAirplaneWithOneWing {
    first: Card,
    count: u32,
}

/// 双翼飞机
pub struct RuleAirplaneWithTwoWings {
    first: Card,
    count: u32,
}

/// 可被任何规则覆盖的空规则
pub struct RuleNone;

impl Rule for RuleNone {
    fn matches(&self, _cards: &[Card]) -> bool {
        false
    }
    fn to_string(&self) -> &str {
        "无"
    }
    fn is_none(&self) -> bool {
        true
    }
}

impl RuleOne {
    fn try_new(cards: &[Card]) -> Option<RuleOne> {
        if cards.len() == 1 {
            Some(RuleOne { card: cards[0] })
        } else {
            None
        }
    }
}

impl Rule for RuleOne {
    fn matches(&self, cards: &[Card]) -> bool {
        cards.len() == 1 && cards[0] > self.card
    }
    fn to_string(&self) -> &str {
        "单牌"
    }
}

impl RuleTwo {
    fn try_new(cards: &[Card]) -> Option<RuleTwo> {
        let groups = to_card_groups(cards);
        if groups.only_has_group(2) {
            Some(RuleTwo { card: cards[0] })
        } else {
            None
        }
    }
}

impl Rule for RuleTwo {
    fn matches(&self, cards: &[Card]) -> bool {
        let groups = to_card_groups(cards);
        groups.only_has_group(2) && cards[0] > self.card
    }
    fn to_string(&self) -> &str {
        "对子"
    }
}
impl RuleThreeWithOne {
    fn try_new(cards: &[Card]) -> Option<RuleThreeWithOne> {
        if cards.len() != 4 {
            return None;
        }

        let groups = to_card_groups(cards);
        let first = groups.find_group_by_count(3);
        let second = groups.find_group_by_count(1);

        Some(RuleThreeWithOne {
            first: first?.card,
            second: second?.card,
        })
    }
}
impl Rule for RuleThreeWithOne {
    fn matches(&self, cards: &[Card]) -> bool {
        let rule = RuleThreeWithOne::try_new(cards);
        rule.is_some() && rule.unwrap().first > self.first
    }
    fn to_string(&self) -> &str {
        "三带一"
    }
}
impl RuleThreeWithTwo {
    fn try_new(cards: &[Card]) -> Option<RuleThreeWithTwo> {
        if cards.len() != 5 {
            return None;
        }

        let groups = to_card_groups(cards);
        let first = groups.find_group_by_count(3);
        let second = groups.find_group_by_count(2);

        Some(RuleThreeWithTwo {
            first: first?.card,
            second: second?.card,
        })
    }
}
impl Rule for RuleThreeWithTwo {
    fn matches(&self, cards: &[Card]) -> bool {
        let rule = RuleThreeWithTwo::try_new(cards);
        return rule.is_some() && rule.unwrap().first > self.first;
    }
    fn to_string(&self) -> &str {
        "三带二"
    }
}
impl RuleFourWithTwo {
    fn try_new(cards: &[Card]) -> Option<RuleFourWithTwo> {
        if cards.len() != 6 {
            return None;
        }

        let groups = to_card_groups(cards);
        let first = groups.find_group_by_count(4)?;

        let second = groups.find_group_by_count(2)?;

        Some(RuleFourWithTwo {
            first: first.card,
            second: second.card,
        })
    }
}
impl Rule for RuleFourWithTwo {
    fn matches(&self, cards: &[Card]) -> bool {
        let rule = RuleFourWithTwo::try_new(cards);
        rule.is_some() && rule.unwrap().first > self.first
    }
    fn to_string(&self) -> &str {
        "四带二"
    }
}
impl RuleBomb {
    fn try_new(cards: &[Card]) -> Option<RuleBomb> {
        if cards.len() != 4 {
            return None;
        }
        let groups = to_card_groups(cards);
        if !groups.only_has_group(4) {
            return None;
        }

        Some(RuleBomb { card: cards[0] })
    }
}
impl Rule for RuleBomb {
    fn matches(&self, cards: &[Card]) -> bool {
        let rule = RuleBomb::try_new(cards);
        rule.is_some() && rule.unwrap().card > self.card
    }
    fn to_string(&self) -> &str {
        "炸弹"
    }
    fn bomb_priority(&self) -> u32 {
        1
    }
}
impl RuleRocket {
    fn try_new(cards: &[Card]) -> Option<RuleRocket> {
        if cards.len() != 2 {
            return None;
        }
        let mut iter = cards.iter();
        if iter.all(|x| *x == Card::CardGhost || *x == Card::CardKing) {
            Some(RuleRocket)
        } else {
            None
        }
    }
}
impl Rule for RuleRocket {
    fn matches(&self, _cards: &[Card]) -> bool {
        false // TODO 双副牌
    }
    fn to_string(&self) -> &str {
        "火箭"
    }
    fn bomb_priority(&self) -> u32 {
        2
    }
}
impl RuleChain {
    fn try_new(cards: &[Card]) -> Option<RuleChain> {
        let groups = to_card_groups(cards);
        let first = groups.groups.first()?;
        let count = first.count;
        let mut card = first.card;

        for group in groups.groups.iter() {
            if group.card.value() - card.value() > 1 || group.count != count {
                return None;
            }
            card = group.card;
        }

        let type_len = groups.type_len() as u32;

        match count {
            1 => {
                if type_len < 5 {
                    return None;
                }
            }
            2 => {
                if type_len < 3 {
                    return None;
                }
            }
            3 => {
                if type_len < 2 {
                    return None;
                }
            }
            _ => return None,
        }
        Some(RuleChain {
            first: first.card,
            count: type_len,
        })
    }
}
impl Rule for RuleChain {
    fn matches(&self, cards: &[Card]) -> bool {
        let rule = RuleChain::try_new(cards);
        if rule.is_none() {
            return false;
        }
        let rule = rule.unwrap();
        rule.first > self.first && rule.count == self.count
    }
    fn to_string(&self) -> &str {
        "顺子"
    }
}
impl RuleAirplaneWithOneWing {
    fn try_new(cards: &[Card]) -> Option<RuleAirplaneWithOneWing> {
        let groups = to_card_groups(cards);
        if groups.type_len() < 4 {
            return None;
        }
        let iter = groups.groups.iter();

        let mut threes_count = 0;
        let mut ones_count = 0;
        let mut first_card: Option<Card> = None;
        let mut card: Option<Card> = None;

        for group in iter {
            if group.count == 3 {
                if card.is_none() {
                    card = Some(group.card);
                    threes_count += 1;
                    first_card = Some(group.card);
                } else if group.card.value() - card.unwrap().value() != 1 {
                    return None;
                } else {
                    card = Some(group.card);
                    threes_count += 1;
                }
            } else {
                ones_count += group.count;
            }
        }

        if first_card.is_none() || ones_count < 2 || ones_count != threes_count {
            return None;
        }

        Some(RuleAirplaneWithOneWing {
            first: first_card.unwrap(),
            count: threes_count,
        })
    }
}
impl Rule for RuleAirplaneWithOneWing {
    fn matches(&self, cards: &[Card]) -> bool {
        let rule = RuleAirplaneWithOneWing::try_new(cards);
        if rule.is_none() {
            return false;
        }
        let rule = rule.unwrap();
        rule.first > self.first && rule.count == self.count
    }
    fn to_string(&self) -> &str {
        "单翼飞机"
    }
}
impl RuleAirplaneWithTwoWings {
    fn try_new(cards: &[Card]) -> Option<RuleAirplaneWithTwoWings> {
        let groups = to_card_groups(cards);
        if groups.type_len() < 4 {
            return None;
        }
        let iter = groups.groups.iter();

        let mut threes_count = 0;
        let mut twos_count = 0;
        let mut first_card: Option<Card> = None;
        let mut card: Option<Card> = None;

        for group in iter {
            if group.count == 3 {
                if card.is_none() {
                    card = Some(group.card);
                    threes_count += 1;
                    first_card = Some(group.card);
                } else if group.card.value() - card.unwrap().value() != 1 {
                    return None;
                } else {
                    card = Some(group.card);
                    threes_count += 1;
                }
            } else if group.count == 2 {
                twos_count += 1;
            } else {
                return None;
            }
        }

        if first_card.is_none() || twos_count < 2 || twos_count != threes_count {
            return None;
        }

        Some(RuleAirplaneWithTwoWings {
            first: first_card.unwrap(),
            count: threes_count,
        })
    }
}
impl Rule for RuleAirplaneWithTwoWings {
    fn matches(&self, cards: &[Card]) -> bool {
        let rule = RuleAirplaneWithTwoWings::try_new(cards);
        if rule.is_none() {
            return false;
        }
        let rule = rule.unwrap();
        rule.first > self.first && rule.count == self.count
    }
    fn to_string(&self) -> &str {
        "双翼飞机"
    }
}

pub fn match_rule(cards: &[Card]) -> Box<dyn Rule> {
    match cards.len() {
        1 => {
            let option = RuleOne::try_new(cards);
            if option.is_some() {
                return Box::new(option.unwrap());
            }
        }
        2 => {
            let option = RuleTwo::try_new(cards);
            if option.is_some() {
                return Box::new(option.unwrap());
            }

            let option = RuleRocket::try_new(cards);
            if option.is_some() {
                return Box::new(option.unwrap());
            }
        }
        4 => {
            let option = RuleThreeWithOne::try_new(cards);
            if option.is_some() {
                return Box::new(option.unwrap());
            }
            let option = RuleBomb::try_new(cards);
            if option.is_some() {
                return Box::new(option.unwrap());
            }
        }
        _ => {
            let option = RuleThreeWithTwo::try_new(cards);
            if option.is_some() {
                return Box::new(option.unwrap());
            }

            let option = RuleFourWithTwo::try_new(cards);
            if option.is_some() {
                return Box::new(option.unwrap());
            }

            let option = RuleChain::try_new(cards);
            if option.is_some() {
                return Box::new(option.unwrap());
            }

            let option = RuleAirplaneWithOneWing::try_new(cards);
            if option.is_some() {
                return Box::new(option.unwrap());
            }

            let option = RuleAirplaneWithTwoWings::try_new(cards);
            if option.is_some() {
                return Box::new(option.unwrap());
            }
        }
    }

    Box::new(RuleNone)
}

/// Returns if the submitted cards match the current rule.
///
/// # Arguments
///
/// * `rule`: Rule to be matched
/// * `to_match`: Cards to match a rule
///
/// returns: bool
///
/// # Examples
///
/// ```
/// use cardgame::{rule_matches, Card, match_rule};
/// let last_cards = vec![Card::Card3];
/// let submitted_cards = vec![Card::Card4];
/// assert!(rule_matches(&*match_rule(&last_cards), &submitted_cards));
/// ```
pub fn rule_matches(rule: &dyn Rule, to_match: &[Card]) -> bool {
    let to_rule = match_rule(to_match);
    if (rule.is_none() && !to_rule.is_none()) || to_rule.bomb_priority() > rule.bomb_priority() {
        true
    } else {
        rule.matches(to_match)
    }
}

/*
impl Rule<Rule> for Rule {
    fn matches(&self, cards: &[Card]) -> bool {
        let groups = to_card_groups(cards);
    }
    fn try_new(cards: &[Card]) -> Option<Rule> {
        let groups = to_card_groups(cards);
    }
}
*/
