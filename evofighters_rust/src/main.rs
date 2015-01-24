#![allow(unstable)]
#![feature(box_syntax)]

use std::rand;
use std::rand::Rng;
use std::num::Int;
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
    let mut rng = rand::thread_rng();
    println!("We got {}", 12us.saturating_sub(13us));
    let mut v: Vec<i8> = vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];

    for x in v.iter_mut() {
        *x = rng.gen_range(-1, 8);
    }
    let owned_v = rc::Rc::new(v);
    println!("Full: {:?}", owned_v);
    let chunks = creatures::gene_primer(owned_v.clone());
    println!("Chunked: {:?}", chunks);

    let mut parser = parsing::Parser::new(owned_v.clone());
    let thought = parser.next().expect("No thought!");
    match thought {
        parsing::Thought::Decision {tree, icount, skipped} =>
            println!("icount: {}, skipped: {}, tree:\n{:?}",
                     icount, skipped, tree),
        parsing::Thought::Indecision {reason, icount, skipped} =>
            println!("icount: {}, skipped: {}, failed because: {:?}",
                     icount, skipped, reason)
    }

}
