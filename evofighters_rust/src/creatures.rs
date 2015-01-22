use std::fmt;
use std::rand;
use std::num::Int;
use std::rc;

use dna;
use eval;
use parsing;
use settings;
use arena;

trait Creature {
    pub fn last_action(&self) -> eval::PerformableAction;
    pub fn add_item(&mut self, item: dna::Item);
    pub fn pop_item(&mut self) -> Option<dna::Item>;

    pub fn top_item(&self) -> Option<dna::Item>;

    pub fn eat(&mut self, item: dna::Item);

    pub fn dead(&self) -> bool;

    pub fn alive(&self) -> bool;

    pub fn lose_life(&mut self, amount: usize);

    pub fn carryout(&mut self,
                    other: &mut Creature,
                    action: eval::PerformableAction,
                    rng: &mut rand::Rng) -> arena::FightStatus;
    
    pub fn new<'a>(id: usize,
                   dna: dna::DNA,
                   parents: (usize, usize)) -> Self;

}

#[derive(Show)]
pub struct RealCreature {
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
    pub is_feeder: bool,
    pub eaten: usize,
    pub parents: (usize, usize),
    // These are used for the iterator
    parser: parsing::Parser,
}

impl fmt::String for RealCreature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<]RealCreature {}[>", self.id)
    }
}

impl RealCreature {
    
    pub fn new(id: usize,
               dna: dna::DNA,
               parents: (usize, usize)) -> RealCreature {
        RealCreature {
            dna: dna,
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
            is_feeder: false,
            eaten: 0,
            parents: parents,
            parser: parsing::Parser::new(dna),
        }
    }

}

impl Creature for RealCreature {
    pub fn add_item(&mut self, item: dna::Item) {
        if self.inv.len() < settings::MAX_INV_SIZE {
            self.inv.push(item)
        } else {
            print2!("{} tries to add {:?} but has no more space",
                    self.id, item);
        }
    }

    pub fn last_action(&self) -> eval::PerformableAction {
        self.last_action
    }

    pub fn pop_item(&mut self) -> Option<dna::Item> {
        self.inv.pop()
    }

    pub fn top_item(&self) -> Option<dna::Item> {
        if !self.inv.is_empty() {
            Some(self.inv[self.inv.len() - 1])
        }else{
            None
        }
    }

    pub fn eat(&mut self, item: dna::Item) {
        let energy_gain = 3 * item as usize;
        print2!("{} gains {} life from {:?}",
                self.id, energy_gain, item);
        self.energy += energy_gain
    }

    pub fn dead(&self) -> bool {
        self.energy == 0 || self.dna.is_empty()
    }

    pub fn alive(&self) -> bool {
        !self.dead()
    }

    pub fn lose_life(&mut self, amount: usize) {
        self.energy = self.energy.saturating_sub(amount)
    }

    pub fn carryout(&mut self,
                    other: &mut RealCreature,
                    action: eval::PerformableAction,
                    rng: &mut rand::Rng) -> arena::FightStatus {
        if self.dead() {
            return arena::FightStatus::End
        }
        match action {
            eval::PerformableAction::Signal(sig) => {
                self.signal = Some(sig);
            },
            eval::PerformableAction::Eat => {
                match self.pop_item() {
                    Some(item) => {
                        print1!("{} eats {:?}", self.id, self.top_item());
                        self.eat(item);
                    },
                    None =>
                        print2!("{} tries to eat an item, but doesn't have \
                                one", self.id)
                }
            },
            eval::PerformableAction::Take => {
                match other.pop_item() {
                    Some(item) => {
                        print1!("{} takes {:?} from {}",
                                self.id, item, other.id);
                        self.add_item(item);
                    },
                    None => {
                        print2!("{} tries to take an item from {}, but \
                                there's nothing to take.", self.id, other.id);
                    }
                }
            },
            eval::PerformableAction::Wait => print2!("{} waits", self.id),
            // This is only defending with no corresponding attack
            eval::PerformableAction::Defend(dmg) =>
                print2!("{} defends with {:?} for no reason", self.id, dmg),
            eval::PerformableAction::Flee => {
                let my_roll: f64 = rng.gen_range(0.0, self.energy as f64);
                let other_roll: f64 = rng.gen_range(0.0, other.energy as f64);
                let dmg: usize = rng.gen_range(0, 3);
                if other_roll < my_roll {
                    print1!("{} flees the encounter and takes {} damage",
                            self.id, dmg);
                    self.lose_life(dmg);
                    return arena::FightStatus::End
                } else {
                    print2!("{} tries to flee, but {} prevents it",
                            self.id, other.id);
                }
                
            },
            invalid_action => panic!("Shouldn't have gotten {:?} here",
                                     invalid_action)
        }
        arena::FightStatus::Continue
    }
}

impl Iterator for RealCreature {
    type Item = (Box<dna::ConditionTree>, usize);
    fn next(&mut self) -> Option<(Box<dna::ConditionTree>, usize)> {
        let thought = self.parser.next().expect("parser ended somehow!");
        match thought {
            parsing::Thought::Decision{
                tree,
                icount,
                skipped,
            } => {
                print3!("{}'s thought process: \n{:?}", self.id, tree);
                print3!("which required {} instructions and {} instructions \
                        skipped over", icount, skipped);
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
                        self.id, reason, icount, skipped);
                self.energy = 0;
                None
            }
        }
    }
}
