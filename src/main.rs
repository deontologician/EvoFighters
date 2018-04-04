#![feature(box_patterns)]
#![feature(nll)]

extern crate clap;
#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate enum_primitive;
extern crate num;
extern crate num_cpus;
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
mod cli;
mod creatures;
mod dna;
mod eval;
mod parsing;
mod rng;
mod saver;
mod stats;
mod sim;
mod simplify;

fn main() {
    let app = cli::parse_args();
    cli::execute_command(&app);
}
