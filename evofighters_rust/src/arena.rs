use std::cmp::{max};
use std::io::prelude::*;
use std::clone::{Clone};
use std::time::{Duration,Instant};
use std::fs::File;

use bincode::{serialize,Infinite};

use creatures;
use creatures::Creature;
use eval;
use settings;
use util::AppState;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum FightStatus {
    End, Continue,
}


fn encounter(p1: &mut Creature,
             p2: &mut Creature,
             app: &mut AppState) -> Option<Vec<Creature>> {
    use parsing::{Decision, Indecision};
    use creatures::Liveness::{Alive,Dead};
    let max_rounds = app.normal_sample(200.0, 30.0) as usize;
    let mut maybe_children: Option<Vec<Creature>> = None;
    info!("Max rounds: {}", max_rounds);
    // combine thought tree iterators, limit rounds
    let iterator = p1.iter().zip(p2.iter()).zip(0..max_rounds);
    let mut fight_timed_out = true;
    let mut p1_action = eval::PerformableAction::NoAction;
    let mut p2_action = eval::PerformableAction::NoAction;
    for (thoughts, round) in iterator {
        debug!("Round {}", round);
        app.rounds += 1;
        let (fight_status, maybe_child) = match thoughts {
            (Ok(Decision{tree: box tree1, icount:i1, skipped:s1, ..}),
             Ok(Decision{tree: box tree2, icount:i2, skipped:s2, ..})) => {
                debug!("{} thinks {:?}", p1, tree1);
                debug!("{} thinks {:?}", p2, tree2);
                p1_action = eval::evaluate(p1, p2, &tree1, app);
                p2_action = eval::evaluate(p2, p1, &tree2, app);
                let (p1_cost, p2_cost) = (i1 + s1, i2 + s2);
                if p1_cost < p2_cost {
                    trace!("{} is going first", p1);
                    trace!("{} intends to {}", p1, p1_action);
                    do_round(p1, p1_action, p2, p2_action, app)
                } else if p2_cost > p1_cost {
                    trace!("{} is going first", p2);
                    trace!("{} intends to {}", p2, p2_action);
                    do_round(p2, p2_action, p1, p1_action, app)
                } else if app.rand() {
                    trace!("{} is going first", p1);
                    do_round(p1, p1_action, p2, p2_action, app)
                } else {
                    trace!("{} is going first", p2);
                    do_round(p2, p2_action, p1, p1_action, app)
                }
            },
            (p1_thought, p2_thought) => {
                // Somebody was undecided, and the fight is over.
                p1.update_from_thought(&p1_thought);
                p2.update_from_thought(&p2_thought);
                if let Err(Indecision{reason, icount, skipped, ..}) = p1_thought {
                    info!("{} died because {:?}. using {} instructions,\
                        with {} skipped", p1, reason, icount, skipped);
                };
                if let Err(Indecision{reason, icount, skipped, ..}) = p2_thought {
                    info!("{} died because {:?}. using {} instructions,\
                        with {} skipped", p1, reason, icount, skipped);
                }
                trace!("The fight ended before it timed out");
                fight_timed_out = false;
                (FightStatus::End, None)
            }
        };
        if let Some(child) = maybe_child {
            match maybe_children {
                None => {
                    maybe_children = Some(vec![child]);
                },
                Some(ref mut children) => {
                    children.push(child)
                },
            }
            app.children_born += 1;
        }
        if let FightStatus::End = fight_status {
            fight_timed_out = false;
            break;
        }
        p1.last_action = p1_action;
        p2.last_action = p2_action;
    }
    if fight_timed_out {
        let penalty = app.rand_range(1, 7);
        info!("Time is up! both combatants take {} damage", penalty);
        p1.lose_energy(penalty);
        p2.lose_energy(penalty);
    }
    match (p1.liveness(), p2.liveness()) {
        (Alive, Dead)  => victory(p1, p2, app),
        (Dead, Alive)  => victory(p2, p1, app),
        (Dead, Dead)   => info!("Both {} and {} have died.", p1, p2),
        (Alive, Alive) => {
            p1.survived_encounter();
            p2.survived_encounter();
        }
    }
    maybe_children
}

fn victory(winner: &mut Creature, loser: &mut Creature, app: &mut AppState) {
    info!("{} has killed {}", winner, loser);
    winner.steal_from(loser);
    if loser.is_feeder() {
        app.feeders_eaten += 1;
        winner.eaten += 1;
        winner.gain_energy(app.rand_range(0, 1));
        winner.last_action = eval::PerformableAction::Wait;
    } else {
        winner.gain_energy(app.rand_range(0, settings::WINNER_LIFE_BONUS));
        winner.kills += 1;
        app.kills += 1;
        winner.survived_encounter();
    }
}

struct CreatureChance {
    chance_to_hit: usize,
    dmg_multiplier: usize,
    mating_share: usize,
}

impl CreatureChance {
    fn damage(&self, app: &mut AppState) -> usize {
        if app.rand_range(1, 101) <= self.chance_to_hit {
            let max_dmg = (self.dmg_multiplier * 6) / 100;
            app.rand_range(1, max(max_dmg, 1))
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

fn damage_matrix(p1_act: eval::PerformableAction,
                 p2_act: eval::PerformableAction) -> Chances {
    use eval::PerformableAction::{Attack, Defend, Mate};
    // TODO: take into account damage type
    match (p1_act, p2_act) {
        (Attack(..), Attack(..)) =>
            Chances{
                chance_to_mate: 0,
                p1: CreatureChance{
                    chance_to_hit: 75,
                    dmg_multiplier: 50,
                    mating_share: 0,
                },
                p2: CreatureChance{
                    chance_to_hit: 75,
                    dmg_multiplier: 50,
                    mating_share: 0,
                }
            },
        (Attack(..), Defend(..)) =>
            Chances{
                chance_to_mate: 0,
                p1: CreatureChance{
                    chance_to_hit: 25,
                    dmg_multiplier: 25,
                    mating_share: 0,
                },
                p2: CreatureChance{
                    chance_to_hit: 25,
                    dmg_multiplier: 25,
                    mating_share: 0,
                },
            },
        (Attack(..), Mate) =>
            Chances{
                chance_to_mate: 50,
                p1: CreatureChance{
                    chance_to_hit: 50,
                    dmg_multiplier: 75,
                    mating_share: 70,
                },
                p2: CreatureChance{
                    chance_to_hit: 0,
                    dmg_multiplier: 0,
                    mating_share: 30,
                },
            },
        (Attack(..), _) =>
            Chances{
                chance_to_mate: 0,
                p1: CreatureChance{
                    chance_to_hit: 100,
                    dmg_multiplier: 100,
                    mating_share: 0,
                },
                p2: CreatureChance{
                    chance_to_hit: 0,
                    dmg_multiplier: 0,
                    mating_share: 0,
                },
            },
        (Defend(..), Defend(..)) =>
            Chances{
                chance_to_mate: 0,
                p1: CreatureChance{
                    chance_to_hit: 0,
                    dmg_multiplier: 0,
                    mating_share: 0,
                },
                p2: CreatureChance{
                    chance_to_hit: 0,
                    dmg_multiplier: 0,
                    mating_share: 0,
                },
            },
        (Defend(..), Mate) =>
            Chances{
                chance_to_mate: 25,
                p1: CreatureChance{
                    chance_to_hit: 0,
                    dmg_multiplier: 0,
                    mating_share: 70,
                },
                p2: CreatureChance{
                    chance_to_hit: 0,
                    dmg_multiplier: 0,
                    mating_share: 30,
                },
            },
        (Defend(..), Attack(..)) =>
            Chances{
                chance_to_mate: 0,
                p1: CreatureChance{
                    chance_to_hit: 25,
                    dmg_multiplier: 25,
                    mating_share: 0,
                },
                p2: CreatureChance{
                    chance_to_hit: 25,
                    dmg_multiplier: 25,
                    mating_share: 0,
                },
            },
        (Defend(..), _) =>
            Chances{
                chance_to_mate: 0,
                p1: CreatureChance{
                    chance_to_hit: 0,
                    dmg_multiplier: 0,
                    mating_share: 0,
                },
                p2: CreatureChance{
                    chance_to_hit: 0,
                    dmg_multiplier: 0,
                    mating_share: 0,
                },
            },
        (Mate, Mate) =>
            Chances{
                chance_to_mate: 100,
                p1: CreatureChance{
                    chance_to_hit: 0,
                    dmg_multiplier: 0,
                    mating_share: 50,
                },
                p2: CreatureChance{
                    chance_to_hit: 0,
                    dmg_multiplier: 0,
                    mating_share: 50,
                },
            },
        (Mate, Attack(..)) =>
            Chances{
                chance_to_mate: 50,
                p1: CreatureChance{
                    chance_to_hit: 0,
                    dmg_multiplier: 0,
                    mating_share: 30,
                },
                p2: CreatureChance{
                    chance_to_hit: 50,
                    dmg_multiplier: 75,
                    mating_share: 70,
                },
            },
        (Mate, Defend(..)) =>
            Chances{
                chance_to_mate: 25,
                p1: CreatureChance{
                    chance_to_hit: 0,
                    dmg_multiplier: 0,
                    mating_share: 30,
                },
                p2: CreatureChance{
                    chance_to_hit: 0,
                    dmg_multiplier: 0,
                    mating_share: 70,
                },
            },
        (Mate, _) =>
            Chances{
                chance_to_mate: 75,
                p1: CreatureChance{
                    chance_to_hit: 0,
                    dmg_multiplier: 0,
                    mating_share: 0,
                },
                p2: CreatureChance{
                    chance_to_hit: 0,
                    dmg_multiplier: 0,
                    mating_share: 100,
                },
            },
        (_, Attack(..)) =>
            Chances{
                chance_to_mate: 0,
                p1: CreatureChance{
                    chance_to_hit: 0,
                    dmg_multiplier: 0,
                    mating_share: 0,
                },
                p2: CreatureChance{
                    chance_to_hit: 100,
                    dmg_multiplier: 100,
                    mating_share: 0,
                },
            },
        (_, Defend(..)) =>
            Chances{
                chance_to_mate: 0,
                p1: CreatureChance{
                    chance_to_hit: 0,
                    dmg_multiplier: 0,
                    mating_share: 0,
                },
                p2: CreatureChance{
                    chance_to_hit: 0,
                    dmg_multiplier: 0,
                    mating_share: 0,
                },
            },
        (_, Mate) =>
            Chances{
                chance_to_mate: 75,
                p1: CreatureChance{
                    chance_to_hit: 0,
                    dmg_multiplier: 0,
                    mating_share: 100,
                },
                p2: CreatureChance{
                    chance_to_hit: 0,
                    dmg_multiplier: 0,
                    mating_share: 0,
                },
            },
        (_, _) =>
            Chances{
                chance_to_mate: 0,
                p1: CreatureChance{
                    chance_to_hit: 0,
                    dmg_multiplier: 0,
                    mating_share: 0,
                },
                p2: CreatureChance{
                    chance_to_hit: 0,
                    dmg_multiplier: 0,
                    mating_share: 0,
                },
            },
    }
}

fn not_attack_mate_defend(act: eval::PerformableAction) -> bool {
    use eval::PerformableAction::{Signal,Eat,Take,Wait,Flee};
    match act {
        Signal(..) | Eat | Take | Wait | Flee => true,
        _ => false
    }
}

fn do_round(p1: &mut Creature,
            p1_act: eval::PerformableAction,
            p2: &mut Creature,
            p2_act: eval::PerformableAction,
            app: &mut AppState) -> (FightStatus,Option<Creature>) {
    let chances = damage_matrix(p1_act, p2_act);
    let p1_dmg = chances.p1.damage(app);
    let p2_dmg = chances.p2.damage(app);
    if p1_dmg > 0 {
        info!("{} takes {} damage", p2, p1_dmg);
        p2.lose_energy(p1_dmg)
    }
    if p2_dmg > 0 {
        info!("{} takes {} damage", p1, p2_dmg);
        p2.lose_energy(p2_dmg)
    }

    // we reverse the order of p1, p2 when calling try_to_mate because
    // paying costs first in mating is worse, and in this function p1
    // is preferred in actions that happen to both creatures in
    // order. Conceivably, p2 could die without p1 paying any cost at
    // all, even if p2 initiated mating against p1's will
    let mut maybe_child: Option<Creature> = creatures::try_to_mate(
        chances.chance_to_mate,
        p2,
        chances.p2.mating_share,
        p1,
        chances.p1.mating_share,
        app);
    maybe_child = maybe_child.and_then(
        |child| {
            info!("{} and {} have a child named {}", p1, p2, child);
            if child.dead() {
                info!("But it was stillborn since it has no dna");
                None
            } else {
                Some(child)
            }
        });
    if not_attack_mate_defend(p1_act) {
        if let FightStatus::End = p1.carryout(p2, p1_act, app) {
            return (FightStatus::End, maybe_child)
        }
    }
    if not_attack_mate_defend(p2_act) {
        if let FightStatus::End = p2.carryout(p1, p2_act, app) {
            return (FightStatus::End, maybe_child)
        }
    }
    trace!("{} has {} life left", p1, p1.energy());
    trace!("{} has {} life left", p2, p2.energy());
    (FightStatus::Continue, maybe_child)
}

fn random_encounter(population: &mut Vec<Creature>,
                    feeder_count: usize,
                    copy: bool,
                    app: &mut AppState) -> (Creature, Creature) {
    let p1_index = app.rand_range(0, population.len());
    let mut p2_index = app.rand_range(0, population.len() + feeder_count);
    while p1_index == p2_index {
        p2_index = app.rand_range(0, population.len() + feeder_count);
    }
    let p1;
    let p2;
    if copy {
        p1 = population[p1_index].clone();
    } else {
        p1 = population.swap_remove(p1_index);
    }
    if p2_index < population.len() {
        if copy {
            p2 = population[p2_index].clone();
        }else {
            p2 = population.swap_remove(p2_index);
        }
    } else {
        p2 = creatures::Creature::feeder();
    }
    (p1, p2)
}

fn post_encounter_cleanup(
    p1: Creature,
    population: &mut Vec<Creature>,
    feeders: &mut usize,
    ) {
    use creatures::Liveness::{Alive,Dead};
    match (p1.is_feeder(), p1.liveness()) {
        (false, Alive) => population.push(p1),
        (false, Dead) => (),
        (true, Dead) => *feeders = feeders.saturating_sub(1),
        (true, Alive) => (),
    }
}

enum SimStatus {
    NotEnoughCreatures,
    Apocalypse,
}

#[derive(Debug,Deserialize,Serialize)]
struct SaveFile {
    max_thinking_steps: usize,
    max_tree_depth: usize,
    max_inv_size: usize,
    default_energy: usize,
    mating_cost: usize,
    mutation_rate: f64,
    max_gene_value: i8,
    winner_life_bonus: usize,
    max_population_size: usize,
    gene_min_size: usize,
    num_encounters: usize,
    feeder_count: usize,
    creatures: Vec<Creature>,
}

fn save(creatures: &Vec<Creature>,
        feeder_count: usize,
        num_encounters: usize) {
    let savefile = SaveFile {
        max_thinking_steps: settings::MAX_THINKING_STEPS,
        max_tree_depth: settings::MAX_TREE_DEPTH,
        max_inv_size: settings::MAX_INV_SIZE,
        default_energy: settings::DEFAULT_ENERGY,
        mating_cost: settings::MATING_COST,
        mutation_rate: settings::MUTATION_RATE,
        max_gene_value: settings::MAX_GENE_VALUE,
        winner_life_bonus: settings::WINNER_LIFE_BONUS,
        max_population_size: settings::MAX_POPULATION_SIZE,
        gene_min_size: settings::GENE_MIN_SIZE,
        num_encounters: num_encounters,
        feeder_count: feeder_count,
        creatures: creatures.clone(),
    };
    let encoded = match serialize(&savefile, Infinite) {
        Err(why) => panic!("couldn't encode savefile: {}", why),
        Ok(encoded) => encoded,
    };
    let mut save_file = File::create("evofighters.save").unwrap();
    save_file.write_all(encoded.as_ref()).unwrap();
}

/// Given an instant and how many events the thread slept for, will
/// return how many events the thread should sleep for next time, and
/// a percentage error in the last prediction
struct RateData {
    pub events_per_second: u64,
    pub events_to_sleep: u64,
    pub prediction_error: f64,
    pub fps: f64,
}

impl RateData {
    pub fn new(start_time: Instant, events_slept: u64) -> RateData {
        let wanted_duration = settings::DISPLAY_FPS.recip();
        let actual_duration = RateData::duration_to_f64(start_time.elapsed());
        let events_per_second = (events_slept as f64) / actual_duration;
        RateData {
            events_to_sleep: (wanted_duration * events_per_second) as u64,
            events_per_second: events_per_second as u64,
            prediction_error: 1.0 - (actual_duration / wanted_duration),
            fps: actual_duration.recip(),
        }
    }

    /// Just some garbage so we have an initial value for these things
    pub fn initial() -> RateData {
        RateData {
            events_to_sleep: settings::INITIAL_EVENTS_PER_SECOND_GUESS,
            events_per_second: 0,
            prediction_error: 0.0,
            fps: 30.0,
        }
    }

    /// Takes a standard duration and returns an f64 representing
    /// seconds
    fn duration_to_f64(dur: Duration) -> f64 {
        let seconds = dur.as_secs() as f64;
        let subseconds = (dur.subsec_nanos() as f64) / 1_000_000_000.0;
        seconds + subseconds
    }
}


pub fn simulate(creatures: &mut Vec<Creature>,
            feeder_count: usize,
            num_encounters: usize,
            app: &mut AppState) {
    let mut timestamp = Instant::now();
    let mut feeders = feeder_count;
    let mut encounters = num_encounters;
    let mut total_events = 0;
    let mut events_since_last_print = 0;
    let mut rates = RateData::initial();
    let sim_status;
    loop {
        if events_since_last_print == rates.events_to_sleep {
            rates = RateData::new(timestamp, events_since_last_print);
            print!("\rCreatures: {creatures}, \
                    Feeders: {feeders}, \
                    F/C: {feeder_creature:.3}, \
                    Mutations: {mutations}, Events: {events}, \
                    Born: {born}, Eaten: {eaten}, kills: {kills}, \
                    eps: {eps}, err: {err:.1}%, \
                    FPS: {fps:.1}       ",
                   creatures = creatures.len(),
                   feeders = feeders,
                   feeder_creature = feeders as f64 / creatures.len() as f64,
                   mutations = app.mutations,
                   events = total_events,
                   born = app.children_born,
                   eaten = app.feeders_eaten,
                   kills = app.kills,
                   eps = rates.events_per_second,
                   err = rates.prediction_error * 100.0,
                   fps = rates.fps,
            );
            timestamp = Instant::now();
            events_since_last_print = 0;
        }
        if creatures.len() < 2 {
            sim_status = SimStatus::NotEnoughCreatures;
            break;
        }
        // if timestamp - update_time > Duration::from_secs(90) {
        //     println!("\nCurrently {} creatures alive\n", creatures.len());
        //     save(creatures, feeders, encounters);
        //     println!("Saved.");
        //     update_time = Instant::now();
        // }
        if (creatures.len() + feeders) < settings::MAX_POPULATION_SIZE {
            feeders += 1;
        }
        let (mut p1, mut p2) = random_encounter(creatures, feeders, false, app);
        info!("{} encounters {} in the wild", p1, p2);
        if let Some(ref mut new_children) = encounter(&mut p1, &mut p2, app) {
            creatures.append(new_children);
        }
        if !p1.is_feeder() && !p2.is_feeder() {
            encounters += 1;
        }
        total_events += 1;
        events_since_last_print += 1;
        post_encounter_cleanup(p1, creatures, &mut feeders);
        post_encounter_cleanup(p2, creatures, &mut feeders);
    }
    match sim_status {
        SimStatus::NotEnoughCreatures => {
            println!("You need at least two creatures in your population \
                     to have an encounter. Unfortunately, this means the \
                     end for your population.");
            if creatures.len() == 1 {
                println!("Here is the last of its kind:\n{:?}", creatures[0]);
            }
        },
        SimStatus::Apocalypse => {
            println!("Whelp, looks like the world ended!");
        }
    }
}
