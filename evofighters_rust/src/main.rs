#![feature(rand)]
#![feature(core)]
#![feature(box_syntax)]

extern crate time;
extern crate "rustc-serialize" as rustc_serialize;

use std::rc;
use rustc_serialize::json;

#[macro_use]
pub mod util;
pub mod dna;
pub mod parsing;
pub mod eval;
pub mod settings;
pub mod creatures;
pub mod arena;

fn main() {
    let mut app = util::AppState::new(1);
    println!("Creating initial population");
    let mut population: dna::DNA = (1..settings::MAX_POPULATION_SIZE).map(
        |id|).from_iter();


    arena::simulate(&mut creature_1, &mut creature_2, &mut app);

}
