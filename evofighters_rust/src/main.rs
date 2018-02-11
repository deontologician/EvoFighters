#![feature(box_patterns)]
#![feature(nll)]

extern crate time;
extern crate rand;
#[macro_use] extern crate enum_primitive;
#[macro_use] extern crate lazy_static;
extern crate num;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate xz2;


use std::env;

#[macro_use] pub mod util;
pub mod dna;
pub mod parsing;
pub mod eval;
pub mod settings;
pub mod creatures;
pub mod arena;
pub mod simplify;
pub mod saver;

use creatures::Creature;
use std::iter::FromIterator;


fn main() {
    let mut args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        run_simulation();
        return
    }
    let command = args[1].clone();
    match command.as_ref() {
        "simulate" => run_simulation(),
        "cycle-check" => cycle_check(&args.split_off(2)),
        _ => println!("Unrecognized command."),
    }
}

fn run_simulation() {
    let mut app = util::AppState::new(settings::MAX_POPULATION_SIZE + 1);
    println!("Creating initial population");
    let mut population: Vec<Creature> = FromIterator::from_iter(
        (1..settings::MAX_POPULATION_SIZE + 1)
            .map(Creature::seed_creature));
    println!("Created {} creatures", settings::MAX_POPULATION_SIZE + 1);

    arena::simulate(&mut population, 0, &mut app)
}

fn cycle_check(args: &[String]) {

    let dna_args: dna::DNA = dna::DNA::from(
        args.iter()
            .map(|x| x.parse().expect("Well that wasn't an integer"))
            .collect::<Vec<i8>>()
    );
    match simplify::cycle_detect(&dna_args) {
        Ok(_thought_cycle) => println!("Got a cycle!"),
        Err(failure) => println!("Failed to get a cycle: {:?}", failure),
    }
}
