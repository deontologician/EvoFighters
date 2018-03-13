use std::fmt;
use std::cmp::{max, min};
use std::rc::Rc;

use dna;
use dna::lex;
use eval;
use parsing;
use parsing::Decision;
use settings;
use arena;
use stats::{GlobalStatistics, CreatureStats};
use rng::RngState;
use simplify::{cycle_detect, ThoughtCycle};

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug, Clone, Copy)]
pub struct CreatureID(u64);

impl CreatureID {
    pub fn feeder() -> CreatureID {
        CreatureID(0)
    }

    pub fn is_feeder(&self) -> bool {
        self.0 == 0
    }

    pub(crate) fn parents_to_u32(
        (CreatureID(p1), CreatureID(p2)): (CreatureID, CreatureID),
    ) -> u32 {
        let p1_prime: u32 = (p1 ^ (p1 >> 16)) as u32;
        let p2_prime: u32 = (p2 ^ (p2 << 16)) as u32;
        p1_prime ^ p2_prime
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct IDGiver {
    next_id_to_give_out: u64,
    modulus: u64,
}

impl IDGiver {
    fn new(start: u64, modulus: u64) -> IDGiver {
        IDGiver {
            next_id_to_give_out: start,
            modulus,
        }
    }

    pub fn unthreaded() -> IDGiver {
        IDGiver::new(1, 1)
    }

    pub fn per_thread(num_threads: usize) -> Vec<IDGiver> {
        assert!(
            num_threads > 0,
            "IDGiver::create must be called with size > 0"
        );
        let nt = num_threads as u64; // avoid a ton of casts
        // next_id_to_give_out can never be zero, since that's the
        // feeder id.
        (1..(nt + 1))
            .map(|i| IDGiver::new(i, nt))
            .collect()
    }

    pub fn into_threads(self, num_threads: usize) -> Vec<IDGiver> {
        let IDGiver {
            next_id_to_give_out,
            modulus,
        } = self;
        let nt = num_threads as u64;

        (0..nt)
            .map(|i| IDGiver::new(i + next_id_to_give_out, nt))
            .collect()
    }

    pub fn next_creature_id(&mut self) -> CreatureID {
        let id = self.next_id_to_give_out;
        self.next_id_to_give_out += self.modulus;
        CreatureID(id)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Creature {
    pub id: CreatureID,
    pub generation: usize,
    pub signal: Option<dna::lex::Signal>,
    pub last_action: eval::PerformableAction,
    pub parents: (CreatureID, CreatureID),
    pub stats: CreatureStats,
    dna: dna::DNA,
    inv: Vec<dna::lex::Item>,
    energy: usize,
    #[serde(skip)]
    thought_cycle: ThoughtCycle,
}

impl fmt::Display for Creature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_feeder() {
            write!(f, "[Feeder]")
        } else {
            write!(f, "[Creature {}]", self.id.0)
        }
    }
}

impl Creature {
    fn new(
        id: CreatureID,
        dna: dna::DNA,
        generation: usize,
        parents: (CreatureID, CreatureID),
    ) -> Result<Creature, parsing::Failure> {
        let thought_cycle = cycle_detect(&dna)?;
        Ok(Creature {
            dna: dna,
            inv: Vec::with_capacity(settings::MAX_INV_SIZE),
            energy: settings::DEFAULT_ENERGY,
            thought_cycle,
            generation: generation,
            signal: None,
            last_action: eval::PerformableAction::NoAction,
            id: id,
            parents: parents,
            stats: CreatureStats::default(),
        })
    }

    pub fn seed_creature(id: CreatureID) -> Creature {
        let dna = dna::DNA::seed();
        // We know the seed dna is valid, so unwrapping
        let thought_cycle = cycle_detect(&dna).unwrap();
        Creature {
            inv: Vec::with_capacity(settings::MAX_INV_SIZE),
            energy: settings::DEFAULT_ENERGY,
            thought_cycle,
            dna: dna,
            generation: 0,
            signal: None,
            last_action: eval::PerformableAction::NoAction,
            id,
            parents: (CreatureID(0), CreatureID(0)),
            stats: CreatureStats::default(),
        }
    }

    pub fn hash(&self) -> u32 {
        self.dna
            .seeded_hash(CreatureID::parents_to_u32(self.parents))
    }

    pub fn next_decision(&mut self) -> Rc<Decision> {
        self.thought_cycle.next()
    }

    pub fn feeder() -> Creature {
        let dna = dna::DNA::feeder();
        // We know the feeder dna is fine, so unwrapping
        let thought_cycle = cycle_detect(&dna).unwrap();
        Creature {
            id: CreatureID::feeder(),
            dna,
            inv: vec![dna::lex::Item::Food],
            energy: 1,
            thought_cycle,
            generation: 0,
            signal: Some(dna::lex::Signal::Green),
            last_action: eval::PerformableAction::NoAction,
            parents: (CreatureID(0), CreatureID(0)),
            stats: CreatureStats::default(),
        }
    }

    pub fn is_feeder(&self) -> bool {
        self.id.is_feeder()
    }

    pub fn attr(&self, attr: lex::Attribute) -> usize {
        match attr {
            lex::Attribute::Energy => self.energy(),
            lex::Attribute::Signal => match self.signal {
                Some(sig) => sig as usize,
                None => 0,
            },
            lex::Attribute::Generation => self.generation,
            lex::Attribute::Kills => self.stats.kills,
            lex::Attribute::Survived => self.stats.survived,
            lex::Attribute::NumChildren => self.stats.num_children,
            lex::Attribute::TopItem => match self.top_item() {
                Some(item) => item as usize,
                None => 0,
            },
        }
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
        self.stats.survived += 1;
        // Want to reset the action at the end of every encounter
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

    pub fn dead(&self) -> bool {
        !self.alive()
    }

    pub fn alive(&self) -> bool {
        self.energy > 0 && (!self.is_feeder() || self.has_items())
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

    pub fn has_eaten(&mut self) {
        self.stats.eaten += 1;
    }

    pub fn has_killed(&mut self) {
        self.stats.kills += 1;
    }

    pub fn mate_with(
        &mut self,
        other: &mut Creature,
        id_giver: &mut IDGiver,
        rng: &mut RngState,
    ) -> (Result<Creature, parsing::Failure>, GlobalStatistics) {
        let (child_dna, stats) = dna::DNA::combine(&self.dna, &other.dna, rng);
        let maybe_child = Creature::new(
            id_giver.next_creature_id(),                // id
            child_dna,                                  // dna
            max(self.generation, other.generation) + 1, // generation
            (self.id, other.id),                        // parents
        );
        if maybe_child.is_ok() {
            self.stats.num_children += 1;
            other.stats.num_children += 1;
        }
        (maybe_child, stats)
    }

    pub fn pay_for_mating(&mut self, share: usize) -> bool {
        let mut cost = (settings::MATING_COST as f64 * (share as f64 / 100.0))
            .round() as isize;
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
                    info!("{} takes {:?} from {}", self, item, other);
                    self.add_item(item);
                }
                None => {
                    debug!(
                        "{} tries to take an item from {}, \
                         but there's nothing to take.",
                        self, other
                    );
                }
            },
            eval::PerformableAction::Wait => debug!("{} waits", self),
            // This is only defending with no corresponding attack
            eval::PerformableAction::Defend(dmg) => {
                debug!("{} defends with {:?} fruitlessly", self, dmg)
            }
            eval::PerformableAction::Flee => {
                let mut rng = RngState::from_creatures(self, other);
                let my_roll = rng.rand_range(0, self.energy);
                let other_roll = rng.rand_range(0, other.energy);
                let dmg = rng.rand_range(0, 4);
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
            invalid_action => {
                panic!("Shouldn't have gotten {:?} here", invalid_action)
            }
        }
        arena::FightStatus::Continue
    }
}

/// Needed because some parts aren't serialized because they can be
/// inferred from other fields
#[derive(Deserialize)]
pub struct DeserializableCreature {
    dna: dna::DNA,
    inv: Vec<dna::lex::Item>,
    energy: usize,
    generation: usize,
    signal: Option<dna::lex::Signal>,
    last_action: eval::PerformableAction,
    id: CreatureID,
    parents: (CreatureID, CreatureID),
    stats: CreatureStats,
}

impl DeserializableCreature {
    pub fn into_creature(self) -> Creature {
        let DeserializableCreature {
            dna,
            inv,
            energy,
            generation,
            signal,
            last_action,
            id,
            parents,
            stats,
        } = self;
        // Invalid creatures are never serialized, so unwrapping
        let thought_cycle = cycle_detect(&dna).unwrap();
        Creature {
            dna,
            inv,
            thought_cycle,
            energy,
            generation,
            signal,
            last_action,
            id,
            parents,
            stats,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Creatures {
    creatures: Vec<Creature>,
    max_pop_size: usize,
    feeder_count: usize,
    #[serde(skip)]
    rng: RngState,
    #[serde(skip)]
    id_giver: IDGiver,
}

impl Creatures {
    fn from_pieces(
        id_giver: IDGiver,
        max_pop_size: usize,
        rng: RngState,
    ) -> Creatures {
        let mut idgv = id_giver;
        let creatures = (0..max_pop_size)
            .map(|_idx| Creature::seed_creature(idgv.next_creature_id()))
            .collect();
        Creatures {
            creatures,
            max_pop_size,
            feeder_count: 0,
            rng,
            id_giver,
        }
    }

    pub fn new(max_pop_size: usize) -> Creatures {
        Creatures::from_pieces(
            IDGiver::unthreaded(),
            max_pop_size,
            RngState::default(),
        )
    }

    /// Create a new population, one for each thread
    pub fn per_thread(
        num_threads: usize,
        max_pop_size: usize,
    ) -> Vec<Creatures> {
        assert!(
            max_pop_size % num_threads == 0,
            "Max population size must be a multiple of the number of threads"
        );
        let pop_per_thread = max_pop_size / num_threads;
        let mut rng = RngState::default();

        IDGiver::per_thread(num_threads)
            .into_iter()
            .map(|id_giver|
                Creatures::from_pieces(
                    id_giver,
                    pop_per_thread,
                    rng.spawn())
            )
            .collect()
    }

    /// Split an existing population that's been loaded from disk into
    /// the specified number of threads
    pub fn split_by_thread(self, num_threads: usize) -> Vec<Creatures> {
        let Creatures {
            mut creatures,
            max_pop_size,
            feeder_count,
            mut rng,
            id_giver,
        } = self;
        let pop_rem = max_pop_size % num_threads;
        let pop_div = max_pop_size / num_threads;
        let creat_rem = creatures.len() % num_threads;
        let creat_div = creatures.len() / num_threads;
        let feed_rem = feeder_count % num_threads;
        let feed_div = feeder_count / num_threads;
        id_giver.into_threads(num_threads)
            .into_iter()
            .enumerate()
            .map(|(i, idg)| Creatures {
                max_pop_size: if i >= pop_rem {pop_div} else {pop_div + 1},
                feeder_count: if i >= feed_rem {feed_div} else {feed_div + 1},
                rng: rng.spawn(),
                id_giver: idg,
                creatures: if i >= creat_rem {
                    creatures.drain(0..creat_div)
                } else {
                    creatures.drain(0..(creat_div + 1))
                }.collect()
            })
            .collect()
    }


    pub fn id_giver(&mut self) -> &mut IDGiver {
        &mut self.id_giver
    }

    pub fn len(&self) -> usize {
        self.creatures.len()
    }

    pub fn refill_feeders(&mut self) {
        if self.len() + self.feeder_count < self.max_pop_size {
            self.feeder_count =
                self.max_pop_size - (self.feeder_count + self.len());
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

    pub fn absorb_all(&mut self, creats: Vec<Creature>) {
        for creature in creats {
            self.absorb(creature)
        }
    }

    pub fn shuffle(&mut self) {
        self.rng.shuffle(self.creatures.as_mut_slice())
    }
}

#[derive(Deserialize)]
pub struct DeserializableCreatures {
    creatures: Vec<DeserializableCreature>,
    max_pop_size: usize,
    feeder_count: usize,
}

impl DeserializableCreatures {
    pub fn into_creatures(self) -> Creatures {
        let DeserializableCreatures {
            creatures: deserialized_creatures,
            max_pop_size,
            feeder_count,
        } = self;
        let max_id = deserialized_creatures
            .iter()
            .fold(0, |max_id, creature| max(max_id, creature.id.0));
        let creatures = deserialized_creatures
            .into_iter()
            .map(|x| x.into_creature())
            .collect();
        Creatures {
            creatures,
            max_pop_size,
            feeder_count,
            rng: RngState::default(),
            id_giver: IDGiver::new(max_id + 1, 1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_by_thread_divides_evenly() {
        let id_giver = IDGiver::new(14, 1);
        let creats = Creatures {
            id_giver,
            rng: RngState::default(),
            feeder_count: 3,
            max_pop_size: 10,
            creatures: vec![
                Creature::seed_creature(CreatureID(1)),
                Creature::seed_creature(CreatureID(3)),
                Creature::seed_creature(CreatureID(5)),
                Creature::seed_creature(CreatureID(7)),
                Creature::seed_creature(CreatureID(9)),
                Creature::seed_creature(CreatureID(11)),
                Creature::seed_creature(CreatureID(13)),
            ],
        };
        let mut res = creats.split_by_thread(3);
        assert_eq!(res.len(), 3);
        let one = res.remove(0);
        let two = res.remove(0);
        let three = res.remove(0);
        assert_eq!(one.creatures.len(), 3);
        assert_eq!(one.creatures[0].id, CreatureID(1));
        assert_eq!(one.creatures[1].id, CreatureID(3));
        assert_eq!(one.creatures[2].id, CreatureID(5));
        assert_eq!(one.max_pop_size, 4);
        assert_eq!(one.feeder_count, 1);
        assert_eq!(one.id_giver.next_id_to_give_out, 14);
        assert_eq!(one.id_giver.modulus, 3);

        assert_eq!(two.creatures.len(), 2);
        assert_eq!(two.creatures[0].id, CreatureID(7));
        assert_eq!(two.creatures[1].id, CreatureID(9));
        assert_eq!(two.max_pop_size, 3);
        assert_eq!(two.feeder_count, 1);
        assert_eq!(two.id_giver.next_id_to_give_out, 15);
        assert_eq!(two.id_giver.modulus, 3);

        assert_eq!(three.creatures.len(), 2);
        assert_eq!(three.creatures[0].id, CreatureID(11));
        assert_eq!(three.creatures[1].id, CreatureID(13));
        assert_eq!(three.max_pop_size, 3);
        assert_eq!(three.feeder_count, 1);
        assert_eq!(three.id_giver.next_id_to_give_out, 16);
        assert_eq!(three.id_giver.modulus, 3);
    }
}
