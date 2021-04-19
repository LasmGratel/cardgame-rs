trait Rule {
    pub fn matches(lastCardGroups: CardGroups)
}

enum Rules {
    Single, Double, ThreeWithOne, ThreeWithTwo
}