#![feature(box_patterns)]
#![feature(nll)]

extern crate clap;
#[macro_use]
extern crate enum_primitive;
extern crate num;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate time;
extern crate twox_hash;
extern crate xz2;

#[macro_use]
mod util;

mod arena;
mod creatures;
mod dna;
mod eval;
mod parsing;
mod rng;
mod saver;
mod stats;
mod settings;
mod simplify;

use arena::Arena;
use creatures::Creatures;
use saver::Saver;

fn main() {
    let app = parse_args();

    match app.subcommand() {
        ("cycle-check", Some(check)) => {
            cycle_check(check.values_of("bases").unwrap())
        }
        _ => run_simulation(app.value_of("savefile").unwrap()),
    }
}

fn parse_args() -> clap::ArgMatches<'static> {
    clap::App::new(
        r"   __             ___
  /              /    /      /    /
 (___       ___ (___    ___ (___ (___  ___  ___  ___
 |     \  )|   )|    | |   )|   )|    |___)|   )|___
 |__    \/ |__/ |    | |__/ |  / |__  |__  |     __/
                       __/ ",
    ).version("1.0")
        .author("Josh Kuhn <deontologician@gmail.com>")
        .about("Evolving fighting bots")
        .arg(
            clap::Arg::with_name("savefile")
                .short("f")
                .long("file")
                .default_value("evofighters.evo")
                .value_name("SAVEFILE")
                .help("Name of save file")
                .takes_value(true),
        )
        .subcommand(
            clap::SubCommand::with_name("simulate")
                .about("Main command. Runs an evofighters simulation"),
        )
        .subcommand(
            clap::SubCommand::with_name("cycle-check")
                .about("Does a cycle detection on the given bases")
                .arg(
                    clap::Arg::with_name("bases")
                        .required(true)
                        .multiple(true)
                        .value_name("BASE"),
                ),
        )
        .get_matches()
}

fn run_simulation(filename: &str) {
    let mut arena = match Saver::load(filename) {
        Ok(checkpoint) => {
            println!("Loading from file {}", filename);
            Arena::from_checkpoint(checkpoint, filename)
        }
        Err(_) => {
            println!("Creating initial population");
            let population: Creatures =
                Creatures::new(settings::MAX_POPULATION_SIZE);
            println!("Created {} creatures", settings::MAX_POPULATION_SIZE);
            Arena::new(population, filename)
        }
    };
    arena.simulate()
}

fn cycle_check(bases: clap::Values) {
    let dna_args: dna::DNA = dna::DNA::from(
        bases
            .map(|x| x.parse().expect("Well that wasn't an integer"))
            .collect::<Vec<i8>>(),
    );
    match simplify::cycle_detect(&dna_args) {
        Ok(_thought_cycle) => println!("Got a cycle!"),
        Err(failure) => println!("Failed to get a cycle: {:?}", failure),
    }
}
