use std::cmp::{ Ord, PartialEq, PartialOrd, Ordering };

#[derive(Eq, Copy, Clone)]
pub enum Card {
    Unknown, Card3, Card4, Card5, Card6, Card7, Card8, Card9, Card10, CardJ, CardQ, CardK, CardA, Card2, CardGhost, CardKing
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
    pub fn from_char(card: char) -> Card {
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
            _ => Card::Unknown
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
            _ => ""
        }
    }

    pub fn value(&self) -> u32 {
        *self as u32
    }

    pub fn from_value(i: u32) -> Card {
        match i {
            1 => Card::Card3,
            2 => Card::Card4,
            3 => Card::Card5,
            4 => Card::Card6,
            5 => Card::Card7,
            6 => Card::Card8,
            7 => Card::Card9,
            8 => Card::Card10,
            9 => Card::CardJ,
            10 => Card::CardQ,
            11 => Card::CardK,
            12 => Card::CardA,
            13 => Card::Card2,
            14 => Card::CardGhost,
            15 => Card::CardKing,
            _ => Card::Unknown
        }
    }
}

#[derive(Eq)]
pub struct CardGroup {
    pub card: Card,
    pub count: u32
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

pub struct CardGroups {
    pub groups: Vec<CardGroup>
}

pub fn to_card_groups(vec: &Vec<Card>) -> CardGroups {
    let mut arr = [0u32; 16];
    for c in vec {
        arr[c.value() as usize] += 1;
    }
    let mut groups = CardGroups { groups: vec![] };
    for i in 0..16 {
        if arr[i as usize] > 0 {
            groups.groups.push(CardGroup { card: Card::from_value(i), count: arr[i as usize] })
        }
    }
    groups.groups.sort();
    return groups
}