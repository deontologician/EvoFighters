#![allow(unstable)]

#[macro_use]
pub mod util;
pub mod dna;
pub mod parsing;
pub mod eval;
pub mod settings;
pub mod creatures;

fn main() {
    let mut x : Vec<usize> = Vec::with_capacity(3);
    x.push(3us);
    x.push(4us);
    print1!("print1 It works! {}", x[x.len() - 1]);
    print2!("print2 {}", x[x.len() - 2]);
    print3!("print3 {}", x[x.len() - 2]);
}
