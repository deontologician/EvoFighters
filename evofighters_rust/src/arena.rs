use std::iter::IteratorExt;

use creatures;
use creatures::Creature;
use eval;
use parsing::Thought;
use util::AppState;

#[derive(Show, PartialEq, Eq, Copy)]
pub enum FightStatus {
    End, Continue,
}


pub fn encounter(p1: &mut Creature,
                 p2: &mut Creature,
                 app: &mut AppState) -> Vec<Creature> {

    let max_rounds = app.normal_sample(200.0, 30.0) as usize;
    let mut children: Vec<Creature> = Vec::with_capacity(5); // dynamically adjust?
    print1!("Max rounds: {}", max_rounds);
    // combine thought tree iterators, limit rounds
    let mut iterator = p1.iter().zip(p2.iter()).zip(range(0, max_rounds));
    for (thoughts, round) in iterator {
        print2!("Round {}", round);
        match thoughts {
            (Thought::Decision{tree: box tree1, icount:i1, skipped:s1},
             Thought::Decision{tree: box tree2, icount:i2, skipped:s2}) => {
                let p1_action = eval::evaluate(p1, p2, &tree1, app);
                let p2_action = eval::evaluate(p2, p1, &tree2, app);
                let (p1_cost, p2_cost) = (i1 + s1, i2 + s2);
                if p1_cost < p2_cost {
                    print3!("{} is going first", p1);

                } else if p2_cost > p1_cost {
                    print3!("{} is going first", p2);

                } else {
                    if app.rand() {
                        print3!("{} is going first", p1);
                    } else {
                        print3!("{} is going first", p2);
                    }
                }
            },
            (p1_thought, p2_thought) => {
                // Somebody was undecided, and the fight is over.
                p1.update_from_thought(&p1_thought);
                p2.update_from_thought(&p2_thought);
                print3!("The fight ended before it timed out");
                break;
            }
        }
    }
    children
}

struct Chances {
    chance_to_mate: usize,
    p1_chance_to_hit: usize,
    p2_chance_to_hit: usize,
    dmg1_multiplier: usize,
    dmg2_multiplier: usize,
    p1_share: usize,
    p2_share: usize,
}

impl Chances {
    fn new(
        chance_to_mate: usize,
        p1_chance_to_hit: usize,
        p2_chance_to_hit: usize,
        dmg1_multiplier: usize,
        dmg2_multiplier: usize,
        p1_share: usize,
        p2_share: usize,
        ) -> Chances {
        Chances {
            chance_to_mate: chance_to_mate,
            p1_chance_to_hit: p1_chance_to_hit,
            p2_chance_to_hit: p2_chance_to_hit,
            dmg1_multiplier: dmg1_multiplier,
            dmg2_multiplier: dmg2_multiplier,
            p1_share: p1_share,
            p2_share: p2_share,
        }
    }
}

fn damage_matrix(p1_act: eval::PerformableAction,
                 p2_act: eval::PerformableAction) -> Chances {
    use eval::PerformableAction as PA;
    match (p1_act, p2_act) {
        (PA::Attack(..), PA::Attack(..)) =>
                               Chances::new(  0, 75, 75, 50, 50,  0,  6),
        (PA::Attack(..), PA::Defend(..)) =>
                               Chances::new(  0, 25, 25, 25, 25,  0,  0),
        (PA::Attack(..),       PA::Mate) =>
                               Chances::new( 50, 50,  0, 75,  0, 70, 30),
        (PA::Attack(..),              _) =>
                               Chances::new(  0,100,  0,100,  0,  0,  0),
        (PA::Defend(..), PA::Defend(..)) =>
                               Chances::new(  0,  0,  0,  0,  0,  0,  0),
        (PA::Defend(..),       PA::Mate) =>
                               Chances::new( 25,  0,  0,  0,  0, 70, 30),
        (PA::Defend(..), PA::Attack(..)) =>
                               Chances::new(  0, 25, 25, 25, 25,  0,  0),
        (PA::Defend(..),              _) =>
                               Chances::new(  0,  0,  0,  0,  0,  0,  0),
        (PA::Mate,             PA::Mate) =>
                               Chances::new(100,  0,  0,  0,  0, 50, 50),
        (PA::Mate,       PA::Attack(..)) =>
                               Chances::new( 50,  0, 50,  0, 75, 30, 70),
        (PA::Mate,       PA::Defend(..)) =>
                               Chances::new( 25,  0,  0,  0,  0, 30, 70),
        (PA::Mate,                    _) =>
                               Chances::new( 75,  0,  0,  0,  0,  0,100),
        (_,              PA::Attack(..)) =>
                               Chances::new(  0,  0,100,  0,100,  0,  0),
        (_,              PA::Defend(..)) =>
                               Chances::new(  0,  0,  0,  0,  0,  0,  0),
        (_,                    PA::Mate) =>
                               Chances::new( 75,  0,  0,  0,  0,100,  0),
        (_,                           _) =>
                               Chances::new(  0,  0,  0,  0,  0,  0,  0),
    }
}

fn do_round(p1: &mut Creature,
            p1_act: eval::PerformableAction,
            p2: &mut Creature,
            p2_act: eval::PerformableAction,
            app: &mut AppState) -> Option<Creature> {
    let mults = damage_matrix(p1_act, p2_act);
    fn damage_fun(chance: usize,
                  mult: usize,
                  app: &mut AppState) -> usize {
        if app.rand_range(0, 100) <= chance {
            app.rand_range(1, ((mult as f64/100.0) * 6.0) as usize)
        } else {
            0
        }
    }
    let p1_dmg = damage_fun(
        mults.p1_chance_to_hit, mults.dmg1_multiplier, app);
    let p2_dmg = damage_fun(
        mults.p2_chance_to_hit, mults.dmg2_multiplier, app);
    // TODO: take into account damage type
    if p1_dmg > 0 {
        print1!("{} takes {} damage", p2, p1_dmg);
        p2.energy -= p1_dmg
    }
    if p2_dmg > 0 {
        print1!("{} takes {} damage", p1, p2_dmg);
        p2.energy -= p2_dmg
    }

    // we reverse the order of p1, p2 when calling try_to_mate because
    // paying costs first in mating is worse, and in this function p1
    // is preferred in actions that happen to both creatures in
    // order. Conceivably, p2 could die without p1 paying any cost at
    // all, even if p2 initiated mating against p1's will
    let maybe_child = creatures::try_to_mate(
        mults.chance_to_mate,
        p2,
        mults.p2_share,
        p1,
        mults.p1_share,
        app);
    panic!("Oh noes")
}
