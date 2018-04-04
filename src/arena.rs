use std::cmp::max;
use std::mem;
use std::io;
use std::io::Write;
use std::cell::Cell;

use creatures::{Creature, Creatures, IDGiver};
use eval;
use eval::PerformableAction;
use parsing::Decision;

use saver::{OwnedCheckpoint, Saver, Settings};
use stats::{GlobalStatistics, TimeKeeper, EncounterStats};
use rng::RngState;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum FightStatus {
    End,
    Continue,
}

struct CreatureChance {
    chance_to_hit: usize,
    dmg_multiplier: usize,
    mating_share: usize,
}

impl CreatureChance {
    fn damage(&self, rng: &mut RngState) -> usize {
        if rng.rand_range(1, 101) <= self.chance_to_hit {
            let max_dmg = (self.dmg_multiplier * 6) / 100;
            rng.rand_range(1, max(max_dmg, 1))
        } else {
            0
        }
    }
}

struct Chances {
    chance_to_mate: usize,
    p1: CreatureChance,
    p2: CreatureChance,
}

fn damage_matrix(
    p1_act: PerformableAction,
    p2_act: PerformableAction,
) -> Chances {
    use self::PerformableAction::{Attack, Defend, Mate};
    // TODO: take into account damage type
    match (p1_act, p2_act) {
        (Attack(..), Attack(..)) => Chances {
            chance_to_mate: 0,
            p1: CreatureChance {
                chance_to_hit: 75,
                dmg_multiplier: 50,
                mating_share: 0,
            },
            p2: CreatureChance {
                chance_to_hit: 75,
                dmg_multiplier: 50,
                mating_share: 0,
            },
        },
        (Attack(..), Defend(..)) | (Defend(..), Attack(..)) => Chances {
            chance_to_mate: 0,
            p1: CreatureChance {
                chance_to_hit: 25,
                dmg_multiplier: 25,
                mating_share: 0,
            },
            p2: CreatureChance {
                chance_to_hit: 25,
                dmg_multiplier: 25,
                mating_share: 0,
            },
        },
        (Attack(..), Mate) => Chances {
            chance_to_mate: 50,
            p1: CreatureChance {
                chance_to_hit: 50,
                dmg_multiplier: 75,
                mating_share: 70,
            },
            p2: CreatureChance {
                chance_to_hit: 0,
                dmg_multiplier: 0,
                mating_share: 30,
            },
        },
        (Attack(..), _) => Chances {
            chance_to_mate: 0,
            p1: CreatureChance {
                chance_to_hit: 100,
                dmg_multiplier: 100,
                mating_share: 0,
            },
            p2: CreatureChance {
                chance_to_hit: 0,
                dmg_multiplier: 0,
                mating_share: 0,
            },
        },
        (Defend(..), Mate) => Chances {
            chance_to_mate: 25,
            p1: CreatureChance {
                chance_to_hit: 0,
                dmg_multiplier: 0,
                mating_share: 70,
            },
            p2: CreatureChance {
                chance_to_hit: 0,
                dmg_multiplier: 0,
                mating_share: 30,
            },
        },
        (Mate, Mate) => Chances {
            chance_to_mate: 100,
            p1: CreatureChance {
                chance_to_hit: 0,
                dmg_multiplier: 0,
                mating_share: 50,
            },
            p2: CreatureChance {
                chance_to_hit: 0,
                dmg_multiplier: 0,
                mating_share: 50,
            },
        },
        (Mate, Attack(..)) => Chances {
            chance_to_mate: 50,
            p1: CreatureChance {
                chance_to_hit: 0,
                dmg_multiplier: 0,
                mating_share: 30,
            },
            p2: CreatureChance {
                chance_to_hit: 50,
                dmg_multiplier: 75,
                mating_share: 70,
            },
        },
        (Mate, Defend(..)) => Chances {
            chance_to_mate: 25,
            p1: CreatureChance {
                chance_to_hit: 0,
                dmg_multiplier: 0,
                mating_share: 30,
            },
            p2: CreatureChance {
                chance_to_hit: 0,
                dmg_multiplier: 0,
                mating_share: 70,
            },
        },
        (Mate, _) => Chances {
            chance_to_mate: 75,
            p1: CreatureChance {
                chance_to_hit: 0,
                dmg_multiplier: 0,
                mating_share: 0,
            },
            p2: CreatureChance {
                chance_to_hit: 0,
                dmg_multiplier: 0,
                mating_share: 100,
            },
        },
        (_, Attack(..)) => Chances {
            chance_to_mate: 0,
            p1: CreatureChance {
                chance_to_hit: 0,
                dmg_multiplier: 0,
                mating_share: 0,
            },
            p2: CreatureChance {
                chance_to_hit: 100,
                dmg_multiplier: 100,
                mating_share: 0,
            },
        },
        (_, Mate) => Chances {
            chance_to_mate: 75,
            p1: CreatureChance {
                chance_to_hit: 0,
                dmg_multiplier: 0,
                mating_share: 100,
            },
            p2: CreatureChance {
                chance_to_hit: 0,
                dmg_multiplier: 0,
                mating_share: 0,
            },
        },
        (_, _) => Chances {
            chance_to_mate: 0,
            p1: CreatureChance {
                chance_to_hit: 0,
                dmg_multiplier: 0,
                mating_share: 0,
            },
            p2: CreatureChance {
                chance_to_hit: 0,
                dmg_multiplier: 0,
                mating_share: 0,
            },
        },
    }
}

fn not_attack_mate_defend(act: PerformableAction) -> bool {
    use self::PerformableAction::{Eat, Flee, Signal, Take, Wait};
    match act {
        Signal(..) | Eat | Take | Wait | Flee => true,
        _ => false,
    }
}

enum SimStatus {
    NotStarted,
    EverythingRunningFine,
    NotEnoughCreatures,
}

pub struct Arena {
    rng: RngState,
    population: Creatures,
    settings: Settings,
    stats: GlobalStatistics,
    total_events: u64,
    encounters: u64,
    events_since_last_print: u64,
    events_since_last_save: u64,
    rates: RateData,
    saver: Saver,
    sim_status: SimStatus,
}

impl Arena {
    pub fn new(
        population: Creatures,
        filename: &str,
        settings: Settings,
    ) -> Arena {
        Arena {
            rng: RngState::default(),
            population,
            settings,
            stats: GlobalStatistics::new(),
            total_events: 0,
            encounters: 0,
            events_since_last_print: 0,
            events_since_last_save: 0,
            rates: RateData::initial(),
            saver: Saver::new(filename),
            sim_status: SimStatus::NotStarted,
        }
    }

    pub fn from_checkpoint(
        checkpoint: OwnedCheckpoint,
        filename: &str,
    ) -> Arena {
        let OwnedCheckpoint {
            creatures,
            stats,
            settings,
        } = checkpoint;
        let mut arena = Arena::new(creatures, filename, settings);
        arena.stats = stats;
        arena
    }

    fn maybe_print_status(&mut self, timestamp: Instant) -> Instant {
        if self.events_since_last_print == self.rates.events_to_sleep {
            self.rates = RateData::new(
                timestamp,
                self.events_since_last_print,
                self.settings.metric_fps,
            );
            print!(
                "\rCreatures: {creatures}, \
                 Feeders: {feeders}, \
                 F/C: {feeder_creature:.3}, \
                 Events: {events}, \
                 Born: {born}, Eaten: {eaten}, kills: {kills}, \
                 eps: {eps}, \
                 FPS: {fps:.1}       ",
                creatures = self.population.len(),
                feeders = self.population.feeder_count(),
                feeder_creature = self.population.feeder_count() as f64
                    / self.population.len() as f64,
                events = self.total_events,
                born = self.stats.children_born,
                eaten = self.stats.feeders_eaten,
                kills = self.stats.kills,
                eps = self.rates.events_per_second,
                fps = self.rates.fps,
            );
            io::stdout().flush().unwrap_or(());
            self.events_since_last_print = 0;
            Instant::now()
        } else {
            timestamp
        }
    }

    fn maybe_save(&mut self) {
        if self.rates.events_per_second > 0
            && self.rates.events_per_second * 30 <= self.events_since_last_save
        {
            println!(
                "\nHit {} out of estimated {} events, one moment...",
                self.rates.events_per_second * 30,
                self.events_since_last_save,
            );
            // TODO: handle failed saves gracefully?
            self.saver.save(&self.population, &self.stats).unwrap();
            println!("Saved to file");
            self.events_since_last_save = 0;
        }
    }

    pub fn simulate(&mut self) {
        let mut timestamp = Instant::now();
        self.sim_status = SimStatus::EverythingRunningFine;
        while self.population.len() >= 2 {
            timestamp = self.maybe_print_status(timestamp);
            self.maybe_save();
            self.population.refill_feeders();
            let p1 = self.population.random_creature();
            let p2 = self.population.random_creature_or_feeder();

            info!("{} encounters {} in the wild", p1, p2);
            if !p1.is_feeder() && !p2.is_feeder() {
                self.encounters += 1;
            }
            let mut encounter = Encounter::new(
                p1,
                p2,
                self.settings.mutation_rate,
                &mut self.rng,
                self.population.id_giver(),
            );
            let EncounterResult { survivors, stats } = encounter.run();
            self.stats.absorb(stats);
            self.population.absorb_all(survivors);

            self.total_events += 1;
            self.events_since_last_save += 1;
            self.events_since_last_print += 1;
        }
        self.sim_status = SimStatus::NotEnoughCreatures;
        match self.sim_status {
            SimStatus::NotEnoughCreatures => {
                println!(
                    "You need at least two creatures in your population \
                     to have an encounter. Unfortunately, this means the \
                     end for your population."
                );
                if self.population.len() == 1 {
                    println!(
                        "Here is the last of its kind:\n{:?}",
                        self.population.random_creature()
                    )
                }
            }
            _ => unreachable!(),
        }
    }
}

pub struct EncounterResult {
    pub survivors: Vec<Creature>,
    pub stats: EncounterStats,
}

pub struct Encounter<'a> {
    pub p1: Creature,
    pub p2: Creature,
    pub stats: EncounterStats,
    pub children: Vec<Creature>,

    rng: &'a mut RngState,
    id_giver: &'a mut IDGiver,
    max_rounds: usize,
    mutation_rate: f64,
    p1_action: Cell<PerformableAction>,
    p2_action: Cell<PerformableAction>,
}

impl<'a> Encounter<'a> {
    pub fn new(
        p1: Creature,
        p2: Creature,
        mutation_rate: f64,
        rng: &'a mut RngState,
        id_giver: &'a mut IDGiver,
    ) -> Encounter<'a> {
        let max_rounds = rng.normal_sample(200.0, 30.0) as usize;
        Encounter {
            p1,
            p2,
            stats: EncounterStats::default(),
            children: Vec::new(),
            rng,
            id_giver,
            max_rounds,
            mutation_rate,
            p1_action: Cell::new(PerformableAction::NoAction),
            p2_action: Cell::new(PerformableAction::NoAction),
        }
    }

    fn decide_and_eval(&self) -> (usize, usize) {
        let &Decision {
            tree: ref tree1,
            icount: i1,
            skipped: s1,
            ..
        } = self.p1.next_decision();
        let &Decision {
            tree: ref tree2,
            icount: i2,
            skipped: s2,
            ..
        } = self.p2.next_decision();
        debug!("{} thinks {:?}", self.p1, tree1);
        debug!("{} thinks {:?}", self.p2, tree2);
        self.p1_action
            .set(eval::evaluate(&self.p1, &self.p2, tree1));
        self.p2_action
            .set(eval::evaluate(&self.p2, &self.p1, tree2));
        (i1 + s1, i2 + s2)
    }

    fn execute_round(&mut self, p1_cost: usize, p2_cost: usize) -> FightStatus {
        if p1_cost < p2_cost {
            trace!("{} is going first", self.p1);
            trace!("{} intends to {}", self.p1, self.p1_action.get());
            self.do_round()
        } else if p2_cost > p1_cost {
            trace!("{} is going first", self.p2);
            trace!("{} intends to {}", self.p2, self.p2_action.get());
            self.do_swapped_round()
        } else if self.rng.rand() {
            trace!("{} is going first", self.p1);
            self.do_round()
        } else {
            trace!("{} is going first", self.p2);
            self.do_swapped_round()
        }
    }

    pub fn run(mut self) -> EncounterResult {
        info!("Max rounds: {}", self.max_rounds);
        // combine thought tree iterators, limit rounds
        for round in 0..self.max_rounds {
            debug!("Round {}", round);
            self.stats.rounds += 1;
            let (p1_cost, p2_cost) = self.decide_and_eval();
            if let FightStatus::End = self.execute_round(p1_cost, p2_cost) {
                break;
            }
            self.p1.last_action = self.p1_action.get();
            self.p2.last_action = self.p2_action.get();
        }
        let Encounter {
            mut p1,
            mut p2,
            children: mut survivors,
            mut stats,
            mut rng,
            ..
        } = self;
        if p1.alive() && p2.dead() {
            Encounter::victory(&mut p1, &mut p2, &mut stats, rng);
            survivors.push(p1);
        } else if p1.dead() && p2.alive() {
            Encounter::victory(&mut p2, &mut p1, &mut stats, rng);
            survivors.push(p2);
        } else if p1.dead() && p2.dead() {
            info!("Both {} and {} have died.", p1, p2)
        } else {
            p1.survived_encounter();
            p2.survived_encounter();
            survivors.push(p1);
            survivors.push(p2);
        }
        EncounterResult { survivors, stats }
    }

    fn victory(
        p1: &mut Creature,
        p2: &mut Creature,
        stats: &mut EncounterStats,
        rng: &mut RngState,
    ) {
        info!("{} has killed {}", p1, p2);
        p1.steal_from(p2);
        if p2.is_feeder() {
            stats.feeders_eaten += 1;
            p1.has_eaten();
            p1.gain_energy(rng.rand_range(0, 1));
            p1.last_action = PerformableAction::Wait;
        } else {
            p1.gain_winner_energy(rng);
            p1.has_killed();
            stats.kills += 1;
            p1.survived_encounter();
        }
    }

    fn try_mating(
        &mut self,
        mating_chance: usize,
        first_share: usize,
        second_share: usize,
    ) -> Option<Creature> {
        if self.rng.rand_range(1, 101) > mating_chance || self.p2.dead()
            || self.p1.dead()
        {
            return None;
        }
        info!("{} tried to mate with {}!", self.p2, self.p1);
        if self.p2.is_feeder() || self.p1.is_feeder() {
            info!("{} tried to mate with {}", self.p2, self.p1);
            // Mating kills the feeder
            if self.p2.is_feeder() {
                self.p2.kill();
            }
            if self.p1.is_feeder() {
                self.p1.kill();
            }
            return None;
        }
        debug!("Attempting to mate");
        if self.p2.pay_for_mating(first_share)
            && self.p1.pay_for_mating(second_share)
        {
            debug!("Both paid their debts, so they get to mate");
            self.mate()
        } else {
            None
        }
    }

    fn mate(&mut self) -> Option<Creature> {
        let maybe_child = self.p1.mate_with(
            &mut self.p2,
            &mut self.id_giver,
            &mut self.rng,
            self.mutation_rate,
        );
        match maybe_child {
            Err(_) => {
                info!("Child didn't live since it had invalid dna.");
                None
            }
            Ok(child) => {
                info!(
                    "{} and {} have a child named {}",
                    self.p1, self.p2, child
                );
                Some(child)
            }
        }
    }

    //swap the players in this encounter, some things are dependent on order
    fn swap_players(&mut self) {
        mem::swap(&mut self.p1, &mut self.p2);
        self.p1_action.swap(&self.p2_action);
    }

    fn do_swapped_round(&mut self) -> FightStatus {
        self.swap_players();
        let result = self.do_round();
        self.swap_players();
        result
    }

    fn do_round(&mut self) -> FightStatus {
        let chances = damage_matrix(self.p1_action.get(), self.p2_action.get());
        let p1_dmg = chances.p1.damage(&mut self.rng);
        let p2_dmg = chances.p2.damage(&mut self.rng);
        if p1_dmg > 0 {
            info!("{} takes {} damage", self.p2, p1_dmg);
            self.p2.lose_energy(p1_dmg)
        }
        if p2_dmg > 0 {
            info!("{} takes {} damage", self.p1, p2_dmg);
            self.p2.lose_energy(p2_dmg)
        }

        // we reverse the order of p1, p2 when calling try_to_mate because
        // paying costs first in mating is worse, and in this function p1
        // is preferred in actions that happen to both creatures in
        // order. Conceivably, p2 could die without p1 paying any cost at
        // all, even if p2 initiated mating against p1's will
        let maybe_child = self.try_mating(
            chances.chance_to_mate,
            chances.p2.mating_share,
            chances.p1.mating_share,
        );
        if let Some(child) = maybe_child {
            self.children.push(child);
            self.stats.children_born += 1;
        };

        if not_attack_mate_defend(self.p1_action.get()) {
            if let FightStatus::End =
                self.p1.carryout(&mut self.p2, self.p1_action.get())
            {
                return FightStatus::End;
            }
        }
        if not_attack_mate_defend(self.p2_action.get()) {
            if let FightStatus::End =
                self.p2.carryout(&mut self.p1, self.p2_action.get())
            {
                return FightStatus::End;
            }
        }
        trace!("{} has {} life left", self.p1, self.p1.energy());
        trace!("{} has {} life left", self.p2, self.p2.energy());
        FightStatus::Continue
    }
}
