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
    let mut v: Vec<u8> = vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];

    for x in v.iter_mut() {
        *x = rng.gen();
    }
    
    let mut parser = parsing::Parser::new(rc::Rc::new(v));
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
