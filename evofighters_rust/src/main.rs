#![feature(rand)]
#![feature(core)]
#![feature(box_syntax)]
#![feature(collections)]
#![feature(path)]
#![feature(std_misc)]
#![feature(io)]

extern crate time;
extern crate "rustc-serialize" as rustc_serialize;

#[macro_use]
pub mod util;
pub mod dna;
pub mod parsing;
pub mod eval;
pub mod settings;
pub mod creatures;
pub mod arena;

use creatures::Creature;
use std::iter::FromIterator;


fn main() {
    let mut app = util::AppState::new(settings::MAX_POPULATION_SIZE + 1);
    println!("Creating initial population");
    let mut population: Vec<Creature> = FromIterator::from_iter(
        (1..settings::MAX_POPULATION_SIZE + 1)
            .map(|id| Creature::seed_creature(id)));
    println!("Created {} creatures", settings::MAX_POPULATION_SIZE + 1);

    arena::simulate(&mut population, 0, 0, &mut app);

}
