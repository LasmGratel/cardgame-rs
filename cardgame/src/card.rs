use std::cmp::{Ord, Ordering, PartialEq, PartialOrd};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

#[derive(Eq, Copy, Clone, FromPrimitive, ToPrimitive)]
pub enum Card {
    Unknown,
    Card3,
    Card4,
    Card5,
    Card6,
    Card7,
    Card8,
    Card9,
    Card10,
    CardJ,
    CardQ,
    CardK,
    CardA,
    Card2,
    CardGhost,
    CardKing,
}

impl Ord for Card {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value().cmp(&other.value())
    }
}

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}

impl Card {
    pub fn from_char(card: &char) -> Card {
        match card {
            '2' => Card::Card2,
            '3' => Card::Card3,
            '4' => Card::Card4,
            '5' => Card::Card5,
            '6' => Card::Card6,
            '7' => Card::Card7,
            '8' => Card::Card8,
            '9' => Card::Card9,
            '1' => Card::Card10,
            'J' => Card::CardJ,
            'Q' => Card::CardQ,
            'K' => Card::CardK,
            'A' => Card::CardA,
            '鬼' => Card::CardGhost,
            '王' => Card::CardKing,
            _ => Card::Unknown,
        }
    }

    pub fn to_string(&self) -> &str {
        match self {
            Card::Card2 => "2",
            Card::Card3 => "3",
            Card::Card4 => "4",
            Card::Card5 => "5",
            Card::Card6 => "6",
            Card::Card7 => "7",
            Card::Card8 => "8",
            Card::Card9 => "9",
            Card::Card10 => "10",
            Card::CardJ => "J",
            Card::CardQ => "Q",
            Card::CardK => "K",
            Card::CardA => "A",
            Card::CardGhost => "鬼",
            Card::CardKing => "王",
            _ => "",
        }
    }

    pub fn value(&self) -> u32 {
        self.to_u32().unwrap_or(0)
    }

    pub fn from_value(i: u32) -> Card {
        return Card::from_u32(i).unwrap_or(Card::Unknown);
    }
}

#[derive(Eq, Clone)]
pub struct CardGroup {
    pub card: Card,
    pub count: u32,
}

impl Ord for CardGroup {
    fn cmp(&self, other: &Self) -> Ordering {
        self.card.cmp(&other.card)
    }
}

impl PartialOrd for CardGroup {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for CardGroup {
    fn eq(&self, other: &Self) -> bool {
        self.card == other.card
    }
}

impl CardGroup {
    pub fn to_cards(self) -> Vec<Card> {
        vec![self.card; self.count as usize]
    }
}

impl std::ops::Sub<CardGroups> for CardGroups {
    type Output = Option<CardGroups>;
    fn sub(self, rhs: CardGroups) -> Option<CardGroups> {
        if self.type_len() < rhs.type_len() {
            return None;
        }
        let mut groups = CardGroups {
            groups: self.groups.clone(),
        };
        for c in groups.groups.iter_mut() {
            let group = rhs.find_group_by_card(c.card);
            if group.is_some() {
                let group = group.unwrap();
                if c.count < group.count {
                    return None;
                }
                c.count -= group.count;
            }
        }
        groups.groups.retain(|x| x.count > 0);
        groups.groups.sort();
        Some(groups)
    }
}

pub struct CardGroups {
    pub groups: Vec<CardGroup>,
}

impl CardGroups {
    pub fn type_len(&self) -> usize {
        self.groups.len()
    }

    pub fn card_len(&self) -> usize {
        let mut size = 0u32;
        for x in self.groups.iter() {
            size += x.count;
        }
        size as usize
    }

    pub fn find_group_by_count(&self, count: u32) -> Option<&CardGroup> {
        self.groups.iter().find(|x| x.count == count)
    }

    pub fn find_group_by_card(&self, card: Card) -> Option<&CardGroup> {
        self.groups.iter().find(|x| x.card == card)
    }

    pub fn has_group(&self, count: u32) -> bool {
        self.groups.iter().any(|x| x.count == count)
    }

    pub fn only_has_group(&self, count: u32) -> bool {
        self.groups.len() == 1 && self.has_group(count)
    }

    pub fn to_cards(self) -> Vec<Card> {
        self.groups.into_iter().flat_map(|x| x.to_cards()).collect()
    }
}

pub fn to_card_groups(vec: &Vec<Card>) -> CardGroups {
    let mut arr = [0u32; 16];
    for c in vec {
        arr[c.value() as usize] += 1;
    }
    let mut groups = CardGroups { groups: vec![] };
    for i in 0..16 {
        if arr[i as usize] > 0 {
            groups.groups.push(CardGroup {
                card: Card::from_value(i),
                count: arr[i as usize],
            })
        }
    }
    groups.groups.sort();
    return groups;
}
