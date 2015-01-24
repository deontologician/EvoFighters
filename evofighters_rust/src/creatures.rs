use std::fmt;
use std::rand;
use std::num::Int;

use dna;
use eval;
use parsing;
use settings;
use arena;

static FEEDER_ID: usize = 0;

#[derive(Show)]
pub struct Creature {
    dna: dna::DNA,
    inv: Vec<dna::Item>,
    pub energy: usize,
    pub generation: usize,
    pub num_children: usize,
    pub signal: Option<dna::Signal>,
    pub survived: usize,
    pub kills: usize,
    pub instr_used: usize,
    pub instr_skipped: usize,
    pub last_action: eval::PerformableAction,
    pub id: usize,
    pub eaten: usize,
    pub parents: (usize, usize),
    // These are used for the iterator
    parser: parsing::Parser,
}


impl fmt::String for Creature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_feeder() {
            write!(f, "[|Feeder|]")
        } else {
            write!(f, "<]Creature {}[>", self.id)
        }
    }
}

impl Creature {

    pub fn new(id: usize,
               dna: dna::DNA,
               parents: (usize, usize)) -> Creature {
        Creature {
            dna: dna.clone(),
            inv: Vec::with_capacity(settings::MAX_INV_SIZE),
            energy: settings::DEFAULT_ENERGY,
            generation: 0,
            num_children: 0,
            signal: None,
            survived: 0,
            kills: 0,
            instr_used: 0,
            instr_skipped: 0,
            last_action: eval::PerformableAction::Wait,
            id: id,
            eaten: 0,
            parents: parents,
            parser: parsing::Parser::new(dna.clone()),
        }
    }

    pub fn feeder() -> Creature {
        let dntel: dna::DNA = dna::empty_dna();
        Creature {
            id: FEEDER_ID,
            dna: dntel.clone(),
            inv: Vec::new(),
            energy: 1,
            generation: 0,
            num_children: 0,
            signal: Some(dna::Signal::Green),
            kills: 0,
            survived: 0,
            instr_used: 0,
            instr_skipped: 0,
            last_action: eval::PerformableAction::Wait,
            eaten: 0,
            parents: (0, 0),
            parser: parsing::Parser::new(dntel.clone()),
        }
    }

    pub fn is_feeder(&self) -> bool {
        self.id == FEEDER_ID
    }

    pub fn add_item(&mut self, item: dna::Item) {
        if self.inv.len() < settings::MAX_INV_SIZE {
            self.inv.push(item)
        } else {
            print2!("{} tries to add {:?} but has no more space",
                    self, item);
        }
    }

    pub fn set_signal(&mut self, signal:dna::Signal) {
        self.signal = Some(signal)
    }

    pub fn pop_item(&mut self) -> Option<dna::Item> {
        self.inv.pop()
    }

    pub fn top_item(&self) -> Option<dna::Item> {
        if !self.inv.is_empty() {
            Some(self.inv[self.inv.len() - 1])
        } else {
            None
        }
    }

    pub fn eat(&mut self, item: dna::Item) {
        let energy_gain = 3 * item as usize;
        print2!("{} gains {} life from {:?}", self, energy_gain, item);
        self.energy += energy_gain
    }

    pub fn dead(&self) -> bool {
        if self.is_feeder() {
            self.energy == 0 || self.inv.is_empty()
        } else {
            self.energy == 0 || self.dna.is_empty()
        }
    }

    pub fn alive(&self) -> bool {
        !self.dead()
    }

    pub fn lose_life(&mut self, amount: usize) {
        self.energy = self.energy.saturating_sub(amount)
    }

    pub fn carryout(&mut self,
                    other: &mut Creature,
                    action: eval::PerformableAction,
                    rng: &mut rand::Rng) -> arena::FightStatus {
        if self.is_feeder() {
            print2!("Feeder does nothing");
            return arena::FightStatus::Continue;
        }
        if self.dead() {
            return arena::FightStatus::End
        }
        match action {
            eval::PerformableAction::Signal(sig) => {
                self.set_signal(sig)
            },
            eval::PerformableAction::Eat => {
                match self.pop_item() {
                    Some(item) => {
                        print1!("{} eats {:?}",
                                self, self.top_item());
                        self.eat(item);
                    },
                    None =>
                        print2!("{} tries to eat an item, but \
                                doesn't have one", self)
                }
            },
            eval::PerformableAction::Take => {
                match other.pop_item() {
                    Some(item) => {
                        print1!("{} takes {:?} from {}",
                                self, item, other.id);
                        self.add_item(item);
                    },
                    None => {
                        print2!("{} tries to take an item from {}, \
                                but there's nothing to take.",
                                self, other.id);
                    }
                }
            },
            eval::PerformableAction::Wait => print2!(
                "{} waits", self),
            // This is only defending with no corresponding attack
            eval::PerformableAction::Defend(dmg) =>
                print2!("{} defends with {:?} fruitlessly",
                        self, dmg),
            eval::PerformableAction::Flee => {
                let my_roll: f64 = rng.gen_range(0.0, self.energy as f64);
                let other_roll: f64 = rng.gen_range(0.0, other.energy as f64);
                let dmg: usize = rng.gen_range(0, 3);
                if other_roll < my_roll {
                    print1!("{} flees the encounter and takes \
                            {} damage", self, dmg);
                    self.lose_life(dmg);
                    return arena::FightStatus::End
                } else {
                    print2!("{} tries to flee, but {} prevents it",
                            self, other);
                }

            },
            invalid_action => panic!("Shouldn't have gotten {:?} here",
                                     invalid_action)
        }
        arena::FightStatus::Continue
    }
}

impl Iterator for Creature {
    type Item = (Box<dna::ConditionTree>, usize);
    fn next(&mut self) -> Option<(Box<dna::ConditionTree>, usize)> {
        let thought = self.parser.next().expect("parser ended somehow!");
        match thought {
            parsing::Thought::Decision{
                tree,
                icount,
                skipped,
            } => {
                print3!("{}'s thought process: \n{:?}",
                        self, tree);
                print3!("which required {} instructions and {} \
                        instructions skipped over", icount, skipped);
                self.instr_used += icount;
                self.instr_skipped += skipped;
                Some((tree, icount + skipped))
            },
            parsing::Thought::Indecision{
                ref reason,
                ref icount,
                ref skipped,
            } => {
                print1!("{} was paralyzed by analysis and died: {:?} after \
                        {} instructions and {} skipped",
                        self, reason, icount, skipped);
                self.energy = 0;
                None
            }
        }
    }
}

pub fn gene_primer(dna: dna::DNA) -> Vec<Vec<i8>> {
    let mut result = Vec::new();
    let mut chunk = Vec::new();
    for &base in dna.iter() {
        chunk.push(base);
        if base == -1 {
            result.push(chunk);
            chunk = Vec::new();
        }
    }
    if !chunk.is_empty() {
        result.push(chunk);
    }
    result
}

pub fn try_to_mate(
    mating_chance: usize,
    first_mate: &mut Creature,
    fm_share: usize,
    second_mate: &mut Creature,
    sm_share: usize,
    rng: &mut rand::Rng,
    ) {
    if rng.gen_range(1, 100) > mating_chance
        || first_mate.dead()
        || second_mate.dead() {
            return
        }
    print1!("{} tried to mate with {}!", first_mate, second_mate);
    if first_mate.is_feeder() || second_mate.is_feeder() {
        print1!("{} tried to mate with {}", first_mate, second_mate);
        if first_mate.is_feeder() {
            first_mate.energy = 0;
        }
        if second_mate.is_feeder() {
            second_mate.energy = 0;
        }
        return
    }
    print2!("Attempting to mate");
    fn pay_cost(p, share) -> bool {
        let cost = settings.MATING_COST * (share / 100.0); //wrong
    }
}
