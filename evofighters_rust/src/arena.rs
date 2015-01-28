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
                break;
            }
        }
    }
    children
}

fn do_round(p1: &mut Creature,
            p1_act: eval::PerformableAction,
            p2: &mut Creature,
            p2_act: eval::PerformableAction,
            rng: &mut ThreadRng) -> Option<Creature> {
    panic!("Oh noes")
}
