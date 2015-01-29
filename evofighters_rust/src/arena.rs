use std::rand;
use std::rand::{Rng,ThreadRng};
use std::rand::distributions::{Normal, IndependentSample};
use std::iter::IteratorExt;

use creatures::Creature;
use eval;
use parsing::Thought;

#[derive(Show, PartialEq, Eq, Copy)]
pub enum FightStatus {
    End, Continue,
}

pub fn encounter(p1: &mut Creature, p2: &mut Creature, rng:
                 &mut ThreadRng) -> Vec<Creature> {

    let normal = Normal::new(200.0, 30.0);
    let max_rounds = normal.ind_sample(rng) as usize;
    let mut children: Vec<Creature> = Vec::with_capacity(5); // dynamically adjust?
    print1!("Max rounds: {}", max_rounds);
    // combine thought tree iterators, limit rounds
    let mut iterator = p1.iter().zip(p2.iter()).zip(range(0, max_rounds));
    for (thoughts, round) in iterator {
        print2!("Round {}", round);
        match thoughts {
            (Thought::Decision{tree: box tree1, icount:i1, skipped:s1},
             Thought::Decision{tree: box tree2, icount:i2, skipped:s2}) => {
                let p1_action = eval::evaluate(p1, p2, &tree1, rng);
                let p2_action = eval::evaluate(p2, p1, &tree2, rng);
                let (p1_cost, p2_cost) = (i1 + s1, i2 + s2);
                if p1_cost < p2_cost {
                    print3!("{} is going first", p1);

                } else if p2_cost > p1_cost {
                    print3!("{} is going first", p2);

                } else {
                    if rng.gen() {
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

enum Act {
    Attacking = 1,
    Defending,
    Mating,
    Other,
}

struct Chances {
    mate_chance: usize,
    p1_chance_to_hit: usize,
    p2_chance_to_hit: usize,
    dmg1_multiplier: usize,
    dmg2_multiplier: usize,
    p1_share: usize,
    p2_share: usize,
}

impl Chances {
    fn new(
        mate_chance: usize,
        p1_chance_to_hit: usize,
        p2_chance_to_hit: usize,
        dmg1_multiplier: usize,
        dmg2_multiplier: usize,
        p1_share: usize,
        p2_share: usize,
        ) -> Chances {
        Chances {
            mate_chance: mate_chance,
            p1_chance_to_hit: p1_chance_to_hit,
            p2_chance_to_hit: p2_chance_to_hit,
            dmg1_multiplier: dmg1_multiplier,
            dmg2_multiplier: dmg2_multiplier,
            p1_share: p1_share,
            p2_share: p2_share,
        }
    }
}

fn damage_matrix(p1_act: Act, p2_act: Act) -> Chances {
    match (p1_act, p2_act) {
        (Act::Attacking, Act::Attacking) =>
            Chances::new(  0, 75, 75, 50, 50,  0,  6),
        (Act::Attacking, Act::Defending) =>
            Chances::new(  0, 25, 25, 25, 25,  0,  0),
        (Act::Attacking, Act::Mating   ) =>
            Chances::new( 50, 50,  0, 75,  0, 70, 30),
        (Act::Attacking, Act::Other    ) =>
            Chances::new(  0,100,  0,100,  0,  0,  0),
        (Act::Defending, Act::Defending) =>
            Chances::new(  0,  0,  0,  0,  0,  0,  0),
        (Act::Defending, Act::Mating   ) =>
            Chances::new( 25,  0,  0,  0,  0, 70, 30),
        (Act::Defending, Act::Other    ) =>
            Chances::new(  0,  0,  0,  0,  0,  0,  0),
        (Act::Mating, Act::   Mating   ) =>
            Chances::new(100,  0,  0,  0,  0, 50, 50),
        (Act::Mating, Act::   Other    ) =>
            Chances::new( 75,  0,  0,  0,  0,  0,100),
        (Act::Other, Act::    Other    ) =>
            Chances::new(  0,  0,  0,  0,  0,  0,  0),
        // the rest are duplicates of the above with swapped order
        (Act::Defending, Act::Attacking) =>
            Chances::new(  0, 25, 25, 25, 25,  0,  0),
        (Act::Mating, Act::   Attacking) =>
            Chances::new( 50,  0, 50,  0, 75, 30, 70),
        (Act::Mating, Act::   Defending) =>
            Chances::new( 25,  0,  0,  0,  0, 30, 70),
        (Act::Other, Act::    Attacking) =>
            Chances::new(  0,  0,100,  0,100,  0,  0),
        (Act::Other, Act::    Defending) =>
            Chances::new(  0,  0,  0,  0,  0,  0,  0),
        (Act::Other, Act::    Mating   ) =>
            Chances::new( 75,  0,  0,  0,  0,100,  0),
    }
}

fn do_round(p1: &mut Creature,
            p1_act: eval::PerformableAction,
            p2: &mut Creature,
            p2_act: eval::PerformableAction,
            rng: &mut ThreadRng) -> Option<Creature> {

    panic!("Oh noes")
}
