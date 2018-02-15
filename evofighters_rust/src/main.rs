#![feature(box_patterns)]
#![feature(nll)]

#[macro_use]
extern crate enum_primitive;
extern crate num;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate time;
extern crate xz2;

#[macro_use]
pub mod util;
pub mod dna;
pub mod parsing;
pub mod eval;
pub mod settings;
pub mod creatures;
pub mod arena;
pub mod simplify;
pub mod saver;

use std::env;

use creatures::Creatures;
use arena::Arena;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        run_simulation();
        return;
    }
    let command = args[1].clone();
    match command.as_ref() {
        "simulate" => run_simulation(),
        "cycle-check" => cycle_check(&args.split_off(2)),
        _ => println!("Unrecognized command."),
    }
}

fn run_simulation() {
    println!("Creating initial population");
    let population: Creatures = Creatures::new(settings::MAX_POPULATION_SIZE);
    println!("Created {} creatures", settings::MAX_POPULATION_SIZE);

    let mut arena = Arena::new(population, "evofighters.save");
    arena.simulate()
}

fn cycle_check(args: &[String]) {
    let dna_args: dna::DNA = dna::DNA::from(
        args.iter()
            .map(|x| x.parse().expect("Well that wasn't an integer"))
            .collect::<Vec<i8>>(),
    );
    match simplify::cycle_detect(&dna_args) {
        Ok(_thought_cycle) => println!("Got a cycle!"),
        Err(failure) => println!("Failed to get a cycle: {:?}", failure),
    }
}
