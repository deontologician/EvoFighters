#![feature(rand)]
#![feature(core)]
#![feature(box_syntax)]
#![feature(collections)]
#![feature(path)]
#![feature(std_misc)]
#![feature(io)]
#![feature(os)]
#![feature(env)]

extern crate time;
extern crate "rustc-serialize" as rustc_serialize;

use std::env;
use std::ffi::OsString;
use std::str::FromStr;

#[macro_use]
pub mod util;
pub mod dna;
pub mod parsing;
pub mod eval;
pub mod settings;
pub mod creatures;
pub mod arena;
pub mod simplify;

use creatures::Creature;
use std::iter::FromIterator;


fn main() {
    let mut args: Vec<String> = FromIterator::from_iter(
        env::args().map(|x:OsString| x.into_string().unwrap()));
    if args.len() < 2 {
        run_simulation();
        return
    }
    let command = args[1].clone();
    match command.as_slice() {
        "simulate" => run_simulation(),
        "cycle-check" => cycle_check(args.split_off(2)),
        _ => println!("Unrecognized command."),
    }
}

fn run_simulation() {
    let mut app = util::AppState::new(settings::MAX_POPULATION_SIZE + 1);
    println!("Creating initial population");
    let mut population: Vec<Creature> = FromIterator::from_iter(
        (1..settings::MAX_POPULATION_SIZE + 1)
            .map(|id| Creature::seed_creature(id)));
    println!("Created {} creatures", settings::MAX_POPULATION_SIZE + 1);

    arena::simulate(&mut population, 0, 0, &mut app);
}


fn cycle_check(args: Vec<String>) {
    let dna_args: Vec<i8> = FromIterator::from_iter(args.iter().map(
        |x| FromStr::from_str(&x).ok().expect("Well, that wasn't a number")));
    simplify::cycle_detect(&dna_args);
}
