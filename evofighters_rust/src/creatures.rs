use std::fmt;
use std::cmp::{max, min};

use dna;
use eval;
use parsing;
use settings;
use arena;
use saver::{GlobalStatistics, RngState};

static FEEDER_ID: usize = 0;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Liveness {
    Alive,
    Dead,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Creature {
    dna: dna::DNA,
    inv: Vec<dna::lex::Item>,
    energy: usize,
    pub generation: usize,
    pub num_children: usize,
    pub signal: Option<dna::lex::Signal>,
    pub survived: usize,
    pub kills: usize,
    pub instr_used: usize,
    pub instr_skipped: usize,
    pub last_action: eval::PerformableAction,
    pub id: usize,
    pub eaten: usize,
    pub parents: (usize, usize),
}

impl fmt::Display for Creature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_feeder() {
            write!(f, "[Feeder]")
        } else {
            write!(f, "[Creature {}]", self.id)
        }
    }
}

impl Creature {
    pub(in creatures) fn new(
        id: usize,
        dna: dna::DNA,
        generation: usize,
        parents: (usize, usize),
    ) -> Creature {
        Creature {
            dna: dna,
            inv: Vec::with_capacity(settings::MAX_INV_SIZE),
            energy: settings::DEFAULT_ENERGY,
            generation: generation,
            num_children: 0,
            signal: None,
            survived: 0,
            kills: 0,
            instr_used: 0,
            instr_skipped: 0,
            last_action: eval::PerformableAction::NoAction,
            id: id,
            eaten: 0,
            parents: parents,
        }
    }

    pub fn seed_creature(id: usize) -> Creature {
        Creature {
            dna: dna::DNA::seed(),
            inv: Vec::with_capacity(settings::MAX_INV_SIZE),
            energy: settings::DEFAULT_ENERGY,
            generation: 0,
            num_children: 0,
            signal: None,
            survived: 0,
            kills: 0,
            instr_used: 0,
            instr_skipped: 0,
            last_action: eval::PerformableAction::NoAction,
            id: id,
            eaten: 0,
            parents: (0, 0),
        }
    }

    pub fn iter(&self) -> parsing::Parser {
        if self.is_feeder() {
            parsing::Parser::feeder_new()
        } else {
            parsing::Parser::new(&self.dna, self.instr_used + self.instr_skipped)
        }
    }

    pub fn feeder() -> Creature {
        Creature {
            id: FEEDER_ID,
            dna: dna::DNA::feeder(),
            inv: vec![dna::lex::Item::Food],
            energy: 1,
            generation: 0,
            num_children: 0,
            signal: Some(dna::lex::Signal::Green),
            kills: 0,
            survived: 0,
            instr_used: 0,
            instr_skipped: 0,
            last_action: eval::PerformableAction::NoAction,
            eaten: 0,
            parents: (0, 0),
        }
    }

    pub fn is_feeder(&self) -> bool {
        self.id == FEEDER_ID
    }

    fn has_items(&self) -> bool {
        !self.inv.is_empty()
    }

    pub fn add_item(&mut self, item: dna::lex::Item) {
        if self.inv.len() < settings::MAX_INV_SIZE {
            self.inv.push(item)
        } else {
            debug!("{} tries to add {:?} but has no more space", self, item);
        }
    }

    pub fn survived_encounter(&mut self) {
        self.survived += 1;
        self.last_action = eval::PerformableAction::NoAction;
    }

    fn set_signal(&mut self, signal: dna::lex::Signal) {
        self.signal = Some(signal)
    }

    fn pop_item(&mut self) -> Option<dna::lex::Item> {
        self.inv.pop()
    }

    pub fn top_item(&self) -> Option<dna::lex::Item> {
        if !self.inv.is_empty() {
            Some(self.inv[self.inv.len() - 1])
        } else {
            None
        }
    }

    fn eat(&mut self, item: dna::lex::Item) {
        let energy_gain = 3 * item as usize;
        debug!("{} gains {} life from {:?}", self, energy_gain, item);
        self.gain_energy(energy_gain)
    }

    pub(in creatures) fn valid(&self) -> bool {
        // TODO: do compilation here, check if the creature has an
        // infinite cycle or parse stack too deep up front instead of
        // letting it live.
        self.dna.valid()
    }

    pub fn liveness(&self) -> Liveness {
        if self.energy > 0 && (!self.is_feeder() || self.has_items()) {
            Liveness::Alive
        } else {
            Liveness::Dead
        }
    }

    pub fn dead(&self) -> bool {
        self.liveness() == Liveness::Dead
    }

    pub fn steal_from(&mut self, other: &mut Creature) {
        if let Some(item) = other.pop_item() {
            self.add_item(item)
        }
    }

    pub fn energy(&self) -> usize {
        self.energy
    }

    pub fn lose_energy(&mut self, amount: usize) {
        self.energy = self.energy.saturating_sub(amount)
    }

    pub fn gain_energy(&mut self, amount: usize) {
        self.energy += amount;
        self.energy = min(settings::DEFAULT_ENERGY, self.energy);
    }

    pub fn kill(&mut self) {
        self.energy = 0;
    }

    pub fn update_from_thought(&mut self, thought: &parsing::Thought) {
        self.instr_used += parsing::skipped(thought);
        self.instr_skipped += parsing::skipped(thought);
        if thought.is_err() {
            self.kill()
        }
    }

    pub fn pay_for_mating(&mut self, share: usize) -> bool {
        let mut cost = (settings::MATING_COST as f64 * (share as f64 / 100.0)).round() as isize;
        while cost > 0 {
            match self.pop_item() {
                Some(item) => {
                    cost -= (item as isize) * 2;
                }
                None => {
                    info!("{} ran out of items and failed to mate", self);
                    return false;
                }
            }
        }
        true
    }

    pub fn carryout(
        &mut self,
        other: &mut Creature,
        action: eval::PerformableAction,
        rng: &mut RngState,
    ) -> arena::FightStatus {
        if self.is_feeder() {
            debug!("Feeder does nothing");
            return arena::FightStatus::Continue;
        }
        if self.dead() {
            return arena::FightStatus::End;
        }
        match action {
            eval::PerformableAction::Signal(sig) => self.set_signal(sig),
            eval::PerformableAction::Eat => match self.pop_item() {
                Some(item) => {
                    info!("{} eats {:?}", self, self.top_item());
                    self.eat(item);
                }
                None => debug!(
                    "{} tries to eat an item, but \
                     doesn't have one",
                    self
                ),
            },
            eval::PerformableAction::Take => match other.pop_item() {
                Some(item) => {
                    info!("{} takes {:?} from {}", self, item, other.id);
                    self.add_item(item);
                }
                None => {
                    debug!(
                        "{} tries to take an item from {}, \
                         but there's nothing to take.",
                        self, other.id
                    );
                }
            },
            eval::PerformableAction::Wait => debug!("{} waits", self),
            // This is only defending with no corresponding attack
            eval::PerformableAction::Defend(dmg) => {
                debug!("{} defends with {:?} fruitlessly", self, dmg)
            }
            eval::PerformableAction::Flee => {
                let my_roll: f64 = rng.rand_range(0.0, self.energy as f64);
                let other_roll: f64 = rng.rand_range(0.0, other.energy as f64);
                let dmg: usize = rng.rand_range(0, 4);
                if other_roll < my_roll {
                    info!(
                        "{} flees the encounter and takes \
                         {} damage",
                        self, dmg
                    );
                    self.lose_energy(dmg);
                    return arena::FightStatus::End;
                } else {
                    debug!("{} tries to flee, but {} prevents it", self, other);
                }
            }
            invalid_action => panic!("Shouldn't have gotten {:?} here", invalid_action),
        }
        arena::FightStatus::Continue
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Creatures {
    creatures: Vec<Creature>,
    last_creature_id: usize,
    max_population_size: usize,
    feeder_count: usize,
    #[serde(skip)]
    rng: RngState,
}

impl Creatures {
    pub fn new(max_population_size: usize) -> Creatures {
        Creatures {
            creatures: ((FEEDER_ID + 1)..max_population_size + 1)
                .map(Creature::seed_creature)
                .collect(),
            last_creature_id: FEEDER_ID,
            max_population_size,
            feeder_count: 0,
            rng: RngState::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.creatures.len()
    }

    pub fn is_empty(&self) -> bool {
        self.creatures.is_empty()
    }

    pub fn spawn_feeders(&mut self) {
        if self.len() + self.feeder_count < self.max_population_size {
            self.feeder_count = self.max_population_size - (self.feeder_count + self.len());
        }
    }

    pub fn feeder_count(&self) -> usize {
        self.feeder_count
    }

    pub fn random_creature(&mut self) -> Creature {
        let index = self.rng.rand_range(0, self.creatures.len());
        self.creatures.swap_remove(index)
    }

    pub fn random_creature_or_feeder(&mut self) -> Creature {
        let index = self.rng
            .rand_range(0, self.creatures.len() + self.feeder_count);
        if index < self.creatures.len() {
            self.creatures.swap_remove(index)
        } else {
            self.feeder_count -= 1;
            Creature::feeder()
        }
    }

    pub fn absorb(&mut self, creature: Creature) {
        if creature.dead() {
            ()
        } else if creature.is_feeder() {
            self.feeder_count += 1;
        } else {
            self.creatures.push(creature);
        }
    }

    pub fn mate(
        &mut self,
        p1: &mut Creature,
        p2: &mut Creature,
    ) -> (Option<Creature>, GlobalStatistics) {
        let (child_dna, stats) = dna::DNA::combine(&mut p1.dna, &mut p2.dna, &mut self.rng);
        let child = Creature::new(
            self.next_creature_id(),               // id
            child_dna,                             // dna
            max(p1.generation, p2.generation) + 1, // generation
            (p1.id, p2.id),                        // parents
        );
        p1.num_children += 1;
        p2.num_children += 1;
        info!("{} and {} have a child named {}", p1, p2, child);
        if !child.valid() {
            info!("But it didn't live since its dna was messed up.");
            (None, stats)
        } else {
            (Some(child), stats)
        }
    }

    fn next_creature_id(&mut self) -> usize {
        self.last_creature_id += 1;
        self.last_creature_id
    }
}
