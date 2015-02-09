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
    fn next(iter: &mut parsing::Parser) -> dna::ConditionTree {
        match iter.next().unwrap() {
            parsing::Thought::Decision{box tree, ..} => tree,
            parsing::Thought::Indecision{reason, ..} => panic!("Oh shit: {:?}", reason),
        }
    }

    let dna_args: Vec<i8> = FromIterator::from_iter(args.iter().map(
        |x| FromStr::from_str(&x).ok().expect("Well, that wasn't a number")));

    let f = |offset: usize| -> usize {
        let mut parser = parsing::Parser::new(dna_args.clone(), offset);
        next(&mut parser);
        parser.current_offset()
    };

    let mut tortoise = f(0);
    let mut hare = f(tortoise);
    println!("tortoise: {}, hare: {}", tortoise, hare);
    while tortoise != hare {
        tortoise = f(tortoise);
        hare = f(f(hare));
        println!("tortoise: {}, hare: {}", tortoise, hare);
    }
    println!("Cycle found between {} and {}\n\n", tortoise, hare);
    println!("Going to find the start index of the cycle now");
    let mut mu = 0;
    // reset tortoise
    tortoise = 0;
    println!("tortoise: {}, hare: {}", tortoise, hare);
    while tortoise != hare {
        tortoise = f(tortoise);
        hare = f(hare);
        println!("tortoise: {}, hare: {}", tortoise, hare);
        mu += 1;
    }
    println!("Discovered that first index of cycle is {}\n\n", mu);
    println!("Going to find the shortest cycle length now.");
    let mut lam = 1;
    hare = f(tortoise);
    println!("tortoise: {}, hare: {}", tortoise, hare);
    while tortoise != hare {
        hare = f(hare);
        println!("tortoise: {}, hare: {}", tortoise, hare);
        lam += 1;
    }
    println!("Cycle starts at {} and has length {}", mu, lam);
    let mut new_iter = parsing::Parser::new(dna_args.clone(), 0);
    let mut thing;
    for i in 0..(mu + lam) {
        let mut offset = new_iter.current_offset();
        thing = next(&mut new_iter);
        if i == mu {
            println!("----- Start cycle ---");
        }
        println!("{}: {:?}", offset, thing);
        let mut simplified = simplify::simplify(thing.clone());
        if simplified != thing {
            println!("    --> {:?}", simplified);
        }
    }

}
