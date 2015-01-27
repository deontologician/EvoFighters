#![allow(unstable)]
#![feature(box_syntax)]

use std::rand;
use std::rand::{ThreadRng,Rng};
use std::rand::distributions::{Normal,IndependentSample};
use std::rc;

#[macro_use]
pub mod util;
pub mod dna;
pub mod parsing;
pub mod eval;
pub mod settings;
pub mod creatures;
pub mod arena;

fn main() {
    let mut rng: ThreadRng = rand::thread_rng();
    let mut v: Vec<i8> = vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
    let mut v2: Vec<i8> = vec![0,0,0,0,0,0,0,0];

    for x in v.iter_mut() {
        *x = rng.gen_range(-1, 9);
    }

    let mut creature_1 = creatures::Creature::new(1, rc::Rc::new(v), 0, (0,0));
    let mut creature_2 = creatures::Creature::new(2, rc::Rc::new(v2), 0, (0,0));

    creature_1.add_item(dna::Item::GoodFood);
    creature_2.add_item(dna::Item::GoodFood);
    let mut idbox = 2;

    arena::encounter(&mut creature_1, &mut creature_2, &mut rng);
}
