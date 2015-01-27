use std::rand;
use std::rand::{Rng,ThreadRng};
use std::rand::distributions::{Normal, IndependentSample};
use std::iter::IteratorExt;

use creatures::Creature;
use eval;

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
    let mut iterator = range(0, max_rounds).zip(p1.iter()).zip(p2.iter());
    for ((round, p1_thought), p2_thought) in iterator {
        if !p1_thought.decided() || !p2_thought.decided() {
            p1.update_from_thought(&p1_thought);
            p2.update_from_thought(&p2_thought);
            break;
        }
        let p1_action = eval::evaluate(p1, p2, p1_thought.tree(), rng);
        let p2_action = eval::evaluate(p2, p1, p2_thought.tree(), rng);
    }

    children
}
