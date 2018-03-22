#[derive(Copy, Clone, Serialize, Deserialize, Debug, Default)]
pub struct GlobalStatistics {
    pub children_born: usize,
    pub feeders_eaten: usize,
    pub kills: usize,
    pub rounds: usize,
}

impl GlobalStatistics {
    pub fn new() -> GlobalStatistics {
        GlobalStatistics::default()
    }

    pub fn absorb(&mut self, other: GlobalStatistics) {
        self.children_born += other.children_born;
        self.feeders_eaten += other.feeders_eaten;
        self.kills += other.kills;
        self.rounds += other.rounds;
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreatureStats {
    pub kills: usize,
    pub num_children: usize,
    pub survived: usize,
    pub eaten: usize,
}
