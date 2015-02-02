use std::iter::IteratorExt;
use std::cmp::{max};
use std::clone::{Clone};
use time;
use std::time::duration::Duration;
use std::old_io::File;
use rustc_serialize::json;

use creatures;
use creatures::Creature;
use eval;
use settings;
use util::AppState;

#[derive(Show, PartialEq, Eq, Copy)]
pub enum FightStatus {
    End, Continue,
}


pub fn encounter(p1: &mut Creature,
                 p2: &mut Creature,
                 app: &mut AppState) -> Vec<Creature> {
    use parsing::Thought::{Decision, Indecision};
    use creatures::Liveness::{Alive,Dead};
    let max_rounds = app.normal_sample(200.0, 30.0) as usize;
    let mut children: Vec<Creature> = Vec::with_capacity(5); // dynamically adjust?
    print1!("Max rounds: {}", max_rounds);
    // combine thought tree iterators, limit rounds
    let mut iterator = p1.iter().zip(p2.iter()).zip(0..max_rounds);
    let mut fight_timed_out = true;
    let mut p1_action;
    let mut p2_action;
    for (thoughts, round) in iterator {
        print2!("Round {}", round);
        let mut maybe_child;
        match thoughts {
            (Decision{tree: box tree1, icount:i1, skipped:s1},
             Decision{tree: box tree2, icount:i2, skipped:s2}) => {
                p1_action = eval::evaluate(p1, p2, &tree1, app);
                p2_action = eval::evaluate(p2, p1, &tree2, app);
                let (p1_cost, p2_cost) = (i1 + s1, i2 + s2);
                if p1_cost < p2_cost {
                    print3!("{} is going first", p1);
                    print3!("{} intends to {}", p1, p1_action);
                    maybe_child = do_round(p1, p1_action, p2, p2_action, app);
                } else if p2_cost > p1_cost {
                    print3!("{} is going first", p2);
                    print3!("{} intends to {}", p2, p2_action);
                    maybe_child = do_round(p2, p2_action, p1, p1_action, app);
                } else {
                    if app.rand() {
                        print3!("{} is going first", p1);
                        maybe_child = do_round(
                            p1, p1_action, p2, p2_action, app);
                    } else {
                        print3!("{} is going first", p2);
                        maybe_child = do_round(
                            p2, p2_action, p1, p1_action, app);
                    }
                }
            },
            (p1_thought, p2_thought) => {
                // Somebody was undecided, and the fight is over.
                p1.update_from_thought(&p1_thought);
                p2.update_from_thought(&p2_thought);
                match p1_thought {
                    Indecision{reason, icount, skipped} => {
                        print1!("{} died because {:?}. using {} instructions,\
                        with {} skipped", p1, reason, icount, skipped);
                    },
                    _ => ()
                }
                match p2_thought {
                    Indecision{reason, icount, skipped} => {
                        print1!("{} died because {:?}. using {} instructions,\
                        with {} skipped", p1, reason, icount, skipped);
                    },
                    _=> ()
                }
                print3!("The fight ended before it timed out");
                fight_timed_out = false;
                break;
            }
        }
        if let Some(child) = maybe_child {
            children.push(child)
        }
        if p1.dead() || p2.dead() {
            fight_timed_out = false;
            break;
        }
        p1.last_action = p1_action;
        p2.last_action = p2_action;
    }
    if fight_timed_out {
        let penalty = app.rand_range(1, 6);
        print1!("Time is up! both combatants take {} damage", penalty);
        p1.lose_energy(penalty);
        p2.lose_energy(penalty);
    }
    match (p1.liveness(), p2.liveness()) {
        (Alive, Dead)  => victory(p1, p2, app),
        (Dead, Alive)  => victory(p2, p1, app),
        (Dead, Dead)   => print1!("Both {} and {} have died.", p1, p2),
        (Alive, Alive) => {
            p1.survived_encounter();
            p2.survived_encounter();
        }
    }
    children
}

fn victory(winner: &mut Creature, loser: &mut Creature, app: &mut AppState) {
    print!("{} has killed {}", winner, loser);
    winner.steal_from(loser);
    if loser.is_feeder() {
        winner.eaten += 1;
        winner.gain_energy(app.rand_range(0, 1));
        winner.last_action = eval::PerformableAction::Wait;
    } else {
        winner.gain_energy(app.rand_range(0, settings::WINNER_LIFE_BONUS));
        winner.kills += 1;
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
            app: &mut AppState) -> Option<Creature> {
    let chances = damage_matrix(p1_act, p2_act);
    let p1_dmg = chances.p1.damage(app);
    let p2_dmg = chances.p2.damage(app);
    if p1_dmg > 0 {
        print1!("{} takes {} damage", p2, p1_dmg);
        p2.lose_energy(p1_dmg)
    }
    if p2_dmg > 0 {
        print1!("{} takes {} damage", p1, p2_dmg);
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
            print1!("{} and {} have a child named {}", p1, p2, child);
            if child.dead() {
                print1!("But it was stillborn since it has no dna");
                None
            } else {
                Some(child)
            }
        });
    if not_attack_mate_defend(p1_act) {
        if let FightStatus::End = p1.carryout(p2, p1_act, app) {
            return maybe_child
        }
    }
    if not_attack_mate_defend(p2_act) {
        if let FightStatus::End = p2.carryout(p1, p2_act, app) {
            return maybe_child
        }
    }
    print3!("{} has {} life left", p1, p1.energy());
    print3!("{} has {} life left", p2, p2.energy());
    maybe_child
}

fn random_encounter(population: &mut Vec<Creature>,
                    feeder_count: usize,
                    copy: bool,
                    app: &mut AppState) -> Option<(Creature, Creature)> {
    if population.len() < 2 {
        return None
    }
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
    Some((p1, p2))
}

fn post_encounter_cleanup(
    p1: Creature,
    p2: Creature,
    population: &mut Vec<Creature>,
    deadpool: &mut Vec<Creature>) {
    if p1.dead() {
        deadpool.push(p1);
    } else {
        population.push(p1);
    }
    if !p2.is_feeder() {
        if p2.dead() {
            deadpool.push(p2)
        } else {
            population.push(p2);
        }
    }
}

enum SimStatus {
    NotEnoughCreatures
}

#[derive(Show,RustcDecodable,RustcEncodable)]
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
    creatures: Vec<Creature>,
}

fn save(creatures: &Vec<Creature>) {
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
        creatures: creatures.clone(),
    };
    let encoded = match json::encode(&savefile) {
        Err(why) => panic!("couldn't encode savefile: {}", why),
        Ok(encoded) => encoded,
    };
    let path = Path::new("evofighters.save");
    let display = path.display();
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why.desc),
        Ok(file) => file,
    };
    match file.write_str(encoded.as_slice()) {
        Err(why) => panic!("Couldn't write to {}: {}", display, why.desc),
        Ok(_) => println!("Successfully saved to {}", display),
    }
}

fn simulate(creatures: &mut Vec<Creature>, app: &mut AppState) {
    let mut update_time = time::get_time();
    let mut timestamp = update_time;
    let sim_status;
    loop {
        let new_time = time::get_time();
        if creatures.len() < 2 {
            sim_status = SimStatus::NotEnoughCreatures;
            break;
        }
        if new_time - timestamp > Duration::seconds(90) {
            println!("\nCurrently {} creatures alive", creatures.len());

        }
    }
}
