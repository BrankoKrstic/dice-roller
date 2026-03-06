#[derive(Debug, Clone)]
pub struct DiceRoll {
    pub id: String,
    pub user_id: String,
    pub user_name: String,
    pub ts: String,
    pub expr: String,
    pub result: i64,
    pub breakdown: String,
}

#[derive(Debug, Default, Clone)]
pub struct DiceRollFeed {
    pub rolls: Vec<DiceRoll>,
    pub has_more: bool,
}

impl DiceRollFeed {
    pub fn add_roll(&mut self, roll: DiceRoll) {
        self.rolls.push(roll);
    }
    pub fn new() -> Self {
        Self {
            rolls: vec![],
            has_more: false,
        }
    }
}
